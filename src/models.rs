use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRecord {
    #[serde(rename = "CurrencyExchangeSnapshotPairId")]
    pub pair_id: u64,
    #[serde(rename = "CurrencyExchangeSnapshotId")]
    pub snapshot_id: u64,
    pub volume: String,
    pub currency_one: CurrencyInfo,
    pub currency_two: CurrencyInfo,
    pub currency_one_data: CurrencyData,
    pub currency_two_data: CurrencyData,
}

impl ExchangeRecord {
    pub fn trading_currency(&self) -> (TradingCurrencyId, TradingCurrencyId) {
        let curr1 = TradingCurrencyId::from_str(&self.currency_one.text).unwrap();
        let curr2 = TradingCurrencyId::from_str(&self.currency_two.text).unwrap();

        (curr1, curr2)
    }

    pub fn is_valid(&self) -> bool {
        let (curr1, curr2) = self.trading_currency();
        (curr1 != TradingCurrencyId::Other && curr2 == TradingCurrencyId::Other)
            || (curr1 == TradingCurrencyId::Other && curr2 != TradingCurrencyId::Other)
    }
}

#[derive(Debug, Deserialize, Default)]
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

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CurrencyData {
    pub highest_stock: u64,
    pub relative_price: String,
    pub stock_value: String,
    pub value_traded: String,
    pub volume_traded: u64,
}

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CurrencyExchangeSnapshot {
    epoch: u64,
    market_cap: String,
    volume: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TradingCurrencyId {
    Exalt,
    Chaos,
    Divine,
    Other,
}

impl FromStr for TradingCurrencyId {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Exalted Orb" => TradingCurrencyId::Exalt,
            "Chaos Orb" => TradingCurrencyId::Chaos,
            "Divine Orb" => TradingCurrencyId::Divine,
            _ => TradingCurrencyId::Other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exalt() {
        let orb = TradingCurrencyId::from_str("Exalted Orb");
        assert_eq!(orb.unwrap(), TradingCurrencyId::Exalt)
    }
    #[test]
    fn test_parse_divine() {
        let orb = TradingCurrencyId::from_str("Divine Orb");
        assert_eq!(orb.unwrap(), TradingCurrencyId::Divine)
    }
    #[test]
    fn test_parse_chaos() {
        let orb = TradingCurrencyId::from_str("Chaos Orb");
        assert_eq!(orb.unwrap(), TradingCurrencyId::Chaos)
    }
    #[test]
    fn test_parse_other() {
        let orb = TradingCurrencyId::from_str("Vaal Orb");
        assert_eq!(orb.unwrap(), TradingCurrencyId::Other)
    }
    #[test]
    fn test_is_valid_curr1_curr2_other() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_one.text = "Exalted Orb".to_string();
        exchange.currency_two.text = "Vaal Orb".to_string();
        assert!(exchange.is_valid())
    }
    #[test]
    fn test_is_valid_curr1_other_curr2() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_two.text = "Exalted Orb".to_string();
        exchange.currency_one.text = "Vaal Orb".to_string();
        assert!(exchange.is_valid())
    }
    #[test]
    fn test_is_valid_curr1_curr2() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_two.text = "Exalted Orb".to_string();
        exchange.currency_one.text = "Divine Orb".to_string();
        assert!(!exchange.is_valid())
    }
    #[test]
    fn test_is_valid_curr1_other_curr2_other() {
        let mut exchange = ExchangeRecord::default();
        exchange.currency_two.text = "Orb of Transmutation".to_string();
        exchange.currency_one.text = "Vaal Orb".to_string();
        assert!(!exchange.is_valid())
    }
}
