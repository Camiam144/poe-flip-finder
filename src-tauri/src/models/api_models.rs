use std::{collections::HashMap, fmt, str::FromStr};

use anyhow::{Context, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::models::logic_models::TradingCurrencyType;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRecord {
    #[serde(rename = "CurrencyExchangeSnapshotPairId")]
    pub pair_id: u64,
    #[serde(rename = "CurrencyExchangeSnapshotId")]
    pub snapshot_id: u64,
    #[serde(default, deserialize_with = "str_as_f64")]
    pub volume: f64,
    pub currency_one: CurrencyInfo,
    pub currency_two: CurrencyInfo,
    pub currency_one_data: CurrencyData,
    pub currency_two_data: CurrencyData,
}

// Need this to deserialize string floats into a f64. Ripped from docs/reddit.
fn str_as_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    let val = Value::deserialize(deserializer)
        .map_err(|e| de::Error::custom(format!("Couldn't deserialize value: {}", e)))?;

    match val {
        Value::String(s) => s
            .parse::<f64>()
            .map_err(|e| de::Error::custom(format!("Got bad float {}: {e}", s))),
        Value::Number(n) => n.as_f64().ok_or(de::Error::custom(format!(
            "Couldn't convert number to f64: {n}"
        ))),
        other => Err(de::Error::custom(format!(
            "Expected parsable string, got {other:?}"
        ))),
    }
}

impl ExchangeRecord {
    pub fn trading_currency(&self) -> (TradingCurrencyType, TradingCurrencyType) {
        let curr1 = TradingCurrencyType::from_str(&self.currency_one.text).unwrap();
        let curr2 = TradingCurrencyType::from_str(&self.currency_two.text).unwrap();

        (curr1, curr2)
    }

    pub fn is_valid_bridge(&self) -> bool {
        let (curr1, curr2) = self.trading_currency();
        (!matches!(curr1, TradingCurrencyType::Other(_))
            && matches!(curr2, TradingCurrencyType::Other(_)))
            || (matches!(curr1, TradingCurrencyType::Other(_))
                && !matches!(curr2, TradingCurrencyType::Other(_)))
    }

    pub fn hub_bridge_price(&self) -> Option<(TradingCurrencyType, f64, String, f64)> {
        // Get the price of the hub -> bridge or bridge -> hub in a manner
        // that is easier to work with
        let (c1, c2) = self.trading_currency();

        match (c1, c2) {
            // hub -> bridge
            (
                hub @ TradingCurrencyType::Exalt
                | hub @ TradingCurrencyType::Chaos
                | hub @ TradingCurrencyType::Divine,
                TradingCurrencyType::Other(_),
            ) => Some((
                hub,
                self.currency_one_data.relative_price,
                self.currency_two.text.clone(),
                self.currency_two_data.relative_price,
            )),
            // bridge -> hub
            (
                TradingCurrencyType::Other(_),
                hub @ TradingCurrencyType::Exalt
                | hub @ TradingCurrencyType::Chaos
                | hub @ TradingCurrencyType::Divine,
            ) => Some((
                hub,
                self.currency_two_data.relative_price,
                self.currency_one.text.clone(),
                self.currency_one_data.relative_price,
            )),
            _ => None,
        }
    }
}

// #[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyInfo {
    pub id: u64,
    pub item_id: u64,
    pub currency_category_id: u64,
    pub api_id: String,
    pub text: String,
    pub category_api_id: String,
    pub icon_url: String,
}

// #[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CurrencyData {
    pub highest_stock: u64,
    #[serde(default, deserialize_with = "str_as_f64")]
    pub relative_price: f64,
    #[serde(default, deserialize_with = "str_as_f64")]
    pub stock_value: f64,
    #[serde(default, deserialize_with = "str_as_f64")]
    pub value_traded: f64,
    pub volume_traded: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeSnapshot {
    pub epoch: u64,
    #[serde(default, deserialize_with = "str_as_f64")]
    pub market_cap: f64,
    #[serde(default, deserialize_with = "str_as_f64")]
    pub volume: f64,
}

// These are the models for the offical GGG api
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrencyPairValues {
    pub ca: (TradingCurrencyType, u64),
    pub cb: (TradingCurrencyType, u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HubBridgeDir {
    HubToBridge,
    BridgeToHub,
}

#[derive(Debug, Deserialize)]
pub struct RawMarket {
    pub league: String,
    pub market_id: String, // This is pipe separated chaos|divine

    pub volume_traded: HashMap<String, u64>,
    pub highest_ratio: HashMap<String, u64>,
    pub highest_stock: HashMap<String, u64>,
    pub lowest_ratio: HashMap<String, u64>,
    pub lowest_stock: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub league: String,
    pub curr_a: TradingCurrencyType,
    pub curr_b: TradingCurrencyType,
    pub stronger: u8,

    pub volume_traded: CurrencyPairValues,
    pub highest_ratio: CurrencyPairValues,
    pub highest_stock: CurrencyPairValues,
    pub lowest_ratio: CurrencyPairValues,
    pub lowest_stock: CurrencyPairValues,
}
impl TryFrom<RawMarket> for Market {
    type Error = anyhow::Error;

    fn try_from(value: RawMarket) -> std::result::Result<Self, Self::Error> {
        let (a, b) = value
            .market_id
            .split_once('|')
            .context("Couldn't split market on vertical bar")?;

        let curr_a = TradingCurrencyType::from_str(a)?;
        let curr_b = TradingCurrencyType::from_str(b)?;

        let volume_traded = Market::try_pair_from_map(&value.volume_traded, a, b)?;

        // TODO: Might want some more logic here, prefer trading currency if a tie?
        let stronger = if volume_traded.ca.1 <= volume_traded.cb.1 {
            0
        } else {
            1
        };

        Ok(Self {
            curr_a,
            curr_b,
            stronger,

            league: value.league,
            volume_traded,
            highest_ratio: Market::try_pair_from_map(&value.highest_ratio, a, b)?,
            highest_stock: Market::try_pair_from_map(&value.highest_stock, a, b)?,
            lowest_ratio: Market::try_pair_from_map(&value.lowest_ratio, a, b)?,
            lowest_stock: Market::try_pair_from_map(&value.lowest_stock, a, b)?,
        })
    }
}
impl Market {
    fn try_pair_from_map(
        map: &HashMap<String, u64>,
        curr_a: &str,
        curr_b: &str,
    ) -> Result<CurrencyPairValues> {
        let val_a = map.get(curr_a).context("Couldn't find currency a in map")?;
        let val_b = map.get(curr_b).context("Couldn't find currency b in map")?;

        Ok(CurrencyPairValues {
            ca: (TradingCurrencyType::from_str(curr_a)?, *val_a),
            cb: (TradingCurrencyType::from_str(curr_b)?, *val_b),
        })
    }

    /// A market is a valid bridge if and only if exactly one currency is a trading
    /// currency type (Exalt, Chaos, or Divine)
    pub fn is_valid_bridge(&self) -> bool {
        (!matches!(self.curr_a, TradingCurrencyType::Other(_))
            && matches!(self.curr_b, TradingCurrencyType::Other(_)))
            || (matches!(self.curr_a, TradingCurrencyType::Other(_))
                && !matches!(self.curr_b, TradingCurrencyType::Other(_)))
    }

    /// Get normalized bids and asks so we can get the correct ratios
    /// We always want the result returned in "hub currencies per item" so for
    /// example if something is selling for 10 per divine we want the result to
    /// be 0.1 divine. We will ignore any non hub currencies for now.
    pub fn get_normed_bid(&self) -> Option<f64> {
        let (stronger, weaker) = if self.stronger == 1 {
            (&self.lowest_ratio.cb, &self.lowest_ratio.ca)
        } else {
            (&self.lowest_ratio.ca, &self.lowest_ratio.cb)
        };

        let normed = if weaker.1 != 0 {
            stronger.1 as f64 / weaker.1 as f64
        } else {
            return None;
        };

        if normed.is_infinite() || normed.is_nan() {
            return None;
        }
        Some(normed)
    }

    pub fn get_normed_ask(&self) -> Option<f64> {
        let (stronger, weaker) = if self.stronger == 1 {
            (&self.highest_ratio.cb, &self.highest_ratio.ca)
        } else {
            (&self.highest_ratio.ca, &self.highest_ratio.cb)
        };

        let normed = if weaker.1 != 0 {
            stronger.1 as f64 / weaker.1 as f64
        } else {
            return None;
        };

        if normed.is_infinite() || normed.is_nan() {
            return None;
        }
        Some(normed)
    }

    /// Get the bid ask spread.
    /// The "stronger" currency is whichever costs more for that specific market
    /// The Bid Ask Spread is presented as stronger per weaker: e.g. 1 item per 400 div
    /// for expensive items or like 1 div per 12 items for cheaper items
    pub fn get_spread(&self) -> Option<(f64, TradingCurrencyType)> {
        let norm_bid = self.get_normed_bid()?;
        let norm_ask = self.get_normed_ask()?;

        let stronger = if self.stronger == 0 {
            self.curr_a.clone()
        } else {
            self.curr_b.clone()
        };

        Some((norm_ask - norm_bid, stronger))
    }
}

#[derive(Debug, Deserialize)]
pub struct RawCxApiResponse {
    pub next_change_id: u64,
    pub markets: Vec<RawMarket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGGMarket {
    pub next_change_id: u64,
    pub markets: Vec<Market>,
}

impl TryFrom<RawCxApiResponse> for GGGMarket {
    type Error = anyhow::Error;

    fn try_from(response: RawCxApiResponse) -> std::result::Result<Self, Self::Error> {
        let parsed_markets: Vec<Market> = response
            .markets
            .into_iter()
            .filter_map(|m| Market::try_from(m).ok())
            .collect();
        Ok(GGGMarket {
            next_change_id: response.next_change_id,
            markets: parsed_markets,
        })
    }
}

impl GGGMarket {
    pub fn filter_league(&self, league: &str) -> Vec<&Market> {
        self.markets
            .iter()
            .filter(|market| market.league == league)
            .collect()
    }
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
// impl TryFrom<i32> for GGGErrorCode {
//     type Error = &'static str;
//
//     fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
//         match value {
//             0 => Ok(Self::Accepted),
//             1 => Ok(Self::ResourceNotFound),
//             2 => Ok(Self::InvalidQuery),
//             3 => Ok(Self::RateLimitExceeded),
//             4 => Ok(Self::InternalError),
//             5 => Ok(Self::UnexpectedContentType),
//             6 => Ok(Self::Forbidden),
//             7 => Ok(Self::TemporarilyUnavailable),
//             8 => Ok(Self::Unauthorized),
//             9 => Ok(Self::MethodNotAllowed),
//             10 => Ok(Self::UnprocessableEntity),
//             _ => Ok(Self::UnknownValue),
//             // _ => Err("Unknown code received"),
//         }
//     }
// }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_curr1_curr2_other() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_one.text = "Exalted Orb".to_string();
        exchange.currency_two.text = "Vaal Orb".to_string();
        assert!(exchange.is_valid_bridge())
    }
    #[test]
    fn test_is_valid_curr1_other_curr2() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_two.text = "Exalted Orb".to_string();
        exchange.currency_one.text = "Vaal Orb".to_string();
        assert!(exchange.is_valid_bridge())
    }
    #[test]
    fn test_is_valid_curr1_curr2() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_two.text = "Exalted Orb".to_string();
        exchange.currency_one.text = "Divine Orb".to_string();
        assert!(!exchange.is_valid_bridge())
    }
    #[test]
    fn test_is_valid_curr1_other_curr2_other() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_two.text = "Orb of Transmutation".to_string();
        exchange.currency_one.text = "Vaal Orb".to_string();
        assert!(!exchange.is_valid_bridge())
    }
}
