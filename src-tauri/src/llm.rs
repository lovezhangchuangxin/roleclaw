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
    // 国际厂商
    OpenAI,
    Anthropic,
    Google,
    AzureOpenAI,
    Ollama,
    Groq,
    Cohere,
    Mistral,
    
    // 国内厂商
    Zhipu,
    Baidu,
    Alibaba,
    Tencent,
    ByteDance,
    MiniMax,
    StepFun,
    Kimi,
    DeepSeek,
    
    Custom,
}

impl ProviderType {
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
            
            "zhipu" | "glm" | "zhipuai" => ProviderType::Zhipu,
            "baidu" | "ernie" | "wenxin" => ProviderType::Baidu,
            "alibaba" | "qwen" | "tongyi" => ProviderType::Alibaba,
            "tencent" | "hunyuan" => ProviderType::Tencent,
            "bytedance" | "doubao" => ProviderType::ByteDance,
            "minimax" | "abab" => ProviderType::MiniMax,
            "stepfun" | "step" => ProviderType::StepFun,
            "kimi" | "moonshot" => ProviderType::Kimi,
            "deepseek" => ProviderType::DeepSeek,
            
            "custom" => ProviderType::Custom,
            _ => ProviderType::OpenAI,
        }
    }

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
            _ => Ok(AdapterKind::OpenAI), // 国内厂商都用 OpenAI 兼容模式
        }
    }

    pub fn requires_api_key(self) -> bool {
        !matches!(self, ProviderType::Ollama)
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            ProviderType::OpenAI => "https://api.openai.com/v1",
            ProviderType::Anthropic => "https://api.anthropic.com/v1",
            ProviderType::Google => "https://generativelanguage.googleapis.com/v1",
            ProviderType::AzureOpenAI => "",
            ProviderType::Ollama => "http://localhost:11434",
            ProviderType::Groq => "https://api.groq.com/openai/v1",
            ProviderType::Cohere => "https://api.cohere.ai/v1",
            ProviderType::Mistral => "https://api.mistral.ai/v1",
            
            ProviderType::Zhipu => "https://open.bigmodel.cn/api/paas/v4",
            ProviderType::Baidu => "https://qianfan.eyun.com/v2",
            ProviderType::Alibaba => "https://dashscope.aliyuncs.com/api/v1",
            ProviderType::Tencent => "https://hunyuan.tencentcloudapi.com/v2",
            ProviderType::ByteDance => "https://ark.cn-beijing.volces.com/api/v3",
            ProviderType::MiniMax => "https://api.minimax.chat/v1",
            ProviderType::StepFun => "https://api.stepfun.com/v1",
            ProviderType::Kimi => "https://api.moonshot.cn/v1",
            ProviderType::DeepSeek => "https://api.deepseek.com/v1",
            
            ProviderType::Custom => "",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ProviderType::OpenAI => "OpenAI",
            ProviderType::Anthropic => "Anthropic Claude",
            ProviderType::Google => "Google Gemini",
            ProviderType::AzureOpenAI => "Azure OpenAI",
            ProviderType::Ollama => "Ollama (本地)",
            ProviderType::Groq => "Groq",
            ProviderType::Cohere => "Cohere",
            ProviderType::Mistral => "Mistral AI",
            
            ProviderType::Zhipu => "智谱AI (GLM)",
            ProviderType::Baidu => "百度 (ERNIE)",
            ProviderType::Alibaba => "阿里 (通义)",
            ProviderType::Tencent => "腾讯 (混元)",
            ProviderType::ByteDance => "字节 (豆包)",
            ProviderType::MiniMax => "MiniMax",
            ProviderType::StepFun => "阶跃星辰 (Step)",
            ProviderType::Kimi => "Kimi (月之暗面)",
            ProviderType::DeepSeek => "DeepSeek",
            
            ProviderType::Custom => "自定义 API",
        }
    }

    pub fn supported_models(self) -> Vec<&'static str> {
        match self {
            ProviderType::OpenAI => vec![
                "gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-4", "gpt-3.5-turbo",
                "o1", "o1-mini", "o3-mini",
            ],
            ProviderType::Anthropic => vec![
                "claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022",
                "claude-3-opus-20240229", "claude-3-haiku-20240307",
            ],
            ProviderType::Google => vec![
                "gemini-2.0-flash-exp", "gemini-2.0-flash",
                "gemini-1.5-pro", "gemini-1.5-flash", "gemini-1.5-flash-8b",
            ],
            ProviderType::AzureOpenAI => vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
            ProviderType::Ollama => vec![
                "llama3.3", "llama3.2", "llama3.1", "qwen2.5", "deepseek-r1", "gemma2",
            ],
            ProviderType::Groq => vec![
                "llama-3.3-70b-versatile", "llama-3.1-70b-versatile",
                "mixtral-8x7b-32768", "gemma2-9b-it",
            ],
            ProviderType::Cohere => vec!["command-r-plus", "command-r", "command"],
            ProviderType::Mistral => vec!["mistral-large-latest", "mistral-small-latest"],
            
            // 国内厂商
            ProviderType::Zhipu => vec!["glm-4-flash", "glm-4-plus", "glm-4-pro", "glm-3-turbo"],
            ProviderType::Baidu => vec!["ernie-4.0-8k", "ernie-3.5-8k", "ernie-speed-8k", "ernie-lite-8k"],
            ProviderType::Alibaba => vec![
                "qwen-turbo", "qwen-plus", "qwen-max",
                "qwen2-72b-instruct", "qwen2-7b-instruct",
            ],
            ProviderType::Tencent => vec!["hunyuan-pro", "hunyuan-standard", "hunyuan-lite"],
            ProviderType::ByteDance => vec!["doubao-pro-32k", "doubao-lite-32k"],
            ProviderType::MiniMax => vec![
                "MiniMax-M2.1",
                "MiniMax-M2.5",
                "abab6.5s-chat",
            ],
            ProviderType::StepFun => vec!["step-1v-8k", "step-1-flash-8k"],
            ProviderType::Kimi => vec!["kimi-k2", "kimi-k2.5"],
            ProviderType::DeepSeek => vec!["deepseek-chat", "deepseek-reasoner"],
            
            ProviderType::Custom => vec![],
        }
    }

    pub fn default_params(self) -> (f32, Option<u32>) {
        match self {
            ProviderType::OpenAI => (0.7, Some(4096)),
            ProviderType::Anthropic => (0.7, Some(4096)),
            ProviderType::Google => (0.9, Some(8192)),
            ProviderType::MiniMax => (0.95, Some(8192)),
            ProviderType::Kimi => (0.9, Some(8192)),
            _ => (0.95, Some(4096)),
        }
    }
}

pub fn get_default_base_url(provider_type: &str) -> String {
    ProviderType::from_str(provider_type).default_base_url().to_string()
}

pub fn get_provider_display_name(provider_type: &str) -> String {
    ProviderType::from_str(provider_type).display_name().to_string()
}

pub fn get_supported_models(provider_type: &str) -> Vec<String> {
    ProviderType::from_str(provider_type).supported_models().into_iter().map(String::from).collect()
}

pub fn provider_requires_api_key(provider_type: &str) -> bool {
    ProviderType::from_str(provider_type).requires_api_key()
}

pub fn get_default_temperature(provider_type: &str) -> f32 {
    ProviderType::from_str(provider_type).default_params().0
}

pub fn get_default_max_tokens(provider_type: &str) -> Option<u32> {
    ProviderType::from_str(provider_type).default_params().1
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
    lower.contains("reasoner") || lower.contains("r1") || lower.contains("o1")
        || lower.contains("deepseek-r1") || lower.contains("qwq") || lower.contains("k2")
        || lower.contains("step-1v")
}

fn build_turn_chat_options(config: &ModelProviderConfig) -> ChatOptions {
    let mut options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(configured_max_tokens(config, TURN_JSON_MAX_TOKENS, TURN_JSON_MAX_TOKENS))
        .with_capture_reasoning_content(true);
    
    if is_reasoning_model(&config.model) {
        options = options.with_reasoning_effort(ReasoningEffort::Low);
    }
    options
}

fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.ends_with('/') {
        trimmed.to_string()
    } else {
        format!("{trimmed}/")
    }
}

fn build_client(config: &ModelProviderConfig) -> Result<Client, String> {
    let base_url = normalize_base_url(&config.base_url);
    let provider_type = ProviderType::from_str(&config.provider_type);
    
    if provider_type.requires_api_key() {
        config.api_key.clone()
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| "API Key 不能为空".to_string())?;
    }
    
    let adapter = provider_type.to_adapter()?;
    let api_key = config.api_key.clone();
    
    let model_mapper = move |model_iden: ModelIden| -> ResolverResult<ModelIden> {
        Ok(ModelIden::new(adapter.clone(), model_iden.model_name.to_string()))
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
    
    if provider_type.requires_api_key() {
        if config.api_key.as_ref().map(|k| k.trim().is_empty()).unwrap_or(true) {
            return Err("API Key 不能为空".to_string());
        }
    }
    
    let client = build_client(config)?;
    let req = ChatRequest::from_user("Reply with exactly: pong");
    let options = ChatOptions::default()
        .with_temperature(0.0)
        .with_max_tokens(CONNECTIVITY_MAX_TOKENS);
    
    let chat_res = client.exec_chat(&config.model, req, Some(&options)).await
        .map_err(|e| format!("连通性测试失败: {e}"))?;

    let reply = chat_res.first_text().unwrap_or("").chars().take(80).collect::<String>();
    Ok(format!("{} / {} 已连通，回复：{}", config.provider, config.model, reply))
}

pub async fn generate_turn_json(
    config: &ModelProviderConfig,
    system_prompt: &str,
    context_prompt: &str,
) -> Result<String, String> {
    let client = build_client(config)?;
    let chat_options = build_turn_chat_options(config);
    let req = ChatRequest::new(vec![system_prompt.into(), context_prompt.into()]);

    let chat_res = client.exec_chat(&config.model, req, Some(&chat_options)).await
        .map_err(|e| format!("调用失败: {e}"))?;

    chat_res.first_text().ok_or_else(|| "模型返回为空".to_string())
}

pub async fn generate_turn_json_stream(
    config: &ModelProviderConfig,
    system_prompt: &str,
    context_prompt: &str,
) -> Result<impl Stream<Item = Result<TurnJsonStreamPiece, String>> + '_ , String> {
    let client = build_client(config)?;
    let chat_options = build_turn_chat_options(config);
    let req = ChatRequest::new(vec![system_prompt.into(), context_prompt.into()]);

    let mut stream = client.exec_chat_stream(&config.model, req, Some(&chat_options)).await
        .map_err(|e| format!("流式调用失败: {e}"))?;

    Ok(async_stream::stream! {
        while let Some(item) = stream.next().await {
            match item {
                Ok(ChatStreamEvent::Content { text, .. }) => {
                    if !text.is_empty() { yield Ok(TurnJsonStreamPiece::Content(text)); }
                }
                Ok(ChatStreamEvent::ReasoningContent { text, .. }) => {
                    if !text.is_empty() { yield Ok(TurnJsonStreamPiece::Reasoning(text)); }
                }
                Ok(ChatStreamEvent::Done) => break,
                Err(e) => { yield Err(format!("流式响应错误: {e}")); break; }
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
        .with_max_tokens(configured_max_tokens(config, WORLD_CARD_MAX_TOKENS, WORLD_CARD_MAX_TOKENS));

    let req = ChatRequest::new(vec![system_prompt.into(), user_prompt.into()]);
    let chat_res = client.exec_chat(&config.model, req, Some(&options)).await
        .map_err(|e| format!("生成世界卡失败: {e}"))?;

    chat_res.first_text().ok_or_else(|| "模型返回为空".to_string())
}

pub async fn generate_world_card_json_stream(
    config: &ModelProviderConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<impl Stream<Item = Result<String, String>> + '_ , String> {
    let client = build_client(config)?;
    let options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(configured_max_tokens(config, WORLD_CARD_MAX_TOKENS, WORLD_CARD_MAX_TOKENS))
        .with_capture_reasoning_content(true);

    let req = ChatRequest::new(vec![system_prompt.into(), user_prompt.into()]);
    let mut stream = client.exec_chat_stream(&config.model, req, Some(&options)).await
        .map_err(|e| format!("生成世界卡失败: {e}"))?;

    Ok(async_stream::stream! {
        while let Some(item) = stream.next().await {
            match item {
                Ok(ChatStreamEvent::Content { text, .. }) => yield Ok(text),
                Ok(ChatStreamEvent::ReasoningContent { text, .. }) => eprint!("[reasoning] {}", text),
                Ok(ChatStreamEvent::Done) => break,
                Err(e) => { yield Err(format!("流式响应错误: {e}")); break; }
                _ => {}
            }
        }
    })
}
