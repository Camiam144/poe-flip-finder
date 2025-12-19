use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Ok, Result};
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
                TradingCurrencyType::Other,
            ) => Some((
                hub,
                self.currency_one_data.relative_price,
                self.currency_two.text.clone(),
                self.currency_two_data.relative_price,
            )),
            // bridge -> hub
            (
                TradingCurrencyType::Other,
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

// #[allow(dead_code)]
// #[derive(Debug)]
// pub struct ExchangeQueryResult {
//     pub ts: u64,
//     pub pair_id: u64,
//     pub snapshot_id: u64,
//     pub from_currency: String,
//     pub to_currency: String,
//     pub from_relative_price: f64,
//     pub to_relative_price: f64,
//     pub volume: f64,
// }

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyPairValues {
    pub c1: (String, u64),
    pub c2: (String, u64),
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
    pub curr_a: String,
    pub curr_b: String,

    pub volume_traded: CurrencyPairValues,
    pub highest_ratio: CurrencyPairValues,
    pub highest_stock: CurrencyPairValues,
    pub lowest_ratio: CurrencyPairValues,
    pub lowest_stock: CurrencyPairValues,
}
impl Market {
    pub fn from_raw(raw: RawMarket) -> Result<Self> {
        let (a, b) = raw
            .market_id
            .split_once('|')
            .context("Couldn't split market on vertical bar")?;

        Ok(Self {
            curr_a: a.to_string(),
            curr_b: b.to_string(),

            league: raw.league,
            volume_traded: Market::pair_from_map(&raw.volume_traded, a, b)?,
            highest_ratio: Market::pair_from_map(&raw.highest_ratio, a, b)?,
            highest_stock: Market::pair_from_map(&raw.highest_stock, a, b)?,
            lowest_ratio: Market::pair_from_map(&raw.lowest_ratio, a, b)?,
            lowest_stock: Market::pair_from_map(&raw.lowest_stock, a, b)?,
        })
    }
    fn pair_from_map(
        map: &HashMap<String, u64>,
        curr_a: &str,
        curr_b: &str,
    ) -> Result<CurrencyPairValues> {
        let val_a = map.get(curr_a).context("Couldn't find currency a in map")?;
        let val_b = map.get(curr_b).context("Couldn't find currency b in map")?;

        Ok(CurrencyPairValues {
            c1: (curr_a.to_string(), *val_a),
            c2: (curr_b.to_string(), *val_b),
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGGMarket {
    pub next_change_id: u64,
    pub markets: Vec<Market>,
}

#[derive(Debug, Deserialize)]
pub struct RawCxApiResponse {
    pub next_change_id: u64,
    pub markets: Vec<RawMarket>,
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
