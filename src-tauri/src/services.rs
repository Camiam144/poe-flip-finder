use chrono::{DateTime, Local};

use crate::{
    db::models::UpdateOutcome,
    errors::FrontendError,
    ggg_api::{
        self,
        models::{RawLeagueApiResponse, Realm},
    },
    logic::{
        self, arb_opp_is_profitable, build_and_populate_graph, get_all_arbitrage_options,
        models::{ArbitrageOpportunity, Market, TradingCurrencyRates},
    },
    sync, AppState,
};

fn get_cached_leagues(state: &AppState, realm: &Realm) -> Option<RawLeagueApiResponse> {
    let cached_leagues = state.league_cache.lock().unwrap();

    match realm {
        Realm::Poe1 => cached_leagues[0].clone(),
        Realm::Poe2 => cached_leagues[1].clone(),
    }
}

fn cache_leagues(state: &AppState, realm: &Realm, leagues: &RawLeagueApiResponse) {
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

/// Get the active leagues for a realm and cache them if they aren't cached.
pub async fn handle_current_leagues(
    state: &AppState,
    realm: &str,
) -> Result<RawLeagueApiResponse, FrontendError> {
    let api_realm = parse_realm(realm)?;

    let leagues = if let Some(val) = get_cached_leagues(state, &api_realm) {
        val
    } else {
        // Didn't have a cached value, store the new value
        let new_leagues = ggg_api::get_leagues_from_ggg(state, &api_realm)
            .await
            .map_err(FrontendError::from)?;

        cache_leagues(state, &api_realm, &new_leagues);
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

/// Get all of the opportunities available. We're going to rebuild the market list
/// here but the current leagues could be cached in the app state and then rebuilt
/// whenever the database update fires.
pub async fn handle_get_opportunities(
    state: &AppState,
    realm: &str,
    league: &str,
) -> Result<Vec<ArbitrageOpportunity>, FrontendError> {
    // TODO: Stopgap for now of just pulling most recent. Eventually I will want
    // to pull the past N steps and see what has been inefficient during that
    // whole time.
    //
    // All of this is copied code. Also I need to parse the realm on intake.
    // This is actually so bad, Just stuff the most recent values into some
    // kind of cache for now and then work out invalidation later.
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
    let all_markets: Vec<Market> = most_recent.iter().map(Market::from).collect();
    let rates = logic::get_base_prices(&all_markets);

    let graph = build_and_populate_graph(&all_markets);
    // This is hardcoded for now but will eventually be a parameter from the frontend.
    let max_depth: usize = 3;
    let options: Vec<ArbitrageOpportunity> = get_all_arbitrage_options(&graph, max_depth)
        .iter()
        .filter(|opp| arb_opp_is_profitable(opp, &rates))
        .cloned()
        .collect();

    let search_item = "Perfect Exalted Orb".to_string();
    let dbg_chaos_item = logic::get_edge_from_graph(
        &graph,
        &logic::models::TradingCurrencyType::Chaos,
        &logic::models::TradingCurrencyType::Other(search_item.clone()),
    );
    let dbg_item_div = logic::get_edge_from_graph(
        &graph,
        &logic::models::TradingCurrencyType::Other(search_item.clone()),
        &logic::models::TradingCurrencyType::Divine,
    );
    if let Some(c_i) = dbg_chaos_item {
        if let Some(i_d) = dbg_item_div {
            // println!("chaos -> item {:#?}", c_i);
            // println!("item -> div {:#?}", i_d);

            for o in options.iter().filter(|&opp| {
                opp.path[0] == logic::models::TradingCurrencyType::Chaos
                    && opp.path[1] == logic::models::TradingCurrencyType::Other(search_item.clone())
            }) {
                println!("{:#?}", &o);
                println!("arb rate: {:#?}", &o.high_ratios.iter().product::<f64>());
            }
        }
    }

    Ok(options)
}
