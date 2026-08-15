//! This module contains stuff for working with GGG's API
//! This is not internal API endpoints.
use anyhow::{anyhow, Context};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::AuthorizedScopes;
use crate::db::DbClient;
use crate::models::api_models::{
    ApiError, GGGMarket, GGGWrappedError, RawCxApiResponse, RawLeagueApiResponse,
};
use crate::{ApiClient, AppState};

/// Make a call to GGG's CXAPI to get an entry
pub async fn get_specified_cxapi_from_ggg(
    client: &ApiClient,
    db_client: &DbClient,
    realm: &str,
    time: i64,
) -> anyhow::Result<RawCxApiResponse> {
    let base_url = "https://api.pathofexile.com/currency-exchange/";

    // For some reason if the realm is "poe1" you need to not include it,
    // for every other realm you need to include it
    let url = match realm {
        "poe1" => format!("{}{}", base_url, time),
        _ => format!("{}{}/{}", base_url, realm.to_lowercase(), time),
    };
    dbg!(format!("url: {}", url));

    let raw_response = client.get_url(&url, AuthorizedScopes::Cxapi).await?;

    // In order to get both the raw text and the json we have to read the entire
    // stream to a string (consuming the stream), save the string, and then
    // use serde to deserialize the string to the rust object we want.
    let text_response = raw_response.text().await?;
    // I need to do some error checking to make sure I'm not storing invalid responses

    // cache to db
    // This probably shouldn't live in this function, we don't first check if the
    // response already exists in the DB.
    db_client.insert_data(time, realm, &text_response).await?;

    Ok(serde_json::from_str::<RawCxApiResponse>(&text_response)?)
}

/// Get the specified cxapi realm and timestamp from the cache if it exists,
/// otherwise grab it from the API and cache it. I think my logic here is too
/// intertwined and I don't want to make this decision automatically. If I don't
/// have the most recent data I should instead update the db to the most recent point.
pub async fn get_specified_cxapi(
    db_client: &DbClient,
    realm: &str,
    time: i64,
) -> anyhow::Result<GGGMarket> {
    let market = db_client.get_specific_change_id(time, realm).await?;

    // let response = if let Some(val) = market {
    //     dbg!("Pulling newest data from cache");
    //     serde_json::from_str::<RawCxApiResponse>(&val.payload)
    //         .context("Should have been able to parse cached row")?
    // } else {
    //     dbg!("Pulling newest data from GGG");
    //     get_specified_cxapi_from_ggg(client, db_client, realm, time).await?
    // };
    //

    match market {
        Some(val) => {
            dbg!("Pulling newest data from cache");
            let response = serde_json::from_str::<RawCxApiResponse>(&val.payload)
                .context("Should have been able to parse cached row")?;

            GGGMarket::try_from(response)
        }
        None => {
            dbg!("Value not in cache");
            Err(anyhow!("No value found for change_id {}", time))
        }
    }
}

/// Get the entire Cxapi dump from the past hour. The current hour does not yet
/// have information.
pub async fn get_most_recent_cxapi(state: &AppState, realm: &str) -> anyhow::Result<GGGMarket> {
    // Safety: duration_since UNIX_EPOCH should never fail unless the system clock is set
    // to before the UNIX_EPOCH
    let past_hour = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600;

    get_specified_cxapi(&state.db_client, realm, past_hour.try_into().unwrap()).await
}

/// Update the database from the most recent recorded timestamp to the most recent available hour
/// Use timers to avoid getting rate limited
pub async fn get_update_data(state: &AppState, realm: &str) -> anyhow::Result<()> {
    let past_hour = ((SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600) as i64;
    dbg!(format!("Most recent hour should be {}", past_hour));

    let most_recent_entry = state.db_client.get_latest(realm).await?;
    // TODO: If most_recent_entry is none, we need to either throw an error or
    // pull data from the beginning?

    // Build a list of which values we need
    if let Some(most_recent) = most_recent_entry {
        dbg!(format!(
            "Most recent entry in db time is {}",
            most_recent.change_id
        ));

        if most_recent.change_id == past_hour {
            dbg!(format!(
                "Most recent time in db {} is equal to past hour {}",
                most_recent.change_id, past_hour
            ));
            return anyhow::Ok(());
        }
        let mut change_ids = Vec::new();

        // Do this to build the list so tokio can run them simultaneously
        for timestamp in ((most_recent.change_id + 3600)..=past_hour).step_by(3600) {
            change_ids.push(timestamp);
        }
        dbg!(&change_ids);

        // Run sequentially, the 1 request per 2 second rate limit is the bottleneck.
        for change_id in change_ids {
            dbg!(format!("Pulling data for {}", &change_id));
            let _ =
                get_specified_cxapi(&state.http_client, &state.db_client, realm, change_id).await?;
        }
        dbg!("Database up to date");
    } else {
        dbg!("No most recent entry, you need to handle this path");
        return Err(anyhow!(
            "No most recent data for realm {} you need to figure it out",
            realm
        ));
    }

    anyhow::Ok(())
}

/// Get all current leagues from GGG's API. This should be cached and
/// updated as needed instead of pinging it every time.
pub async fn get_leagues_from_ggg(
    state: &AppState,
    realm: &str,
) -> Result<RawLeagueApiResponse, ApiError> {
    // TODO: Cache these in the database, check once per day?
    // Cache on front end, expire once per day? Cache somewhere for sure
    let url = "https://api.pathofexile.com/league";
    let realm = if realm == "poe1" { "pc" } else { realm };
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params).expect("Malformed leagues url.");

    let response = state
        .http_client
        .get_url(url.as_str(), AuthorizedScopes::Leagues)
        .await?;
    // dbg!(&response);

    let status = response.status();
    let bytes = response.bytes().await?;

    if status.is_success() {
        match serde_json::from_slice::<RawLeagueApiResponse>(&bytes) {
            Ok(data) => std::result::Result::Ok(data),
            Err(parse_error) => {
                // Wrapped error with success status code
                if let Ok(wrap) = serde_json::from_slice::<GGGWrappedError>(&bytes) {
                    Err(ApiError::Api {
                        code: wrap.error.code,
                        message: wrap.error.message,
                    })
                } else {
                    Err(ApiError::Parse(parse_error))
                }
            }
        }
    } else {
        match serde_json::from_slice::<GGGWrappedError>(&bytes) {
            Ok(wrapped_error) => Err(ApiError::Api {
                code: wrapped_error.error.code,
                message: wrapped_error.error.message,
            }),
            Err(_) => Err(ApiError::Api {
                code: (status.as_u16() as i32).into(),
                message: format!("Bad status and unparseable body."),
            }),
        }
    }
}
