// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use anyhow::{bail, Ok, Result};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use reqwest::Client;
use std::env;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tauri::State;
use tokio::time;

use crate::db::DbClient;
use crate::models::api_models::RawLeagueApiResponse;
use crate::models::frontend_models::FrontendError;
use crate::models::logic_models::TradingCurrencyRates;
pub mod auth;
pub mod db;
pub mod ggg_api;
pub mod handlers;
pub mod logic;
pub mod models;

#[tauri::command(async)]
async fn get_leagues(
    state: State<'_, AppState>,
    realm: String,
) -> Result<RawLeagueApiResponse, FrontendError> {
    // let state = state;

    ggg_api::get_leagues_from_ggg(&state, &realm)
        .await
        .map_err(FrontendError::from)
}

#[tauri::command(async)]
async fn get_rates(
    state: State<'_, AppState>,
    realm: String,
    league: String,
) -> Result<TradingCurrencyRates, String> {
    let all_markets = ggg_api::get_most_recent_cxapi(&state, &realm)
        .await
        .map_err(|err| err.to_string())?;

    // TODO: Should I cache this? Worth refiltering every time? Not sure.
    let filtered_markets = all_markets.filter_league(&league);
    // dbg!(filtered_markets.first());

    let rates = logic::get_ggg_base_prices(&filtered_markets);
    // dbg!(&rates);
    Ok(rates)
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
        .invoke_handler(tauri::generate_handler![get_leagues, get_rates])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Holds a bunch of stuff the http client needs to function. This includes the
/// client itself, the rate limiter, the time at which we can make our next request
/// if we happen to get rate limited despite our best efforts, and the tokens we
/// need to make our requests.
pub struct ApiClient {
    http: Client,
    limiter: DefaultDirectRateLimiter,
    penalty_until_ms: AtomicI64, // Atomic because multiple clients could be writing
    cxapi_token: String,
    leagues_token: String,
}

impl ApiClient {
    pub fn new(http: Client, cxapitoken: &str, leaguesapitoken: &str) -> Self {
        // I need some flavor of backoff or retry to avoid getting rate limited
        // and banned. I can read the headers as provided by GGG:
        // https://www.pathofexile.com/developer/docs/index#ratelimits
        // and do the backoff accordingly. As of Jul 5 2026 the cxapi rate limit is
        // "30:60:60", which means 30 hits every 60 seconds before a 60 second timeout.
        // We can either trickle them in at less than or equal to 1 per 2 seconds or we can burst 30 and
        // then wait for a minute. I'll keep it 1 request short and change this if I hit errors later.
        // Hardcoding for now since quota and RateLimiter need compile-time guarantees
        // and can do some craziness with mutable Arcs (ArcSwap crate) later or something.
        // Will need a different limiter if I ever want to hit the river.
        let quota = Quota::with_period(Duration::from_secs(60) / 30)
            .unwrap()
            .allow_burst(nonzero!(30u32));
        Self {
            http,
            limiter: RateLimiter::direct(quota),
            penalty_until_ms: AtomicI64::new(0),
            cxapi_token: cxapitoken.to_string(),
            leagues_token: leaguesapitoken.to_string(),
        }
    }

    pub async fn get_url(
        &self,
        url: &str,
        required_scope: auth::AuthorizedScopes,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.wait_out_penalty().await;
        self.limiter.until_ready().await;

        let token = match required_scope {
            auth::AuthorizedScopes::Cxapi => &self.cxapi_token,
            auth::AuthorizedScopes::Leagues => &self.leagues_token,
        };

        let resp = self.http.get(url).bearer_auth(token).send().await?;
        // If we get hit with a 429, wait for
        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let penalty_seconds: i64 = resp
                .headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60) as i64;

            self.penalty_until_ms
                .store(now_ms() + penalty_seconds * 1000, Ordering::Relaxed);
            // bail!("Hit 429 despite limiter, delaying for hopefully enough time.")
        }

        Ok(resp)
    }

    async fn wait_out_penalty(&self) {
        loop {
            let until = self.penalty_until_ms.load(Ordering::Relaxed);
            let now = now_ms();
            if until <= now {
                return;
            }
            time::sleep(Duration::from_millis((until - now) as u64)).await;
        }
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

async fn build_http_client() -> Result<reqwest::Client, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent(env::var("USER_AGENT").expect("No valid USER_AGENT"))
        .build()?;
    Ok(client)
}
