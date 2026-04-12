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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrencyPairValues {
    pub ca: (TradingCurrencyType, u64),
    pub cb: (TradingCurrencyType, u64),
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

/// This holds the bid/ask spread for a given market.
// #[derive(Debug)]
// pub struct BidAskSpread {
//     item_1: TradingCurrencyType,
//     item_2: TradingCurrencyType,
//     bid: CurrencyPairValues,
//     ask: CurrencyPairValues,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub league: String,
    pub curr_a: TradingCurrencyType,
    pub curr_b: TradingCurrencyType,

    pub volume_traded: CurrencyPairValues,
    pub highest_ratio: CurrencyPairValues,
    pub highest_stock: CurrencyPairValues,
    pub lowest_ratio: CurrencyPairValues,
    pub lowest_stock: CurrencyPairValues,
}
impl Market {
    pub fn try_from_raw(raw: RawMarket) -> Result<Self> {
        let (a, b) = raw
            .market_id
            .split_once('|')
            .context("Couldn't split market on vertical bar")?;

        Ok(Self {
            curr_a: TradingCurrencyType::from_str(a)?,
            curr_b: TradingCurrencyType::from_str(b)?,

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
            ca: (TradingCurrencyType::from_str(curr_a)?, *val_a),
            cb: (TradingCurrencyType::from_str(curr_b)?, *val_b),
        })
    }

    /// Get normalized bids and asks so we can get the correct ratios.
    pub fn get_normed_bid(&self) -> Option<f64> {
        let (stronger, weaker) = if self.curr_a < self.curr_b {
            (&self.lowest_ratio.cb, &self.lowest_ratio.ca)
        } else {
            (&self.lowest_ratio.ca, &self.lowest_ratio.cb)
        };
        let normed = stronger.1 as f64 / weaker.1 as f64;

        if normed.is_infinite() || normed.is_nan() {
            return None;
        }
        Some(normed)
    }

    pub fn get_normed_ask(&self) -> Option<f64> {
        let (stronger, weaker) = if self.curr_a < self.curr_b {
            (&self.highest_ratio.cb, &self.highest_ratio.ca)
        } else {
            (&self.highest_ratio.ca, &self.highest_ratio.cb)
        };
        let normed = stronger.1 as f64 / weaker.1 as f64;

        if normed.is_infinite() || normed.is_nan() {
            return None;
        }
        Some(normed)
    }

    /// Get the bid ask spread.
    /// Currencies are ranked from weakest to strongest: Anything, Ex, Chaos, Div.
    /// The Bid Ask Spread is presented as weaker per stronger (e.g. items per div)
    pub fn get_spread(&self) -> Option<(f64, TradingCurrencyType)> {
        let norm_bid = self.get_normed_bid()?;
        let norm_ask = self.get_normed_ask()?;

        Some((norm_ask - norm_bid, self.curr_a.clone()))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGGMarket {
    pub next_change_id: u64,
    pub markets: Vec<Market>,
}

impl GGGMarket {
    pub fn try_from_raw_cxapi_response(response: RawCxApiResponse) -> Result<Self> {
        let parsed_markets: Vec<Market> = response
            .markets
            .into_iter()
            .map(|m| Market::try_from_raw(m).expect("Should have been able to parse raw market"))
            .collect();
        Ok(GGGMarket {
            next_change_id: response.next_change_id,
            markets: parsed_markets,
        })
    }
    pub fn filter_league(&self, league: &str) -> Vec<&Market> {
        self.markets
            .iter()
            .filter(|market| market.league == league)
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GGGLeagueList {
    pub leagues: Vec<GGGLeague>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GGGLeague {
    pub id: String,
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
