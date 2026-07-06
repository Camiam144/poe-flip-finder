use anyhow::{Ok, Result, bail};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use reqwest::Client;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, sync::Arc};
use tokio::time;

use axum::{Router, routing::get};
mod ggg_api;
// mod app;
mod auth;
mod db;
mod handlers;
mod logic;
mod models;

use crate::db::DbClient;
use crate::handlers::{
    handler_404, health_checker_handler, hello_world_handler, leagues_handler,
    most_recent_cxapi_handler, update_data_handler,
};

// use app::App;

struct AppState {
    db_client: DbClient,
    http_client: ApiClient,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    let db_url = env::var("DB_URL").expect("No valid DB_URL");
    let db_client = DbClient::try_from_path(db_url.into())
        .await
        .expect("Should have created a database.");

    let http_client = build_http_client()
        .await
        .expect("Should have built http client");

    let cxapi_token = env::var("AUTH_TOKEN_SERVICE_CXAPI").unwrap();
    let leagues_token = env::var("AUTH_TOKEN_SERVICE_LEAGUES").unwrap();

    let api_client = ApiClient::new(http_client, &cxapi_token, &leagues_token);

    let app_state = Arc::new(AppState {
        db_client,
        http_client: api_client,
    });

    // Routes need to be bound here
    let app = Router::new()
        .route("/", get(hello_world_handler))
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/{realm}", get(leagues_handler))
        .route("/api/most_recent", get(most_recent_cxapi_handler))
        .route("/api/update", get(update_data_handler))
        .with_state(app_state);
    let app = app.fallback(handler_404);

    println!("Server started successfully");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Should have been able to bind to port 8000");
    axum::serve(listener, app)
        .await
        .expect("Should have been able to start server");

    println!("Listening on localhost:8000");
}

/// Holds a bunch of stuff the http client needs to function. This includes the
/// client itself, the rate limiter, the time at which we can make our next request
/// if we happen to get rate limited despite our best efforts, and the tokens we
/// need to make our requests.
pub struct ApiClient {
    http: Client,
    limiter: DefaultDirectRateLimiter,
    penalty_until_ms: AtomicI64,
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
        // We can either trickle them in at less than or equal to 2 per second or we can burst 30 and
        // then wait for a minute. I'll keep it 1 request short and change this if I hit errors later.
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
    ) -> Result<reqwest::Response> {
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
            bail!("Hit 429 despite limiter, delaying for hopefully enough time.")
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

async fn build_http_client() -> Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .user_agent(env::var("USER_AGENT").expect("No valid USER_AGENT"))
        .build()?;
    Ok(client)
}
