use anyhow::Result;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use reqwest::StatusCode;

use crate::AppState;
use crate::models::api_models::{GGGLeague, GGGLeagueList};

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not found go 404 yourself")
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "RUST API Example with Axum Framework and Sqlite.";

    let json_response = serde_json::json!({
        "status" : "success",
        "message" : MESSAGE
    });

    Json(json_response)
}

pub async fn hello_world_handler() -> impl IntoResponse {
    format!("Welcome to poe_flip_finder {}", env!("CARGO_PKG_VERSION"))
}

/// Get all current leagues from GGG's API. This should be cached and
/// updated as needed instead of pinging it every time
pub async fn leagues_handler(
    State(data): State<Arc<AppState>>,
    Path(realm): Path<String>,
) -> impl IntoResponse {
    // TODO: Cache these in the database, check once per day?
    let token = &data.leagues_token;
    let url = "https://api.pathofexile.com/league";
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params).expect("Couldn't build leagues url");

    let response = data
        .http_client
        .get(url)
        .bearer_auth(token)
        // .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("Couldn't get data from GGG");

    let result: GGGLeagueList = response.json().await.expect("Couldn't parse result");

    Json(result)
}
