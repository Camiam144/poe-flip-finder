// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use anyhow::{bail, Ok, Result};
use chrono::{DateTime, Local};
use std::env;
use std::str::FromStr;
use tauri::Manager;
use tauri::State;

use crate::db::DbClient;
use crate::ggg_api::client::{build_http_client, ApiClient};
use crate::ggg_api::models::{RawCxApiResponse, RawLeagueApiResponse};
use crate::ggg_api::Realm;
use crate::models::api_models::GGGMarket;
use crate::models::frontend_models::FrontendError;
use crate::models::logic_models::TradingCurrencyRates;
pub mod auth;
pub mod db;
pub mod ggg_api;
pub mod logic;
pub mod models;
pub mod sync;

#[tauri::command(async)]
async fn get_leagues(
    state: State<'_, AppState>,
    realm: String,
) -> Result<RawLeagueApiResponse, FrontendError> {
    // let state = state;
    let api_realm: Realm = if let Ok(val) = Realm::from_str(&realm) {
        val
    } else {
        return Err(FrontendError::Other {
            message: "Invalid Realm Provided".to_string(),
        });
    };
    ggg_api::get_leagues_from_ggg(&state, api_realm)
        .await
        .map_err(FrontendError::from)
}

#[tauri::command(async)]
async fn get_most_recent_update_time(
    state: State<'_, AppState>,
    realm: String,
) -> Result<String, String> {
    let api_realm: Realm = if let Ok(val) = Realm::from_str(&realm) {
        val
    } else {
        return Err("Invalid Realm Provided".to_string());
    };

    let most_recent = state
        .db_client
        .get_latest(api_realm)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(val) = most_recent {
        let time = val.change_id;
        let local_time: DateTime<Local> = DateTime::from_timestamp_secs(time)
            .expect("Invalid timestamp")
            .with_timezone(&Local);
        // dbg!(local_time);
        Ok(local_time.to_string())
    } else {
        Err(format!("No entry for realm {}", realm))
    }
}

#[tauri::command(async)]
async fn get_rates(
    state: State<'_, AppState>,
    realm: String,
    league: String,
) -> Result<TradingCurrencyRates, String> {
    let api_realm: Realm = if let Ok(val) = Realm::from_str(&realm) {
        val
    } else {
        return Err("Invalid Realm Provided".to_string());
    };
    let most_recent = state
        .db_client
        .get_latest(api_realm)
        .await
        .map_err(|e| e.to_string())?;

    // Is there a cleaner way to write this?
    let response = if let Some(val) = most_recent {
        serde_json::from_str::<RawCxApiResponse>(&val.payload).map_err(|e| e.to_string())?
    } else {
        return Err::<TradingCurrencyRates, String>("Couldn't Find Most Recent Entry".to_string());
    };

    let all_markets = GGGMarket::try_from(response).map_err(|e| e.to_string())?;

    // TODO: Should I cache this? Worth refiltering every time? Not sure.
    // What I should be doing is working this out in my ETL/ELT pipeline that I
    // haven't written yet.
    let filtered_markets = all_markets.filter_league(&league);

    let rates = logic::get_ggg_base_prices(&filtered_markets);
    // dbg!(&rates);
    Ok(rates)
}

#[tauri::command(async)]
async fn update_database(state: State<'_, AppState>, realm: String) -> Result<String, String> {
    dbg!("Updating Data".to_string());
    let api_realm: Realm = if let Ok(val) = Realm::from_str(&realm) {
        val
    } else {
        return Err("Invalid Realm Provided".to_string());
    };
    sync::get_update_data(&state, api_realm)
        .await
        .map_err(|e| e.to_string())?;

    Ok("success".to_string())
}

pub struct AppState {
    db_client: DbClient,
    http_client: ApiClient,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
