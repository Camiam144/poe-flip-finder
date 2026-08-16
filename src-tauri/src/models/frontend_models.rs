use serde::Serialize;

use crate::ggg_api::models::ApiError;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FrontendError {
    Network { message: String },
    Parse { message: String },
    Api { code: i32, message: String },
}

impl From<ApiError> for FrontendError {
    fn from(value: ApiError) -> Self {
        match value {
            ApiError::Network(v) => FrontendError::Network {
                message: v.to_string(),
            },
            ApiError::Parse(v) => FrontendError::Parse {
                message: v.to_string(),
            },
            ApiError::Api { code, message } => FrontendError::Api {
                code: code.into(),
                message,
            },
        }
    }
}
