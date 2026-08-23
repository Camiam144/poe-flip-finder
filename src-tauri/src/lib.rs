// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use anyhow::{bail, Ok, Result};
pub mod auth;
pub mod db;
pub mod errors;
pub mod ggg_api;
pub mod logic;
pub mod models;
pub mod sync;

use chrono::{DateTime, Local};
use std::env;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{Manager, State};

use crate::db::models::UpdateOutcome;
use crate::db::DbClient;
use crate::errors::FrontendError;
use crate::ggg_api::client::{build_http_client, ApiClient};
use crate::ggg_api::models::{RawLeagueApiResponse, Realm};
use crate::logic::models::TradingCurrencyRates;

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
    {
        let cached_leagues = state.league_cache.lock().unwrap();

        let maybe_cache = match api_realm {
            Realm::Poe1 => cached_leagues[0].as_ref(),
            Realm::Poe2 => cached_leagues[1].as_ref(),
        };

        if let Some(cached) = maybe_cache {
            dbg!("Pulling leagues from cache");
            return Ok(cached.clone());
        }
    }

    // Didn't have a cached value, store the new value
    let leagues = ggg_api::get_leagues_from_ggg(&state, api_realm)
        .await
        .map_err(FrontendError::from)?;

    {
        let mut cached_leagues = state.league_cache.lock().unwrap();

        // dbg!("Caching leagues for {}", api_realm.to_string());

        match api_realm {
            Realm::Poe1 => cached_leagues[0] = Some(leagues.clone()),
            Realm::Poe2 => cached_leagues[1] = Some(leagues.clone()),
        };
    }

    Ok(leagues)
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
        .get_latest_raw(api_realm)
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
) -> Result<TradingCurrencyRates, FrontendError> {
    let api_realm: Realm = if let Ok(val) = Realm::from_str(&realm) {
        val
    } else {
        return Err(FrontendError::InvalidInput {
            message: "Invalid Realm Provided".to_string(),
        });
    };
    let most_recent = state
        .db_client
        .get_latest_parsed_marketplace(api_realm, &league)
        .await
        .map_err(|e| FrontendError::Database {
            message: e.to_string(),
        })?;

    if most_recent.is_empty() {
        return Err(FrontendError::Database {
            message: format!("No most recent entry for {} and {}", api_realm, league),
        });
    }

    // TODO: This should probably be cached in a kv cache in app state, like the
    // leagues are in a vector.
    let all_markets: Vec<logic::models::Market> = most_recent
        .iter()
        .map(logic::models::Market::from)
        .collect();

    // dbg!("num markets {}", &all_markets.len());

    let rates = logic::get_base_prices(&all_markets);
    // dbg!(&rates);
    Ok(rates)
}

#[tauri::command(async)]
async fn update_database(
    state: State<'_, AppState>,
    realm: String,
) -> Result<UpdateOutcome, FrontendError> {
    dbg!("Updating Data".to_string());
    let api_realm: Realm = if let Ok(val) = Realm::from_str(&realm) {
        val
    } else {
        return Err(FrontendError::InvalidInput {
            message: ("Invalid Realm Provided".to_string()),
        });
    };

    let update_result = sync::update_and_run_elt(&state, api_realm)
        .await
        .map_err(|e| FrontendError::Database {
            message: (e.to_string()),
        })?;

    Ok(update_result)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
