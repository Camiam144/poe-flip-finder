// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use anyhow::{bail, Ok, Result};
pub mod auth;
pub mod db;
pub mod errors;
pub mod ggg_api;
pub mod logic;
pub mod services;
pub mod sync;

use std::env;
use std::sync::Mutex;
use tauri::{Manager, State};

use crate::db::models::UpdateOutcome;
use crate::db::DbClient;
use crate::errors::FrontendError;
use crate::ggg_api::client::{build_http_client, ApiClient};
use crate::ggg_api::models::RawLeagueApiResponse;
use crate::logic::models::{ArbitrageOpportunity, TradingCurrencyRates};
use crate::services::{
    handle_current_leagues, handle_get_opportunities, handle_get_rates,
    handle_most_recent_update_time, handle_update_database,
};

#[tauri::command(async)]
async fn get_leagues(
    state: State<'_, AppState>,
    realm: String,
) -> Result<RawLeagueApiResponse, FrontendError> {
    // let state = state;
    handle_current_leagues(&state, &realm).await
}

#[tauri::command(async)]
async fn get_most_recent_update_time(
    state: State<'_, AppState>,
    realm: String,
) -> Result<String, FrontendError> {
    handle_most_recent_update_time(&state, &realm).await
}

#[tauri::command(async)]
async fn get_rates(
    state: State<'_, AppState>,
    realm: String,
    league: String,
) -> Result<TradingCurrencyRates, FrontendError> {
    handle_get_rates(&state, &realm, &league).await
}

#[tauri::command(async)]
async fn update_database(
    state: State<'_, AppState>,
    realm: String,
) -> Result<UpdateOutcome, FrontendError> {
    handle_update_database(&state, &realm).await
}

#[tauri::command(async)]
async fn get_arbitrage(
    state: State<'_, AppState>,
    realm: String,
    league: String,
) -> Result<Vec<ArbitrageOpportunity>, FrontendError> {
    handle_get_opportunities(&state, &realm, &league).await
}

pub struct AppState {
    db_client: DbClient,
    http_client: ApiClient,
    league_cache: Mutex<[Option<RawLeagueApiResponse>; 2]>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            dotenvy::dotenv().unwrap();
            let db_url = env::var("DB_URL").expect("No valid DB_URL");
            let cxapi_token = env::var("AUTH_TOKEN_SERVICE_CXAPI").unwrap();
            let leagues_token = env::var("AUTH_TOKEN_SERVICE_LEAGUES").unwrap();

            let app_state = tauri::async_runtime::block_on(async move {
                let db_client = DbClient::try_from_path(db_url.into()).await?;
                let http_client = build_http_client().await?;
                let api_client = ApiClient::new(http_client, &cxapi_token, &leagues_token);

                anyhow::Ok(AppState {
                    db_client,
                    http_client: api_client,
                    league_cache: Mutex::new([None, None]),
                })
            })?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_leagues,
            get_rates,
            get_most_recent_update_time,
            update_database,
            get_arbitrage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
