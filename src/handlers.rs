use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use reqwest::{Client, StatusCode};

use crate::AppState;
use crate::db::DbClient;
use crate::models::api_models::{GGGLeagueList, GGGMarket, RawCxApiResponse};

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not found go 404 yourself")
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "RUST API Example with Axum Framework and Sqlite.";

    let json_response = serde_json::json!({
        "status" : "success",
        "message" : MESSAGE
    });

    (StatusCode::OK, Json(json_response))
}

pub async fn hello_world_handler() -> impl IntoResponse {
    format!("Welcome to poe_flip_finder {}", env!("CARGO_PKG_VERSION"))
}

/// Get all current leagues from GGG's API. This should be cached and
/// updated as needed instead of pinging it every time
pub async fn leagues_handler(
    State(data): State<Arc<AppState>>,
    Path(realm): Path<String>,
) -> Response {
    // TODO: Cache these in the database, check once per day?
    // Should the logic go in a helper function(s) and handler *only* calls functions?
    // This technically builds the url *AND* gets the data *AND* deserializes the data *AND*
    // returns it to the caller.
    let token = &data.leagues_token;
    let url = "https://api.pathofexile.com/league";
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params).expect("Couldn't build leagues url");

    let response = match data
        .http_client
        .get(url)
        .bearer_auth(token)
        // .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Couldn't get League List from GGG API.",
            )
                .into_response();
        }
    };

    let result: GGGLeagueList = match response.json().await {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Couldn't deserialize GGG League List",
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(result)).into_response()
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

    // TODO: I don't really like this function, too much logic in the else block
    // should be like
    // if let Some(val) = market {get_from_db()} else {get_from_GGG()};
    let response = if let Some(val) = market {
        serde_json::from_str::<RawCxApiResponse>(&val.payload)
            .context("Should have been able to parse cached row")?
    } else {
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

        // save the stuff to the db
        db_client.insert_data(time, realm, &text_response).await?;

        serde_json::from_str::<RawCxApiResponse>(&text_response)?
    };

    GGGMarket::try_from_raw_cxapi_response(response)
}

/// Get the entire Cxapi dump from the past hour. The current hour does not yet
/// have information.
pub async fn most_recent_cxapi_handler(
    State(data): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let past_hour = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600;

    let recent = match get_specified_cxapi(
        &data.http_client,
        &data.cxapi_token,
        &data.db_client,
        &params["realm"],
        past_hour.try_into().unwrap(),
    )
    .await
    {
        Ok(val) => val,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error pulling data.").into_response();
        }
    };
    (StatusCode::OK, Json(recent)).into_response()
}
