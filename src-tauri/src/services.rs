use chrono::{DateTime, Local};

use crate::{
    db::models::UpdateOutcome,
    errors::FrontendError,
    ggg_api::{
        self,
        models::{RawLeagueApiResponse, Realm},
    },
    logic::{
        self,
        models::{Market, TradingCurrencyRates},
    },
    sync, AppState,
};

fn get_cached_leagues(state: &AppState, realm: Realm) -> Option<RawLeagueApiResponse> {
    let cached_leagues = state.league_cache.lock().unwrap();

    match realm {
        Realm::Poe1 => cached_leagues[0].clone(),
        Realm::Poe2 => cached_leagues[1].clone(),
    }
}

fn cache_leagues(state: &AppState, realm: Realm, leagues: &RawLeagueApiResponse) {
    let mut cached_leagues = state.league_cache.lock().unwrap();

    // dbg!("Caching leagues for {}", api_realm.to_string());
    match realm {
        Realm::Poe1 => cached_leagues[0] = Some(leagues.clone()),
        Realm::Poe2 => cached_leagues[1] = Some(leagues.clone()),
    };
}

fn parse_realm(realm: &str) -> Result<Realm, FrontendError> {
    realm
        .parse::<Realm>()
        .map_err(|e| FrontendError::InvalidInput { message: e })
}

pub async fn handle_current_leagues(
    state: &AppState,
    realm: &str,
) -> Result<RawLeagueApiResponse, FrontendError> {
    let api_realm = parse_realm(realm)?;
    let maybe_cached = get_cached_leagues(state, api_realm);

    let leagues = if let Some(val) = maybe_cached {
        val
    } else {
        // Didn't have a cached value, store the new value
        let new_leagues = ggg_api::get_leagues_from_ggg(state, &api_realm)
            .await
            .map_err(FrontendError::from)?;

        cache_leagues(state, api_realm, &new_leagues);
        new_leagues
    };

    Ok(leagues)
}

pub async fn handle_most_recent_update_time(
    state: &AppState,
    realm: &str,
) -> Result<String, FrontendError> {
    let api_realm = parse_realm(realm)?;

    let most_recent = state
        .db_client
        .get_latest_raw(&api_realm)
        .await
        .map_err(|e| FrontendError::Database {
            message: e.to_string(),
        })?;

    if let Some(val) = most_recent {
        let time = val.change_id;
        let local_time: DateTime<Local> = DateTime::from_timestamp_secs(time)
            .expect("Invalid timestamp")
            .with_timezone(&Local);
        // dbg!(local_time);
        Ok(local_time.to_string())
    } else {
        Err(FrontendError::Database {
            message: format!("No entry for realm {}", realm),
        })
    }
}

pub async fn handle_get_rates(
    state: &AppState,
    realm: &str,
    league: &str,
) -> Result<TradingCurrencyRates, FrontendError> {
    let api_realm = parse_realm(realm)?;

    let most_recent = state
        .db_client
        .get_latest_parsed_marketplace(&api_realm, league)
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
    // leagues are in a vector. Otherwise I re-pull it every time.
    // But then I have to deal with cache invalidation...
    let all_markets: Vec<Market> = most_recent.iter().map(Market::from).collect();

    // dbg!("num markets {}", &all_markets.len());

    let rates = logic::get_base_prices(&all_markets);
    // dbg!(&rates);
    Ok(rates)
}

pub async fn handle_update_database(
    state: &AppState,
    realm: &str,
) -> Result<UpdateOutcome, FrontendError> {
    dbg!("Updating Data".to_string());
    let api_realm = parse_realm(realm)?;

    let update_result = sync::update_and_run_elt(state, &api_realm)
        .await
        .map_err(|e| FrontendError::Database {
            message: (e.to_string()),
        })?;

    Ok(update_result)
}
