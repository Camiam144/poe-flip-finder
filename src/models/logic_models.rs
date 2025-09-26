use std::{convert::Infallible, fmt, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TradingCurrencyType {
    Exalt,
    Chaos,
    Divine,
    Other,
}

impl FromStr for TradingCurrencyType {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Exalted Orb" => TradingCurrencyType::Exalt,
            "Chaos Orb" => TradingCurrencyType::Chaos,
            "Divine Orb" => TradingCurrencyType::Divine,
            _ => TradingCurrencyType::Other,
        })
    }
}
impl fmt::Display for TradingCurrencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TradingCurrencyType::Exalt => write!(f, "Exalt"),
            TradingCurrencyType::Chaos => write!(f, "Chaos"),
            TradingCurrencyType::Divine => write!(f, "Divine"),
            TradingCurrencyType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Default)]
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
        assert_eq!(orb.unwrap(), TradingCurrencyType::Exalt)
    }
    #[test]
    fn test_parse_divine() {
        let orb = TradingCurrencyType::from_str("Divine Orb");
        assert_eq!(orb.unwrap(), TradingCurrencyType::Divine)
    }
    #[test]
    fn test_parse_chaos() {
        let orb = TradingCurrencyType::from_str("Chaos Orb");
        assert_eq!(orb.unwrap(), TradingCurrencyType::Chaos)
    }
    #[test]
    fn test_parse_other() {
        let orb = TradingCurrencyType::from_str("Vaal Orb");
        assert_eq!(orb.unwrap(), TradingCurrencyType::Other)
    }
}
