use crate::domain::ModelProviderConfig;
use futures_util::StreamExt;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatStreamEvent, ReasoningEffort};
use genai::resolver::{AuthData, Endpoint, Result as ResolverResult};
use genai::{Client, ModelIden, ServiceTarget, WebConfig};
use std::time::Duration;

const CONNECTIVITY_MAX_TOKENS: u32 = 64;
const TURN_JSON_MAX_TOKENS: u32 = 5200;
const WORLD_CARD_MAX_TOKENS: u32 = 6400;
const PROVIDER_MAX_TOKENS_CAP: u32 = 65_536;

pub enum TurnJsonStreamPiece {
    Content(String),
    Reasoning(String),
}

fn to_adapter(provider_type: &str) -> Result<AdapterKind, String> {
    match provider_type {
        "openai_compatible" => Ok(AdapterKind::OpenAI),
        other => Err(format!(
            "不支持的 providerType: {}（当前仅支持 openai_compatible）",
            other
        )),
    }
}

fn configured_max_tokens(config: &ModelProviderConfig, fallback: u32, scenario_cap: u32) -> u32 {
    let requested = config.max_tokens.unwrap_or(fallback);
    requested.clamp(1, scenario_cap.min(PROVIDER_MAX_TOKENS_CAP))
}

fn is_reasoning_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.contains("reasoner") || lower.contains("r1")
}

fn build_turn_chat_options(config: &ModelProviderConfig) -> ChatOptions {
    let mut options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(configured_max_tokens(
            config,
            TURN_JSON_MAX_TOKENS,
            TURN_JSON_MAX_TOKENS,
        ))
        .with_capture_reasoning_content(true);
    if is_reasoning_model(&config.model) {
        options = options.with_reasoning_effort(ReasoningEffort::Low);
    }
    options
}

fn normalize_base_url_for_join(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.ends_with('/') {
        trimmed.to_string()
    } else {
        format!("{trimmed}/")
    }
}

fn build_client(config: &ModelProviderConfig) -> Result<Client, String> {
    let base_url = normalize_base_url_for_join(&config.base_url);
    let api_key = config
        .api_key
        .clone()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| "apiKey 不能为空".to_string())?;
    let adapter = to_adapter(&config.provider_type)?;
    eprintln!(
        "[llm] build_client providerType={} provider={} model={} baseUrlRaw={} baseUrlNormalized={} timeoutMs={}",
        config.provider_type,
        config.provider,
        config.model,
        config.base_url,
        base_url,
        config.timeout_ms
    );

    let model_mapper = move |model_iden: ModelIden| -> ResolverResult<ModelIden> {
        Ok(ModelIden::new(
            adapter.clone(),
            model_iden.model_name.to_string(),
        ))
    };

    let auth_resolver = move |_model_iden: ModelIden| -> ResolverResult<Option<AuthData>> {
        Ok(Some(AuthData::from_single(api_key.clone())))
    };

    let target_resolver = move |mut target: ServiceTarget| -> ResolverResult<ServiceTarget> {
        if !base_url.is_empty() {
            target.endpoint = Endpoint::from_owned(base_url.clone());
        }
        Ok(target)
    };

    // Use idle/read timeout instead of total request timeout:
    // as long as stream bytes continue to arrive, the request stays alive.
    let mut web_config = WebConfig::default().with_connect_timeout(Duration::from_millis(15_000));
    web_config.read_timeout = Some(Duration::from_millis(config.timeout_ms as u64));

    Ok(Client::builder()
        .with_web_config(web_config)
        .with_model_mapper_fn(model_mapper)
        .with_auth_resolver_fn(auth_resolver)
        .with_service_target_resolver_fn(target_resolver)
        .build())
}

pub async fn test_provider_connectivity(config: &ModelProviderConfig) -> Result<String, String> {
    let _ = to_adapter(&config.provider_type)?;

    let client = build_client(config)?;
    let req = ChatRequest::from_user("Reply with exactly: pong");
    let options = ChatOptions::default()
        .with_temperature(0.0)
        .with_max_tokens(configured_max_tokens(
            config,
            CONNECTIVITY_MAX_TOKENS,
            CONNECTIVITY_MAX_TOKENS,
        ));
    let chat_res = client
        .exec_chat(&config.model, req, Some(&options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] connectivity failed model={} baseUrl={} err={:?}",
                config.model, config.base_url, e
            );
            format!("genai 连通性测试失败: {e}")
        })?;

    let reply = chat_res
        .first_text()
        .unwrap_or("")
        .chars()
        .take(80)
        .collect::<String>();
    Ok(format!(
        "{} / {} 已连通（{}），模型回复片段：{}",
        config.provider, config.model, config.base_url, reply
    ))
}

pub async fn generate_turn_json(
    config: &ModelProviderConfig,
    prompt: &str,
) -> Result<String, String> {
    let client = build_client(config)?;
    let req = ChatRequest::from_user(prompt).with_system(
        "你是中文 RPG 回合引擎。你必须只输出一个 JSON 对象，不要输出 markdown、代码块或解释。",
    );
    let options = build_turn_chat_options(config);
    let chat_res = client
        .exec_chat(&config.model, req, Some(&options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] turn json failed model={} baseUrl={} err={:?}",
                config.model, config.base_url, e
            );
            format!("genai 回合 JSON 生成失败: {e}")
        })?;
    let text = chat_res.first_text().unwrap_or("").trim().to_string();
    if text.is_empty() {
        return Err("模型返回了空内容".to_string());
    }
    Ok(text)
}

pub async fn stream_turn_json(
    config: &ModelProviderConfig,
    prompt: &str,
    on_piece: &mut (dyn FnMut(TurnJsonStreamPiece) -> Result<(), String> + Send),
) -> Result<String, String> {
    let client = build_client(config)?;
    let req = ChatRequest::from_user(prompt).with_system(
        "你是中文 RPG 回合引擎。你必须只输出一个 JSON 对象，不要输出 markdown、代码块或解释。",
    );
    let options = build_turn_chat_options(config);
    let mut stream_res = client
        .exec_chat_stream(&config.model, req, Some(&options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] turn json stream start failed model={} baseUrl={} err={:?}",
                config.model, config.base_url, e
            );
            format!("genai 回合 JSON 流式启动失败: {e}")
        })?;

    let mut out = String::new();
    while let Some(event) = stream_res.stream.next().await {
        let event = event.map_err(|e| format!("genai 回合 JSON 流式事件错误: {e}"))?;
        match event {
            ChatStreamEvent::Chunk(chunk) => {
                out.push_str(&chunk.content);
                on_piece(TurnJsonStreamPiece::Content(chunk.content))?;
            }
            ChatStreamEvent::ReasoningChunk(chunk) => {
                let reasoning = chunk.content;
                if !reasoning.is_empty() {
                    on_piece(TurnJsonStreamPiece::Reasoning(reasoning))?;
                }
            }
            _ => {}
        }
    }
    // eprintln!(
    //     "[llm] turn json stream completed chunks={} totalChars={}",
    //     chunk_count,
    //     out.chars().count()
    // );
    let text = out.trim().to_string();
    if text.is_empty() {
        return Err("模型流式返回了空内容".to_string());
    }
    Ok(text)
}

pub async fn generate_world_card_json(
    config: &ModelProviderConfig,
    user_prompt: &str,
) -> Result<String, String> {
    let client = build_client(config)?;
    eprintln!(
        "[llm] world-card request model={} baseUrl={} promptLen={}",
        config.model,
        config.base_url,
        user_prompt.chars().count()
    );
    let system_prompt = "你是 RPG 世界卡设计器。你必须只输出一个合法 JSON 对象，不要输出 markdown、代码块标记或任何解释文本。\
JSON 必须符合 RoleClaw WorldCard v2（camelCase）：\
id,name,schemaVersion,contentVersion,worldbook,map,npcs,events,chapterGoals。\
地图边是无向边，a/b 必须引用有效节点，startNodeId 必须存在。\
npcs 每项字段仅: id,name,personality(string[]),identity。\
events 每项字段仅: id,name,prompt。\
chapterGoals 每项字段仅: id,title,prompt。\
x/y 为数字，canvas 必须包含 width,height。";
    let req = ChatRequest::from_user(user_prompt).with_system(system_prompt);
    let options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(configured_max_tokens(
            config,
            WORLD_CARD_MAX_TOKENS,
            WORLD_CARD_MAX_TOKENS,
        ));
    let chat_res = client
        .exec_chat(&config.model, req, Some(&options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] world-card failed model={} baseUrl={} err={:?}",
                config.model, config.base_url, e
            );
            format!("genai 世界卡生成失败: {e}")
        })?;
    let text = chat_res.first_text().unwrap_or("").trim().to_string();
    if text.is_empty() {
        return Err("模型返回了空内容".to_string());
    }
    Ok(text)
}

pub async fn stream_world_card_json(
    config: &ModelProviderConfig,
    user_prompt: &str,
    on_chunk: &mut (dyn FnMut(&str) -> Result<(), String> + Send),
) -> Result<String, String> {
    let client = build_client(config)?;
    eprintln!(
        "[llm] world-card stream request model={} baseUrl={} promptLen={}",
        config.model,
        config.base_url,
        user_prompt.chars().count()
    );
    let system_prompt = "你是 RPG 世界卡设计器。你必须只输出一个合法 JSON 对象，不要输出 markdown、代码块标记或任何解释文本。\
JSON 必须符合 RoleClaw WorldCard v2（camelCase）：\
id,name,schemaVersion,contentVersion,worldbook,map,npcs,events,chapterGoals。\
地图边是无向边，a/b 必须引用有效节点，startNodeId 必须存在。\
npcs 每项字段仅: id,name,personality(string[]),identity。\
events 每项字段仅: id,name,prompt。\
chapterGoals 每项字段仅: id,title,prompt。\
x/y 为数字，canvas 必须包含 width,height。";
    let req = ChatRequest::from_user(user_prompt).with_system(system_prompt);
    let options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(configured_max_tokens(
            config,
            WORLD_CARD_MAX_TOKENS,
            WORLD_CARD_MAX_TOKENS,
        ));
    let mut stream_res = client
        .exec_chat_stream(&config.model, req, Some(&options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] world-card stream start failed model={} baseUrl={} err={:?}",
                config.model, config.base_url, e
            );
            format!("genai 世界卡流式生成启动失败: {e}")
        })?;

    let mut out = String::new();
    while let Some(event) = stream_res.stream.next().await {
        let event = event.map_err(|e| format!("genai 世界卡流式事件错误: {e}"))?;
        if let ChatStreamEvent::Chunk(chunk) = event {
            out.push_str(&chunk.content);
            on_chunk(&chunk.content)?;
        }
    }
    let text = out.trim().to_string();
    if text.is_empty() {
        return Err("模型流式返回了空内容".to_string());
    }
    Ok(text)
}
