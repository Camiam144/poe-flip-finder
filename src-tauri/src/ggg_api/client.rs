use crate::auth;
use crate::ggg_api::models::ApiError;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
// use nonzero_ext::nonzero;
use reqwest::Client;
use std::env;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
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
        // Will need a different limiter if I ever want to hit the river.
        let quota = Quota::with_period(Duration::from_secs(60) / 30).unwrap();
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
    ) -> Result<reqwest::Response, ApiError> {
        self.wait_out_penalty().await;
        self.limiter.until_ready().await;

        let token = match required_scope {
            auth::AuthorizedScopes::Cxapi => &self.cxapi_token,
            auth::AuthorizedScopes::Leagues => &self.leagues_token,
        };

        let resp = self
            .http
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(ApiError::Network)?;

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
            return Err(ApiError::Api {
                code: super::models::GGGErrorCode::RateLimitExceeded,
                message: "Rate Limit Exceeded".to_string(),
            });
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
            dbg!(format!(
                "Hit penalty, waiting {} ms",
                self.penalty_until_ms.load(Ordering::Relaxed) - now_ms()
            ));
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

pub async fn build_http_client() -> Result<reqwest::Client, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent(env::var("USER_AGENT").expect("No valid USER_AGENT"))
        .build()?;
    Ok(client)
}
