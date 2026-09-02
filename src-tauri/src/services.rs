use chrono::{DateTime, Local};

use crate::{
    db::models::UpdateOutcome,
    errors::FrontendError,
    ggg_api::{
        self,
        models::{RawLeagueApiResponse, Realm},
    },
    logic::{
        self, build_and_populate_graph, get_all_arbitrage_options,
        models::{ArbitrageOpportunity, Market, TradingCurrencyRates},
    },
    sync, AppState,
};

fn parse_realm(realm: &str) -> Result<Realm, FrontendError> {
    realm
        .parse::<Realm>()
        .map_err(|e| FrontendError::InvalidInput { message: e })
}

/// Get the active leagues for a realm and cache them if they aren't cached.
pub async fn handle_current_leagues(
    state: &AppState,
    realm: &str,
) -> Result<RawLeagueApiResponse, FrontendError> {
    let api_realm = parse_realm(realm)?;

    // Don't have to worry about cache invalidation here because the leagues
    // change on a monthly cadence.
    let leagues = if let Some(val) = state.get_cached_leagues(&api_realm) {
        val
    } else {
        // Didn't have a cached value, store the new value
        let new_leagues = ggg_api::get_leagues_from_ggg(state, &api_realm)
            .await
            .map_err(FrontendError::from)?;

        state.cache_leagues(&api_realm, &new_leagues);
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

async fn get_newest_parsed_change_id(
    state: &AppState,
    realm: &Realm,
    league: &str,
) -> Result<i64, FrontendError> {
    let newest_change_id = state
        .db_client
        .get_most_recent_parsed_changeid(realm, league)
        .await
        .map_err(|e| FrontendError::Database {
            message: e.to_string(),
        })?;

    if let Some(id) = newest_change_id {
        Ok(id)
    } else {
        Err(FrontendError::Database {
            message: format!("No most recent parsed entry for {} and {}", realm, league),
        })
    }
}

async fn get_most_recent_markets(
    state: &AppState,
    realm: &Realm,
    league: &str,
) -> Result<Vec<Market>, FrontendError> {
    let most_recent = state
        .db_client
        .get_latest_parsed_marketplace(realm, league)
        .await
        .map_err(|e| FrontendError::Database {
            message: e.to_string(),
        })?;

    if most_recent.is_empty() {
        return Err(FrontendError::Database {
            message: format!("No most recent entry for {} and {}", realm, league),
        });
    }

    Ok(most_recent.iter().map(Market::from).collect())
}

/// Check if we need to invalidate the cache based on our db
async fn get_or_update_most_recent_market_cache(
    state: &AppState,
    realm: &Realm,
    league: &str,
) -> Result<Vec<Market>, FrontendError> {
    // First we check the cache. If it doesn't exist, get a new one.
    // If the cache does exist, check the change id. If cache id < change_id,
    // invalidate and get a new cache.

    let maybe_cache = state.get_cached_most_recent_markets(realm);
    let newest_change_id = get_newest_parsed_change_id(state, realm, league).await?;

    let is_stale = match maybe_cache {
        Some(cache) => cache.0 < newest_change_id,
        None => true,
    };

    if is_stale {
        let most_recent = get_most_recent_markets(state, realm, league).await?;
        state.cache_most_recent_markets(realm, newest_change_id, most_recent);
    }

    // We now know markets are in the cache
    Ok(state.get_cached_most_recent_markets(realm).unwrap().1)
}

pub async fn handle_get_rates(
    state: &AppState,
    realm: &str,
    league: &str,
) -> Result<TradingCurrencyRates, FrontendError> {
    let api_realm = parse_realm(realm)?;

    let all_markets = get_or_update_most_recent_market_cache(state, &api_realm, league).await?;

    // This also could be in a kv cache. Could be as simple as a cache that has
    // like <(realm, league, timestamp), (Vec<Market>, TCR)> and if the key doesn't exist
    // then I have to go pull. Or a struct that holds the markets, TCR, Graph, and
    // all of that stuff for a given realm, league and timestamp.
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

/// Get all of the opportunities available.
pub async fn handle_get_opportunities(
    state: &AppState,
    realm: &str,
    league: &str,
) -> Result<Vec<ArbitrageOpportunity>, FrontendError> {
    // TODO: Stopgap for now of just pulling most recent. Eventually I will want
    // to pull the past N steps and see what has been inefficient during that
    // whole time.
    let api_realm = parse_realm(realm)?;

    let all_markets = get_or_update_most_recent_market_cache(state, &api_realm, league).await?;
    let rates = logic::get_base_prices(&all_markets);

    let graph = build_and_populate_graph(&all_markets);
    // This is hardcoded for now but will eventually be a parameter from the frontend.
    let max_depth: usize = 3;
    let min_volume: i64 = 0;
    let good_options: Vec<ArbitrageOpportunity> = get_all_arbitrage_options(&graph, max_depth)
        .iter()
        .filter(|opp| opp.is_profitable(&rates) && opp.min_volume() >= min_volume)
        .cloned()
        .collect();

    Ok(good_options)
}
