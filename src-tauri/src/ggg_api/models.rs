use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct RawMarket {
    pub league: String,
    pub market_id: String, // This is pipe separated chaos|divine
    pub market_pair: Vec<String>,

    pub volume_traded: HashMap<String, u64>,
    pub highest_ratio: HashMap<String, u64>,
    pub highest_stock: HashMap<String, u64>,
    pub lowest_ratio: HashMap<String, u64>,
    pub lowest_stock: HashMap<String, u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawCxApiResponse {
    pub next_change_id: u64,
    pub markets: Vec<RawMarket>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawLeagueApiResponse {
    pub leagues: Vec<GGGLeague>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GGGLeague {
    pub id: String,
    pub name: Option<String>,
    pub realm: Option<String>,
    pub url: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub description: Option<String>,
    pub category: Option<Category>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub current: Option<bool>,
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
pub enum ApiError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("GGG API Error {code}: {message}")]
    Api { code: GGGErrorCode, message: String },
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
