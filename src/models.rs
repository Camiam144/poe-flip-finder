use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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
