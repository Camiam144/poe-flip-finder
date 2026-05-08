//! This module contains stuff for working with GGG's API
//! This is not internal API endpoints.
use anyhow::{Context, Ok, Result};
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::AppState;
use crate::db::DbClient;
use crate::models::api_models::{GGGLeagueList, GGGMarket, RawCxApiResponse};

/// Make a call to GGG's CXAPI to get an entry
pub async fn get_specified_cxapi_from_ggg(
    client: &Client,
    cxapi_token: &str,
    db_client: &DbClient,
    realm: &str,
    time: i64,
) -> Result<RawCxApiResponse> {
    let base_url = "https://api.pathofexile.com/currency-exchange/";

    // For some reason if the realm is "poe1" you need to not include it,
    // for every other realm you need to include it
    let url = match realm {
        "poe1" => format!("{}{}", base_url, time),
        _ => format!("{}{}/{}", base_url, realm.to_lowercase(), time),
    };
    println!("{}", url);

    let raw_response = client.get(url).bearer_auth(cxapi_token).send().await?;

    // In order to get both the raw text and the json we have to read the entire
    // stream to a string (consuming the stream), save the string, and then
    // use serde to deserialize the string to the rust object we want.
    let text_response = raw_response.text().await?;
    // I need to do some error checking to make sure I'm not storing invalid responses

    // cache to db
    db_client.insert_data(time, realm, &text_response).await?;

    Ok(serde_json::from_str::<RawCxApiResponse>(&text_response)?)
}
/// Get the specified cxapi realm and timestamp from the cache if it exists,
/// otherwise grab it from the API and cache it.
pub async fn get_specified_cxapi(
    client: &Client,
    cxapi_token: &str,
    db_client: &DbClient,
    realm: &str,
    time: i64,
) -> Result<GGGMarket> {
    let market = db_client.get_specific_change_id(time, realm).await?;

    let response = if let Some(val) = market {
        serde_json::from_str::<RawCxApiResponse>(&val.payload)
            .context("Should have been able to parse cached row")?
    } else {
        get_specified_cxapi_from_ggg(client, cxapi_token, db_client, realm, time).await?
    };

    GGGMarket::try_from_raw_cxapi_response(response)
}

/// Get the entire Cxapi dump from the past hour. The current hour does not yet
/// have information.
pub async fn get_most_recent_cxapi(state: Arc<AppState>, realm: &str) -> Result<GGGMarket> {
    let past_hour = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600;

    get_specified_cxapi(
        &state.http_client,
        &state.cxapi_token,
        &state.db_client,
        realm,
        past_hour.try_into().unwrap(),
    )
    .await
}

/// Get all current leagues from GGG's API. This should be cached and
/// updated as needed instead of pinging it every time.
pub async fn get_leagues_from_ggg(state: Arc<AppState>, realm: &str) -> Result<GGGLeagueList> {
    // TODO: Cache these in the database, check once per day?
    // Cache on front end, expire once per day? Cache somewhere for sure
    let token = &state.leagues_token;
    let url = "https://api.pathofexile.com/leagues";
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params)?;

    let response = state
        .http_client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;

    let result: GGGLeagueList = response.json().await?;

    Ok(result)
}
