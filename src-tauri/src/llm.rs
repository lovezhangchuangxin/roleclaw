use crate::domain::ModelProviderConfig;
use futures_util::StreamExt;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatStreamEvent};
use genai::resolver::{AuthData, Endpoint, Result as ResolverResult};
use genai::{Client, ModelIden, ServiceTarget, WebConfig};
use std::time::Duration;

const CONNECTIVITY_MAX_TOKENS: u32 = 64;
const TURN_NARRATION_MAX_TOKENS: u32 = 2200;

fn build_openai_compatible_client(config: &ModelProviderConfig) -> Result<Client, String> {
    let base_url = config.base_url.trim_end_matches('/').to_string();
    let api_key = config
        .api_key
        .clone()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| "apiKey 不能为空".to_string())?;

    let model_mapper = move |model_iden: ModelIden| -> ResolverResult<ModelIden> {
        Ok(ModelIden::new(
            AdapterKind::OpenAI,
            model_iden.model_name.to_string(),
        ))
    };

    let auth_resolver = move |_model_iden: ModelIden| -> ResolverResult<Option<AuthData>> {
        Ok(Some(AuthData::from_single(api_key.clone())))
    };

    let target_resolver = move |mut target: ServiceTarget| -> ResolverResult<ServiceTarget> {
        target.endpoint = Endpoint::from_owned(base_url.clone());
        Ok(target)
    };

    let web_config =
        WebConfig::default().with_timeout(Duration::from_millis(config.timeout_ms as u64));

    Ok(Client::builder()
        .with_web_config(web_config)
        .with_model_mapper_fn(model_mapper)
        .with_auth_resolver_fn(auth_resolver)
        .with_service_target_resolver_fn(target_resolver)
        .build())
}

pub async fn test_provider_connectivity(config: &ModelProviderConfig) -> Result<String, String> {
    if config.provider_type != "openai_compatible" {
        return Err("当前仅支持 openai_compatible 协议".to_string());
    }

    let client = build_openai_compatible_client(config)?;
    let req = ChatRequest::from_user("Reply with exactly: pong");
    let options = ChatOptions::default()
        .with_temperature(0.0)
        .with_max_tokens(CONNECTIVITY_MAX_TOKENS);
    let chat_res = client
        .exec_chat(&config.model, req, Some(&options))
        .await
        .map_err(|e| format!("genai 连通性测试失败: {e}"))?;

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

pub async fn generate_narration(
    config: &ModelProviderConfig,
    prompt: &str,
) -> Result<String, String> {
    let client = build_openai_compatible_client(config)?;
    let req = ChatRequest::from_user(prompt)
        .with_system("你是一个中文 RPG 叙事引擎，回复仅输出叙事文本，不要解释。");
    let options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(TURN_NARRATION_MAX_TOKENS);
    let chat_res = client
        .exec_chat(&config.model, req, Some(&options))
        .await
        .map_err(|e| format!("genai 生成失败: {e}"))?;
    let text = chat_res.first_text().unwrap_or("").trim().to_string();
    if text.is_empty() {
        return Err("模型返回了空内容".to_string());
    }
    Ok(text)
}

pub async fn stream_narration(
    config: &ModelProviderConfig,
    prompt: &str,
    on_chunk: &mut (dyn FnMut(&str) -> Result<(), String> + Send),
) -> Result<String, String> {
    let client = build_openai_compatible_client(config)?;
    let req = ChatRequest::from_user(prompt)
        .with_system("你是一个中文 RPG 叙事引擎，回复仅输出叙事文本，不要解释。");
    let options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(TURN_NARRATION_MAX_TOKENS);
    let mut stream_res = client
        .exec_chat_stream(&config.model, req, Some(&options))
        .await
        .map_err(|e| format!("genai 流式启动失败: {e}"))?;

    let mut narration = String::new();
    while let Some(event) = stream_res.stream.next().await {
        let event = event.map_err(|e| format!("genai 流式事件错误: {e}"))?;
        if let ChatStreamEvent::Chunk(chunk) = event {
            narration.push_str(&chunk.content);
            on_chunk(&chunk.content)?;
        }
    }
    let narration = narration.trim().to_string();
    if narration.is_empty() {
        return Err("模型流式返回了空内容".to_string());
    }
    Ok(narration)
}
