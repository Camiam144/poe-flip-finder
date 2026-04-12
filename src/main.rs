use anyhow::{Ok, Result};
use reqwest::Client;
use std::{env, sync::Arc};

use axum::{Router, routing::get};
// mod api;
// mod app;
mod auth;
mod db;
// mod logic;
mod handlers;
mod models;

use crate::db::DbClient;
use crate::handlers::{
    handler_404, health_checker_handler, hello_world_handler, leagues_handler,
    most_recent_cxapi_handler,
};

// use app::App;

struct AppState {
    db_client: DbClient,
    http_client: Client,
    cxapi_token: String,
    leagues_token: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let db_url = env::var("DB_URL").expect("No valid DB_URL");
    let db_client = DbClient::try_from_path(db_url.into())
        .await
        .expect("Should have created a database.");

    let http_client = build_http_client()
        .await
        .expect("Should have built http client");

    let cxapi_token = env::var("AUTH_TOKEN_SERVICE_CXAPI").unwrap();
    let leagues_token = env::var("AUTH_TOKEN_SERVICE_LEAGUES").unwrap();

    let app_state = Arc::new(AppState {
        db_client,
        http_client,
        cxapi_token,
        leagues_token,
    });

    // Routes need to be bound here
    let app = Router::new()
        .route("/", get(hello_world_handler))
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/{realm}", get(leagues_handler))
        .route("/api/most_recent", get(most_recent_cxapi_handler))
        .with_state(app_state);
    let app = app.fallback(handler_404);

    println!("Server started successfully");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Should have been able to bind to port 8000");
    axum::serve(listener, app)
        .await
        .expect("Should have been able to start server")
}

async fn build_http_client() -> Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .user_agent(env::var("USER_AGENT").expect("No valid USER_AGENT"))
        .build()?;
    Ok(client)
}
