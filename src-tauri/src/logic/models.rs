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

impl TradingCurrencyRates {
    pub fn get_comp_rate(
        &self,
        start_curr: &TradingCurrencyType,
        end_curr: &TradingCurrencyType,
    ) -> Option<f64> {
        match (start_curr, end_curr) {
            (TradingCurrencyType::Chaos, TradingCurrencyType::Exalt) => Some(self.exalt_per_chaos),
            (TradingCurrencyType::Exalt, TradingCurrencyType::Chaos) => {
                Some(1.0 / self.exalt_per_chaos)
            }
            (TradingCurrencyType::Divine, TradingCurrencyType::Exalt) => Some(self.exalt_per_div),
            (TradingCurrencyType::Exalt, TradingCurrencyType::Divine) => {
                Some(1.0 / self.exalt_per_div)
            }
            (TradingCurrencyType::Divine, TradingCurrencyType::Chaos) => Some(self.chaos_per_div),
            (TradingCurrencyType::Chaos, TradingCurrencyType::Divine) => {
                Some(1.0 / self.chaos_per_div)
            }
            (_, _) => None,
        }
    }
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

impl ArbitrageOpportunity {
    pub fn start(&self) -> Option<&TradingCurrencyType> {
        self.path.first()
    }

    pub fn end(&self) -> Option<&TradingCurrencyType> {
        self.path.last()
    }

    pub fn effective_rate_highest(&self) -> f64 {
        self.high_ratios.iter().product()
    }

    pub fn effective_rate_lowest(&self) -> f64 {
        self.low_ratios.iter().product()
    }

    pub fn min_volume(&self) -> i64 {
        self.volumes.iter().min().copied().unwrap_or(0)
    }

    /// Get the ROI of the initial investment expressed as a fraction from zero.
    /// e.g. an ROI of 0.0 means no difference than using a direct exchange between
    /// currencies, an ROI of 1.0 means you double your input, and an ROI of -0.5
    /// means you lose half of what you put in (don't trade those!).
    pub fn roi(&self, rates: &TradingCurrencyRates) -> Option<f64> {
        let direct_rate = rates.get_comp_rate(self.start()?, self.end()?)?;
        // Use this for now, but we might need to also look at lowest or hi/lo
        let arb_rate = 1.0 / self.effective_rate_highest();

        if !arb_rate.is_finite() {
            None
        } else {
            Some(arb_rate / direct_rate - 1.0)
        }
    }

    /// Determine if a given arbitrage opportunity is profitable
    /// An opportunity is profitable if the path A -> X -> B -> A is more profitable
    /// than just A -> B -> A and no step has zero volume.
    /// Right now we only check if taking is profitable under the assumption that
    /// there will be enough inefficiences that trying to eek out market making
    /// profit is not worth our playtime.
    pub fn is_profitable(&self, rates: &TradingCurrencyRates) -> bool {
        self.roi(rates).is_some_and(|roi| roi > 0.0)
    }

    /// If the path is profitable, calculate the profit in terms of the ending
    /// currency. Again, we use the high_ratios here but could be different
    pub fn get_profit(&self, rates: &TradingCurrencyRates) -> Option<f64> {
        let revenue = self.effective_rate_highest();
        let mult = match self.high_ratios.first() {
            Some(val) if *val > 1.0 => *val,
            Some(_) => 1.0,
            None => return None,
        };
        let direct_rate = rates.get_comp_rate(self.start()?, self.end()?)?;

        Some(mult * (1.0 / revenue - direct_rate))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityDisplay {
    pub path: Vec<TradingCurrencyType>,
    pub high_ratios: Vec<f64>,
    pub low_ratios: Vec<f64>,
    pub min_volume: i64,
    pub roi: f64,
    pub profit: f64,
}

impl OpportunityDisplay {
    pub fn from_opportunity(
        opp: &ArbitrageOpportunity,
        rates: &TradingCurrencyRates,
    ) -> Option<Self> {
        Some(Self {
            path: opp.path.to_vec(),
            high_ratios: opp.high_ratios.to_vec(),
            low_ratios: opp.low_ratios.to_vec(),
            min_volume: opp.min_volume(),
            roi: opp.roi(rates)?,
            profit: opp.get_profit(rates)?,
        })
    }
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
