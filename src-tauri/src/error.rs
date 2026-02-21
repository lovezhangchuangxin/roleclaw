use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationFailed,
    NotFound,
    StorageError,
    ProviderError,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::ValidationFailed,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::NotFound,
            message: message.into(),
        }
    }

    pub fn storage(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::StorageError,
            message: message.into(),
        }
    }

    pub fn provider(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::ProviderError,
            message: message.into(),
        }
    }
}
