//! This module contains stuff for working with GGG's API
pub mod client;
pub mod models;
use std::{fmt, str::FromStr};

use client::ApiClient;
use serde::{Deserialize, Serialize};

use crate::auth::AuthorizedScopes;
use crate::AppState;
use models::{ApiError, GGGWrappedError, RawCxApiResponse, RawLeagueApiResponse};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum Realm {
    Poe1,
    Poe2,
}
impl FromStr for Realm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "poe1" | "Poe1" | "POE1" => Ok(Realm::Poe1),
            "poe2" | "Poe2" | "POE2" => Ok(Realm::Poe2),
            _ => Err("Invalid Realm (only poe1 and poe2 accepted)"),
        }
    }
}
impl fmt::Display for Realm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poe1 => write!(f, "poe1"),
            Self::Poe2 => write!(f, "poe2"),
        }
    }
}

/// Make a call to GGG's CXAPI to get an entry
pub async fn get_specified_cxapi_from_ggg(
    client: &ApiClient,
    realm: Realm,
    time: i64,
) -> anyhow::Result<RawCxApiResponse> {
    let base_url = "https://api.pathofexile.com/currency-exchange/";

    // For some reason if the realm is "poe1" you need to not include it,
    // for every other realm you need to include it
    let url = match realm {
        Realm::Poe1 => format!("{}{}", base_url, time),
        _ => format!("{}{}/{}", base_url, realm.to_string().to_lowercase(), time),
    };
    dbg!(format!("url: {}", url));

    let raw_response = client.get_url(&url, AuthorizedScopes::Cxapi).await?;

    // I need to do some error checking to make sure I'm not storing invalid responses
    // What do I need to check? Http code and that we have a non-empty body?
    // TODO: Validation
    let status = raw_response.status();
    let head = raw_response.headers();
    dbg!(status);
    dbg!(head);

    // In order to get both the raw text and the json we have to read the entire
    // stream to a string (consuming the stream), save the string, and then
    // use serde to deserialize the string to the rust object we want.
    let text_response = raw_response.text().await?;
    let serialized = serde_json::from_str::<RawCxApiResponse>(&text_response)?;

    Ok(serialized)
}

/// Get all current leagues from GGG's API. This should be cached and
/// updated as needed instead of pinging it every time.
pub async fn get_leagues_from_ggg(
    state: &AppState,
    realm: Realm,
) -> Result<RawLeagueApiResponse, ApiError> {
    // TODO: Cache these somewhere, check once per day? Once per session?
    let url = "https://api.pathofexile.com/league";
    let realm = if realm == Realm::Poe1 {
        "pc"
    } else {
        &realm.to_string().to_lowercase()
    };
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params).expect("Malformed leagues url.");

    let response = state
        .http_client
        .get_url(url.as_str(), AuthorizedScopes::Leagues)
        .await?;
    // dbg!(&response);

    let status = response.status();
    let bytes = response.bytes().await?;

    if status.is_success() {
        match serde_json::from_slice::<RawLeagueApiResponse>(&bytes) {
            Ok(data) => std::result::Result::Ok(data),
            Err(parse_error) => {
                // Wrapped error with success status code
                if let Ok(wrap) = serde_json::from_slice::<GGGWrappedError>(&bytes) {
                    Err(ApiError::Api {
                        code: wrap.error.code,
                        message: wrap.error.message,
                    })
                } else {
                    Err(ApiError::Parse(parse_error))
                }
            }
        }
    } else {
        match serde_json::from_slice::<GGGWrappedError>(&bytes) {
            Ok(wrapped_error) => Err(ApiError::Api {
                code: wrapped_error.error.code,
                message: wrapped_error.error.message,
            }),
            Err(_) => Err(ApiError::Api {
                code: (status.as_u16() as i32).into(),
                message: "Bad status and unparseable body.".to_string(),
            }),
        }
    }
}
