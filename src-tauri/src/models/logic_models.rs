use std::{convert::Infallible, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum TradingCurrencyType {
    Other(String),
    Exalt,
    Chaos,
    Divine,
}

impl FromStr for TradingCurrencyType {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Exalted Orb" | "exalted" | "exalt" => TradingCurrencyType::Exalt,
            "Chaos Orb" | "chaos" => TradingCurrencyType::Chaos,
            "Divine Orb" | "divine" => TradingCurrencyType::Divine,
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

#[derive(Debug, Default, Serialize)]
pub struct TradingCurrencyRates {
    pub div_to_exalt: f64,
    pub div_to_chaos: f64,
    pub chaos_to_exalt: f64,
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
