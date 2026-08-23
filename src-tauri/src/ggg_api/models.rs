use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum Realm {
    Poe1,
    Poe2,
}
impl FromStr for Realm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "poe1" | "Poe1" | "POE1" => Ok(Realm::Poe1),
            "poe2" | "Poe2" | "POE2" => Ok(Realm::Poe2),
            _ => Err("Invalid Realm (only poe1 and poe2 accepted)".to_string()),
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

#[derive(Debug, Deserialize, Serialize)]
pub struct RawMarket {
    pub league: String,
    pub market_id: String, // This is pipe separated chaos|divine
    pub market_pair: Vec<String>,

    pub volume_traded: HashMap<String, u64>,
    pub highest_ratio: HashMap<String, u64>,
    pub highest_stock: HashMap<String, u64>,
    pub lowest_ratio: HashMap<String, u64>,
    pub lowest_stock: HashMap<String, u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawCxApiResponse {
    pub next_change_id: u64,
    pub markets: Vec<RawMarket>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RawLeagueApiResponse {
    pub leagues: Vec<GGGLeague>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GGGLeague {
    pub id: String,
    pub name: Option<String>,
    pub realm: Option<String>,
    pub url: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub description: Option<String>,
    pub category: Option<Category>,
    pub event: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub current: Option<bool>,
}

impl GGGLeague {
    /// A league is active if category exists and category.current is true and
    /// event does not exist
    pub fn is_active(&self) -> bool {
        self.category
            .as_ref()
            .is_some_and(|c| c.current.is_some_and(|cur| cur))
            && self.event.is_none()
    }
}
