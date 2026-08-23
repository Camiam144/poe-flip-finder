use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FrontendError {
    Network { message: String },
    Parse { message: String },
    Api { code: i32 },
    Database { message: String },
    InvalidInput { message: String },
    Other { message: String },
}

impl From<GGGApiError> for FrontendError {
    fn from(value: GGGApiError) -> Self {
        match value {
            GGGApiError::Network(v) => FrontendError::Network {
                message: v.to_string(),
            },
            GGGApiError::Parse(v) => FrontendError::Parse {
                message: v.to_string(),
            },
            GGGApiError::Api { code } => FrontendError::Api {
                code: code.into(),
                // message: code.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[repr(i32)]
pub enum GGGErrorCode {
    Accepted = 0,
    ResourceNotFound = 1,
    InvalidQuery = 2,
    RateLimitExceeded = 3,
    InternalError = 4,
    UnexpectedContentType = 5,
    Forbidden = 6,
    TemporarilyUnavailable = 7,
    Unauthorized = 8,
    MethodNotAllowed = 9,
    UnprocessableEntity = 10,
    UnknownValue = 999,
}
impl fmt::Display for GGGErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Accepted => "Accepted",
            Self::ResourceNotFound => "Resource Not Found",
            Self::InvalidQuery => "Invalid Query",
            Self::RateLimitExceeded => "Rate Limit Exceeded",
            Self::InternalError => "Internal Error",
            Self::UnexpectedContentType => "Unexpected Content Type",
            Self::Forbidden => "Forbidden",
            Self::TemporarilyUnavailable => "Temporarily Unavailable",
            Self::Unauthorized => "Unauthorized",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::UnprocessableEntity => "Unprocessable Entity",
            Self::UnknownValue => "Unknown Value Received",
        };
        write!(f, "{}", message)
    }
}

impl From<GGGErrorCode> for i32 {
    fn from(value: GGGErrorCode) -> Self {
        match value {
            GGGErrorCode::Accepted => 0,
            GGGErrorCode::ResourceNotFound => 1,
            GGGErrorCode::InvalidQuery => 2,
            GGGErrorCode::RateLimitExceeded => 3,
            GGGErrorCode::InternalError => 4,
            GGGErrorCode::UnexpectedContentType => 5,
            GGGErrorCode::Forbidden => 6,
            GGGErrorCode::TemporarilyUnavailable => 7,
            GGGErrorCode::Unauthorized => 8,
            GGGErrorCode::MethodNotAllowed => 9,
            GGGErrorCode::UnprocessableEntity => 10,
            GGGErrorCode::UnknownValue => 999,
        }
    }
}

impl From<i32> for GGGErrorCode {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Accepted,
            1 => Self::ResourceNotFound,
            2 => Self::InvalidQuery,
            3 => Self::RateLimitExceeded,
            4 => Self::InternalError,
            5 => Self::UnexpectedContentType,
            6 => Self::Forbidden,
            7 => Self::TemporarilyUnavailable,
            8 => Self::Unauthorized,
            9 => Self::MethodNotAllowed,
            10 => Self::UnprocessableEntity,
            _ => Self::UnknownValue,
        }
    }
}

// TODO: Add actual returned messages from GGG errors instead of writing them in.
#[derive(Debug, thiserror::Error)]
pub enum GGGApiError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("GGG API Error {code}")]
    Api { code: GGGErrorCode },
}

#[derive(Debug, Deserialize)]
pub struct GGGErrorBody {
    pub code: GGGErrorCode,
    pub message: String,
}
#[derive(Debug, Deserialize)]
pub struct GGGWrappedError {
    pub error: GGGErrorBody,
}
