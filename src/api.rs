//! This module contains stuff for working with GGG's API
//! This is not internal API endpoints.
use anyhow::{Context, Ok, Result};
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth;
use crate::auth::AuthorizedScopes;
use crate::db::DbClient;
use crate::models::api_models::{GGGLeague, GGGMarket, RawCxApiResponse};

/// Get the specified cxapi realm and timestamp from the cache if it exists,
/// otherwise grab it from the API
pub async fn get_specified_cxapi(
    client: &Client,
    db_client: &DbClient,
    realm: &str,
    time: u64,
) -> Result<GGGMarket> {
    let market = db_client
        .get_specific_change_id(time.try_into().unwrap(), realm)
        .await?;
    let response = if market.is_some() {
        serde_json::from_str::<RawCxApiResponse>(&market.unwrap().payload)
            .context("Should have been able to parse cached payload")?
    } else {
        // We don't have the specific payload already cached, so we have to get it

        let token = auth::get_api_token(&AuthorizedScopes::Cxapi).await?;
        let base_url = "https://api.pathofexile.com/currency-exchange/";
        let url = format!("{}{}/{}", base_url, realm.to_lowercase(), time);

        let raw_response = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .send()
            .await?;

        raw_response.json().await?
    };

    GGGMarket::try_from_raw_cxapi_response(response)
}

/// Get the entire Cxapi dump from the past hour. The current hour does not yet
/// have information.
pub async fn get_most_recent_cxapi(
    client: &Client,
    db_client: &DbClient,
    realm: &str,
) -> Result<GGGMarket> {
    let past_hour = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600;

    get_specified_cxapi(client, db_client, realm, past_hour).await
}

/// Get all current leagues from GGG's API. This can probably be cached and
/// updated as needed instead of pinging it every time, but for now we will
/// continue to ping ever time
pub async fn get_leagues(client: &Client, realm: &str) -> Result<Vec<GGGLeague>> {
    let token = auth::get_api_token(&AuthorizedScopes::Leagues).await?;
    let url = "https://api.pathofexile.com/league";
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params)?;

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
        .send()
        .await?;

    let result: Vec<GGGLeague> = response.json().await?;

    Ok(result)
}

/// Check if a given snapshot already exists in the database
pub async fn check_if_snapshot_exists(
    dbclient: &DbClient,
    snapshot_to_check: i64,
    game_version: &str,
) -> Result<bool> {
    let snapshot = dbclient
        .get_specific_change_id(snapshot_to_check, game_version)
        .await?;

    Ok(snapshot.is_some())
}
