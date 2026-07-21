use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;

use crate::models::api_models::RawLeagueApiResponse;
use crate::{ggg_api::get_update_data, logic};
use crate::{
    ggg_api::{get_leagues_from_ggg, get_most_recent_cxapi},
    models::api_models::Market,
};
use crate::{logic::get_ggg_base_prices, AppState};

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

/// Get all current leagues
pub async fn leagues_handler(
    State(data): State<Arc<AppState>>,
    Path(realm): Path<String>,
) -> Response {
    let response = get_leagues_from_ggg(&data, &realm).await;

    let result: RawLeagueApiResponse = match response {
        Ok(value) => value,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Couldn't get GGG League List\n{}", err),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(result)).into_response()
}

// Get the entire Cxapi dump from the past hour. The current hour does not yet
// have information. Accepts a query of the realm
// pub async fn most_recent_cxapi_handler(
//     State(data): State<Arc<AppState>>,
//     Query(params): Query<HashMap<String, String>>,
// ) -> Response {
//     let Some(realm) = params.get("realm") else {
//         return (StatusCode::BAD_REQUEST, "Invalid Realm").into_response();
//     };
//
//     let recent = match get_most_recent_cxapi(data, realm).await {
//         Ok(val) => val,
//         Err(_) => {
//             return (StatusCode::INTERNAL_SERVER_ERROR, "Error pulling data.").into_response();
//         }
//     };
//     (StatusCode::OK, Json(recent)).into_response()
// }

// Update all of the data to the most recent version
// pub async fn update_data_handler(
//     State(data): State<Arc<AppState>>,
//     Query(params): Query<HashMap<String, String>>,
// ) -> Response {
//     let Some(realm) = params.get("realm") else {
//         return (StatusCode::BAD_REQUEST, "Invalid Realm").into_response();
//     };
//
//     match get_update_data(data, realm).await {
//         Ok(()) => (StatusCode::OK).into_response(),
//         Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error pulling data.").into_response(),
//     }
// }

// Get the whole list of arbitrage options
// pub async fn get_arbitrage_handler(
//     State(data): State<Arc<AppState>>,
//     Query(params): Query<HashMap<String, String>>,
// ) -> Response {
//     let Some(realm) = params.get("realm") else {
//         return (StatusCode::BAD_REQUEST, "Invalid Realm").into_response();
//     };
//
//     let current_records = match get_most_recent_cxapi(data, realm).await {
//         Ok(val) => val,
//         Err(_) => {
//             return (StatusCode::INTERNAL_SERVER_ERROR, "Error pulling data.").into_response();
//         }
//     };
//
//     let base_rates = get_ggg_base_prices(&current_records.markets);
//
//     // TODO: Write this function in logic
//     let arbitrage = current_records;
//
//     (StatusCode::OK, Json(arbitrage)).into_response()
// }
