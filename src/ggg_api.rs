//! This module contains stuff for working with GGG's API
//! This is not internal API endpoints.
use anyhow::{Context, Ok, Result};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::AuthorizedScopes;
use crate::db::DbClient;
use crate::models::api_models::{GGGLeagueList, GGGMarket, RawCxApiResponse};
use crate::{ApiClient, AppState};

/// Make a call to GGG's CXAPI to get an entry
pub async fn get_specified_cxapi_from_ggg(
    client: &ApiClient,
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

    let raw_response = client.get_url(&url, AuthorizedScopes::Cxapi).await?;

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
    client: &ApiClient,
    db_client: &DbClient,
    realm: &str,
    time: i64,
) -> Result<GGGMarket> {
    let market = db_client.get_specific_change_id(time, realm).await?;

    let response = if let Some(val) = market {
        dbg!("Pulling newest data from cache");
        serde_json::from_str::<RawCxApiResponse>(&val.payload)
            .context("Should have been able to parse cached row")?
    } else {
        dbg!("Pulling newest data from GGG");
        get_specified_cxapi_from_ggg(client, db_client, realm, time).await?
    };

    GGGMarket::try_from(response)
}

/// Get the entire Cxapi dump from the past hour. The current hour does not yet
/// have information.
pub async fn get_most_recent_cxapi(state: Arc<AppState>, realm: &str) -> Result<GGGMarket> {
    // Safety: duration_since UNIX_EPOCH should never fail unless the system clock is set
    // to before the UNIX_EPOCH
    let past_hour = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600;

    get_specified_cxapi(
        &state.http_client,
        &state.db_client,
        realm,
        past_hour.try_into().unwrap(),
    )
    .await
}

/// Update the database from the most recent recorded timestamp to the most recent available hour
/// Use timers to avoid getting rate limited
pub async fn get_update_data(state: Arc<AppState>, realm: &str) -> Result<()> {
    let past_hour = ((SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600) as i64;
    dbg!("Most recent hour should be {}", past_hour);

    let most_recent_entry = state.db_client.get_latest(realm).await?;
    // TODO: If most_recent_entry is none, we need to either throw an error or
    // pull data from the beginning?

    // Build a list of which values we need
    if let Some(most_recent) = most_recent_entry
        && most_recent.change_id < past_hour
    {
        dbg!("Most recent entry in db time is {}", most_recent.change_id);
        for timestamp in ((most_recent.change_id + 3600)..past_hour).skip(3600) {
            dbg!("Pulling data for {}", &timestamp);
            let _ =
                get_specified_cxapi(&state.http_client, &state.db_client, realm, timestamp).await?;
        }
    }
    dbg!("Database up to date");

    Ok(())
}

/// Get all current leagues from GGG's API. This should be cached and
/// updated as needed instead of pinging it every time.
pub async fn get_leagues_from_ggg(state: Arc<AppState>, realm: &str) -> Result<GGGLeagueList> {
    // TODO: Cache these in the database, check once per day?
    // Cache on front end, expire once per day? Cache somewhere for sure
    let url = "https://api.pathofexile.com/leagues";
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params)?;

    let response = state
        .http_client
        .get_url(url.as_str(), AuthorizedScopes::Leagues)
        .await?;
    // dbg!(&response);

    let text_response = dbg!(response.text().await?);

    Ok(serde_json::from_str::<GGGLeagueList>(&text_response)?)
}
