use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ExchangeRecord {
    #[serde(rename = "CurrencyExchangeSnapshotPairId")]
    pub pair_id: u64,
    #[serde(rename = "CurrencyExchangeSnapshotId")]
    pub snapshot_id: u64,
    #[serde(rename = "Volume")]
    pub volume: String,
    #[serde(rename = "CurrencyOne")]
    pub currency_one: CurrencyInfo,
    #[serde(rename = "CurrencyTwo")]
    pub currency_two: CurrencyInfo,
    #[serde(rename = "CurrencyOneData")]
    pub currency_one_data: CurrencyData,
    #[serde(rename = "CurrencyTwoData")]
    pub currency_two_data: CurrencyData,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyInfo {
    #[serde(rename = "id")]
    pub id: u64,
    #[serde(rename = "itemId")]
    pub item_id: u64,
    #[serde(rename = "apiId")]
    pub api_id: String,
    #[serde(rename = "text")]
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyData {
    #[serde(rename = "RelativePrice")]
    pub relative_price: String,
    #[serde(rename = "StockValue")]
    pub stock_value: String,
    #[serde(rename = "VolumeTraded")]
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
