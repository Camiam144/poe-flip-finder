use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de};
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
// Can find multiple examples online, I chose this because I want the program
// to panic if it can't format the string. No unwrapping options in my struct!
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
        (curr1 != TradingCurrencyType::Other && curr2 == TradingCurrencyType::Other)
            || (curr1 == TradingCurrencyType::Other && curr2 != TradingCurrencyType::Other)
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExchangeQueryResult {
    pub ts: u64,
    pub pair_id: u64,
    pub snapshot_id: u64,
    pub from_currency: String,
    pub to_currency: String,
    pub from_relative_price: f64,
    pub to_relative_price: f64,
    pub volume: f64,
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
