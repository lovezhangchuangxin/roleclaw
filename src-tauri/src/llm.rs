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

/// 支持的模型提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    OpenAI,           // OpenAI 兼容 (OpenAI, 第三方兼容 API)
    Anthropic,       // Anthropic Claude
    Google,          // Google Gemini
    AzureOpenAI,     // Azure OpenAI
    Ollama,          // 本地 Ollama
    Groq,            // Groq
    Cohere,          // Cohere
    Mistral,         // Mistral
    Custom,          // 自定义 API
}

impl ProviderType {
    /// 从字符串解析提供商类型
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" | "openai_compatible" => ProviderType::OpenAI,
            "anthropic" | "claude" => ProviderType::Anthropic,
            "google" | "gemini" => ProviderType::Google,
            "azure" | "azure_openai" => ProviderType::AzureOpenAI,
            "ollama" => ProviderType::Ollama,
            "groq" => ProviderType::Groq,
            "cohere" => ProviderType::Cohere,
            "mistral" => ProviderType::Mistral,
            "custom" => ProviderType::Custom,
            _ => ProviderType::OpenAI, // 默认使用 OpenAI 兼容模式
        }
    }

    /// 获取对应的 AdapterKind
    pub fn to_adapter(self) -> Result<AdapterKind, String> {
        match self {
            ProviderType::OpenAI => Ok(AdapterKind::OpenAI),
            ProviderType::Anthropic => Ok(AdapterKind::Anthropic),
            ProviderType::Google => Ok(AdapterKind::Google),
            ProviderType::AzureOpenAI => Ok(AdapterKind::AzureOpenAI),
            ProviderType::Ollama => Ok(AdapterKind::Ollama),
            ProviderType::Groq => Ok(AdapterKind::Groq),
            ProviderType::Cohere => Ok(AdapterKind::Cohere),
            ProviderType::Mistral => Ok(AdapterKind::Mistral),
            ProviderType::Custom => Ok(AdapterKind::OpenAI), // 自定义也用 OpenAI 兼容模式
        }
    }

    /// 是否需要 API Key
    pub fn requires_api_key(self) -> bool {
        match self {
            ProviderType::Ollama => false, // 本地部署不需要 API Key
            _ => true,
        }
    }

    /// 获取默认的 Base URL
    pub fn default_base_url(self) -> &'static str {
        match self {
            ProviderType::OpenAI => "https://api.openai.com/v1",
            ProviderType::Anthropic => "https://api.anthropic.com/v1",
            ProviderType::Google => "https://generativelanguage.googleapis.com/v1",
            ProviderType::AzureOpenAI => "", // Azure 需要用户配置
            ProviderType::Ollama => "http://localhost:11434",
            ProviderType::Groq => "https://api.groq.com/openai/v1",
            ProviderType::Cohere => "https://api.cohere.ai/v1",
            ProviderType::Mistral => "https://api.mistral.ai/v1",
            ProviderType::Custom => "", // 自定义需要用户配置
        }
    }

    /// 获取支持的模型列表
    pub fn supported_models(self) -> Vec<&'static str> {
        match self {
            ProviderType::OpenAI => vec![
                "gpt-4o",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "gpt-4",
                "gpt-3.5-turbo",
                "o1",
                "o1-mini",
                "o3-mini",
            ],
            ProviderType::Anthropic => vec![
                "claude-sonnet-4-20250514",
                "claude-sonnet-4-20250507",
                "claude-3-5-sonnet-20241022",
                "claude-3-5-sonnet-20240620",
                "claude-3-opus-20240229",
                "claude-3-haiku-20240307",
            ],
            ProviderType::Google => vec![
                "gemini-2.0-flash-exp",
                "gemini-2.0-flash",
                "gemini-1.5-pro",
                "gemini-1.5-flash",
                "gemini-1.5-flash-8b",
            ],
            ProviderType::AzureOpenAI => vec![
                "gpt-4o",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "gpt-35-turbo",
            ],
            ProviderType::Ollama => vec![
                "llama3.3",
                "llama3.2",
                "llama3.1",
                "llama3",
                "qwen2.5",
                "qwen2",
                "mistral",
                "phi4",
                "deepseek-r1",
                "gemma2",
            ],
            ProviderType::Groq => vec![
                "llama-3.3-70b-versatile",
                "llama-3.1-70b-versatile",
                "llama-3.1-8b-instant",
                "mixtral-8x7b-32768",
                "gemma2-9b-it",
            ],
            ProviderType::Cohere => vec![
                "command-r-plus",
                "command-r",
                "command",
                "command-light",
            ],
            ProviderType::Mistral => vec![
                "mistral-large-latest",
                "mistral-small-latest",
                "mistral-medium-latest",
                "pixtral-large-latest",
                "pixtral-small-latest",
            ],
            ProviderType::Custom => vec![], // 自定义由用户指定
        }
    }
}

/// 获取提供商的默认 URL
pub fn get_default_base_url(provider_type: &str) -> String {
    ProviderType::from_str(provider_type)
        .default_base_url()
        .to_string()
}

/// 获取支持的模型列表
pub fn get_supported_models(provider_type: &str) -> Vec<String> {
    ProviderType::from_str(provider_type)
        .supported_models()
        .into_iter()
        .map(String::from)
        .collect()
}

/// 检查提供商是否需要 API Key
pub fn provider_requires_api_key(provider_type: &str) -> bool {
    ProviderType::from_str(provider_type).requires_api_key()
}

fn to_adapter(provider_type: &str) -> Result<AdapterKind, String> {
    ProviderType::from_str(provider_type).to_adapter()
}

fn configured_max_tokens(config: &ModelProviderConfig, fallback: u32, scenario_cap: u32) -> u32 {
    let requested = config.max_tokens.unwrap_or(fallback);
    requested.clamp(1, scenario_cap.min(PROVIDER_MAX_TOKENS_CAP))
}

fn is_reasoning_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.contains("reasoner")
        || lower.contains("r1")
        || lower.contains("o1")
        || lower.contains("o3")
        || lower.contains("deepseek-r1")
        || lower.contains("qwq")
}

fn is_vision_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.contains("vision")
        || lower.contains("gpt-4o")
        || lower.contains("claude-3")
        || lower.contains("gemini-1.5")
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
    let provider_type = ProviderType::from_str(&config.provider_type);
    
    // 检查是否需要 API Key
    if provider_type.requires_api_key() {
        let _ = config
            .api_key
            .clone()
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| "apiKey 不能为空".to_string())?;
    }
    
    let adapter = provider_type.to_adapter()?;
    
    eprintln!(
        "[llm] build_client providerType={} provider={} model={} baseUrlRaw={} baseUrlNormalized={} timeoutMs={}",
        config.provider_type,
        config.provider,
        config.model,
        config.base_url,
        base_url,
        config.timeout_ms
    );

    let api_key = config.api_key.clone();
    
    let model_mapper = move |model_iden: ModelIden| -> ResolverResult<ModelIden> {
        Ok(ModelIden::new(
            adapter.clone(),
            model_iden.model_name.to_string(),
        ))
    };

    let auth_resolver = move |_model_iden: ModelIden| -> ResolverResult<Option<AuthData>> {
        if let Some(key) = api_key.as_ref() {
            if !key.trim().is_empty() {
                return Ok(Some(AuthData::from_single(key.clone())));
            }
        }
        Ok(None)
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
    let provider_type = ProviderType::from_str(&config.provider_type);
    
    // 检查 API Key
    if provider_type.requires_api_key() {
        if config.api_key.as_ref().map(|k| k.trim().is_empty()).unwrap_or(true) {
            return Err("API Key 不能为空".to_string());
        }
    }
    
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
    system_prompt: &str,
    context_prompt: &str,
) -> Result<String, String> {
    let client = build_client(config)?;
    let chat_options = build_turn_chat_options(config);

    let req = ChatRequest::new(vec![
        system_prompt.into(),
        context_prompt.into(),
    ]);

    let chat_res = client
        .exec_chat(&config.model, req, Some(&chat_options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] generate_turn_json failed model={} baseUrl={} err={:?}",
                config.model, config.base_url, e
            );
            format!("genai 调用失败: {e}")
        })?;

    let text = chat_res
        .first_text()
        .ok_or_else(|| "模型返回为空".to_string())?;
    Ok(text)
}

pub async fn generate_turn_json_stream(
    config: &ModelProviderConfig,
    system_prompt: &str,
    context_prompt: &str,
) -> Result<impl Stream<Item = Result<TurnJsonStreamPiece, String>> + '_ , String> {
    let client = build_client(config)?;
    let chat_options = build_turn_chat_options(config);

    let req = ChatRequest::new(vec![
        system_prompt.into(),
        context_prompt.into(),
    ]);

    let mut stream = client
        .exec_chat_stream(&config.model, req, Some(&chat_options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] generate_turn_json_stream failed model={} baseUrl={} err={:?}",
                config.model, config.base_url, e
            );
            format!("genai 流式调用失败: {e}")
        })?;

    Ok(async_stream::stream! {
        while let Some(item) = stream.next().await {
            match item {
                Ok(ChatStreamEvent::Content { text, .. }) => {
                    if !text.is_empty() {
                        yield Ok(TurnJsonStreamPiece::Content(text));
                    }
                }
                Ok(ChatStreamEvent::ReasoningContent { text, .. }) => {
                    if !text.is_empty() {
                        yield Ok(TurnJsonStreamPiece::Reasoning(text));
                    }
                }
                Ok(ChatStreamEvent::Done) => break,
                Err(e) => {
                    yield Err(format!("流式响应错误: {e}"));
                    break;
                }
                _ => {}
            }
        }
    })
}

pub async fn generate_world_card_json(
    config: &ModelProviderConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    let client = build_client(config)?;
    let options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(configured_max_tokens(
            config,
            WORLD_CARD_MAX_TOKENS,
            WORLD_CARD_MAX_TOKENS,
        ));

    let req = ChatRequest::new(vec![system_prompt.into(), user_prompt.into()]);

    let chat_res = client
        .exec_chat(&config.model, req, Some(&options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] generate_world_card_json failed model={} err={:?}",
                config.model, e
            );
            format!("生成世界卡失败: {e}")
        })?;

    let text = chat_res
        .first_text()
        .ok_or_else(|| "模型返回为空".to_string())?;
    Ok(text)
}

pub async fn generate_world_card_json_stream(
    config: &ModelProviderConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<impl Stream<Item = Result<String, String>> + '_ , String> {
    let client = build_client(config)?;
    let options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(configured_max_tokens(
            config,
            WORLD_CARD_MAX_TOKENS,
            WORLD_CARD_MAX_TOKENS,
        ))
        .with_capture_reasoning_content(true);

    let req = ChatRequest::new(vec![system_prompt.into(), user_prompt.into()]);

    let mut stream = client
        .exec_chat_stream(&config.model, req, Some(&options))
        .await
        .map_err(|e| {
            eprintln!(
                "[llm] generate_world_card_json_stream failed model={} err={:?}",
                config.model, e
            );
            format!("生成世界卡失败: {e}")
        })?;

    Ok(async_stream::stream! {
        let mut content = String::new();
        while let Some(item) = stream.next().await {
            match item {
                Ok(ChatStreamEvent::Content { text, .. }) => {
                    content.push_str(&text);
                    yield Ok(text);
                }
                Ok(ChatStreamEvent::ReasoningContent { text, .. }) => {
                    // 思考过程不输出给用户
                    eprint!("[world-card reasoning] {}", text);
                }
                Ok(ChatStreamEvent::Done) => break,
                Err(e) => {
                    yield Err(format!("流式响应错误: {e}"));
                    break;
                }
                _ => {}
            }
        }
        eprintln!("[world-card] Total content length: {}", content.len());
    })
}
