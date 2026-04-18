use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;

use crate::AppState;
use crate::api::{get_leagues_from_ggg, get_most_recent_cxapi};
use crate::db::DbClient;
use crate::models::api_models::GGGLeagueList;

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
    let response = get_leagues_from_ggg(data, &realm).await;

    let result: GGGLeagueList = match response {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Couldn't get GGG League List",
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(result)).into_response()
}

/// Get the entire Cxapi dump from the past hour. The current hour does not yet
/// have information.
pub async fn most_recent_cxapi_handler(
    State(data): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(realm) = params.get("realm") else {
        return (StatusCode::BAD_REQUEST, "Invalid Realm").into_response();
    };

    let recent = match get_most_recent_cxapi(data, realm).await {
        Ok(val) => val,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error pulling data.").into_response();
        }
    };
    (StatusCode::OK, Json(recent)).into_response()
}
