use reqwest::{Result, blocking::Client};

use crate::models::api_models::{ExchangeRecord, ExchangeSnapshot};

pub fn get_exchange_snapshot(client: &Client) -> Result<ExchangeSnapshot> {
    let url = "https://poe2scout.com/api/currencyExchangeSnapshot?league=Rise%20of%20the%20Abyssal";

    let response = client.get(url).send()?;
    response.error_for_status_ref()?;
    response.json()
}

pub fn get_newest_snapshot_pairs(client: &Client) -> Result<Vec<ExchangeRecord>> {
    let url =
        "https://poe2scout.com/api/currencyExchange/SnapshotPairs?league=Rise%20of%20the%20Abyssal";
    client.get(url).send()?.json()
}
