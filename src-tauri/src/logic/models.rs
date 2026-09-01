use std::{collections::HashMap, convert::Infallible, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::db::models::ParsedDbRow;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(tag = "type", content = "name")]
pub enum TradingCurrencyType {
    Other(String),
    Exalt,
    Chaos,
    Divine,
}

impl FromStr for TradingCurrencyType {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "exalted orb" | "exalted" | "exalt" => TradingCurrencyType::Exalt,
            "chaos orb" | "chaos" => TradingCurrencyType::Chaos,
            "divine orb" | "divine" => TradingCurrencyType::Divine,
            _ => TradingCurrencyType::Other(String::from(s)),
        })
    }
}
impl fmt::Display for TradingCurrencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TradingCurrencyType::Exalt => write!(f, "Exalt"),
            TradingCurrencyType::Chaos => write!(f, "Chaos"),
            TradingCurrencyType::Divine => write!(f, "Divine"),
            TradingCurrencyType::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

impl TradingCurrencyType {
    pub fn is_hub(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

#[derive(Debug, Default, Serialize)]
pub struct TradingCurrencyRates {
    pub exalt_per_div: f64,
    pub chaos_per_div: f64,
    pub exalt_per_chaos: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Market {
    pub change_id: i64,
    pub league: String,
    pub currency_a: TradingCurrencyType,
    pub currency_b: TradingCurrencyType,
    pub volume_traded_currency_a: i64,
    pub volume_traded_currency_b: i64,
    pub lowest_stock_currency_a: i64,
    pub lowest_stock_currency_b: i64,
    pub highest_stock_currency_a: i64,
    pub highest_stock_currency_b: i64,
    pub lowest_ratio_currency_a: i64,
    pub lowest_ratio_currency_b: i64,
    pub highest_ratio_currency_a: i64,
    pub highest_ratio_currency_b: i64,
    pub is_hub_curr_a: bool,
    pub is_hub_curr_b: bool,
}
impl From<&ParsedDbRow> for Market {
    fn from(value: &ParsedDbRow) -> Self {
        // Safety: TradingCurrencyType conversion is infallable
        let currency_a = TradingCurrencyType::from_str(&value.currency_a_name_common).unwrap();
        let currency_b = TradingCurrencyType::from_str(&value.currency_b_name_common).unwrap();
        let is_hub_curr_a = value.is_hub_curr_a != 0;
        let is_hub_curr_b = value.is_hub_curr_b != 0;

        Market {
            change_id: value.change_id,
            league: value.league.clone(),
            currency_a,
            currency_b,
            volume_traded_currency_a: value.volume_traded_currency_a,
            volume_traded_currency_b: value.volume_traded_currency_b,
            lowest_stock_currency_a: value.lowest_stock_currency_a,
            lowest_stock_currency_b: value.lowest_stock_currency_b,
            highest_stock_currency_a: value.highest_stock_currency_a,
            highest_stock_currency_b: value.highest_stock_currency_b,
            lowest_ratio_currency_a: value.lowest_ratio_currency_a,
            lowest_ratio_currency_b: value.lowest_ratio_currency_b,
            highest_ratio_currency_a: value.highest_ratio_currency_a,
            highest_ratio_currency_b: value.highest_ratio_currency_b,
            is_hub_curr_a,
            is_hub_curr_b,
        }
    }
}

impl From<ParsedDbRow> for Market {
    fn from(value: ParsedDbRow) -> Self {
        Market::from(&value)
    }
}

impl Market {
    pub fn is_hub(&self) -> bool {
        self.is_hub_curr_a || self.is_hub_curr_b
    }
    pub fn is_trading_rate(&self) -> bool {
        self.is_hub_curr_a && self.is_hub_curr_b
    }
    pub fn get_lowest_ratio(&self) -> Option<f64> {
        if self.lowest_ratio_currency_b == 0 {
            return None;
        }
        Some(self.lowest_ratio_currency_a as f64 / self.lowest_ratio_currency_b as f64)
    }
    pub fn get_highest_ratio(&self) -> Option<f64> {
        if self.highest_ratio_currency_b == 0 {
            return None;
        }
        Some(self.highest_ratio_currency_a as f64 / self.highest_ratio_currency_b as f64)
    }
}

#[derive(Debug, Clone)]
pub struct TradingEdge {
    pub from_currency: TradingCurrencyType,
    pub to_currency: TradingCurrencyType,
    pub lowest_ratio: f64,
    pub highest_ratio: f64,
    pub volume: i64,
}

pub type Graph = HashMap<TradingCurrencyType, Vec<TradingEdge>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub path: Vec<TradingCurrencyType>,
    pub high_ratios: Vec<f64>,
    pub low_ratios: Vec<f64>,
    pub volumes: Vec<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exalt() {
        let orb = TradingCurrencyType::from_str("Exalted Orb");
        let orb2 = TradingCurrencyType::from_str("exalted");
        assert_eq!(orb.unwrap(), TradingCurrencyType::Exalt);
        assert_eq!(orb2.unwrap(), TradingCurrencyType::Exalt);
    }
    #[test]
    fn test_parse_divine() {
        let orb = TradingCurrencyType::from_str("Divine Orb");
        let orb2 = TradingCurrencyType::from_str("divine");
        assert_eq!(orb.unwrap(), TradingCurrencyType::Divine);
        assert_eq!(orb2.unwrap(), TradingCurrencyType::Divine);
    }
    #[test]
    fn test_parse_chaos() {
        let orb = TradingCurrencyType::from_str("Chaos Orb");
        let orb2 = TradingCurrencyType::from_str("chaos");
        assert_eq!(orb.unwrap(), TradingCurrencyType::Chaos);
        assert_eq!(orb2.unwrap(), TradingCurrencyType::Chaos);
    }
    #[test]
    fn test_parse_other() {
        let orb = TradingCurrencyType::from_str("Vaal Orb");
        assert_eq!(
            orb.unwrap(),
            TradingCurrencyType::Other("Vaal Orb".to_string())
        )
    }
    #[test]
    fn test_comparison() {
        assert!(TradingCurrencyType::Divine > TradingCurrencyType::Chaos);
        assert!(TradingCurrencyType::Chaos > TradingCurrencyType::Exalt);
        assert!(
            TradingCurrencyType::Exalt > TradingCurrencyType::Other("anything-here".to_string())
        )
    }
}
