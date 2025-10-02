use reqwest::blocking::Client;
use std::path::Path;

mod api;
mod app;
mod logic;
mod models;

use crate::app::App;
use crate::models::api_models::{ExchangeRecord, ExchangeSnapshot};
use crate::models::logic_models::{TradingCurrencyRates, TradingCurrencyType};

fn main() {
    let mut app = App::default();
    app.set_volume(200.0);
    let client: Client = reqwest::blocking::Client::builder()
        .user_agent("poe-flip-finder/1.0-camiam144@gmail.com")
        .build()
        .expect("Couldn't build client: ");

    let most_recent_snapshot: ExchangeSnapshot =
        api::get_exchange_snapshot(&client).expect("Couldn't get newest snapshot: ");

    println!(
        "Most recent snapshot number: {}",
        &most_recent_snapshot.epoch
    );

    let data_path: &Path = Path::new("data");

    let cached_snapshots: Vec<std::fs::DirEntry> = logic::list_all_snapshots(data_path);

    let newest_pairs: Vec<ExchangeRecord> = logic::get_freshest_data(
        most_recent_snapshot.epoch,
        &cached_snapshots,
        &client,
        data_path,
    );

    // These are the base rates we need to compare against.
    let mut base_rates: TradingCurrencyRates = TradingCurrencyRates::default();

    logic::get_base_prices(&newest_pairs, &mut base_rates);

    println!("Divine to Exalt ratio {:?}", &base_rates.div_to_exalt);
    println!("Divine to Chaos ratio {:?}", &base_rates.div_to_chaos);
    println!("Chaos to Exalt ratio {:?}", &base_rates.chaos_to_exalt);

    // These will be configurable via cmd line at some point probably
    let min_vol: f64 = 7500.0;

    let valid_bridges: Vec<ExchangeRecord> = newest_pairs
        .into_iter()
        .filter(|exch| exch.volume >= min_vol && exch.is_valid_bridge())
        .collect();

    let (hub_to_bridge, bridge_to_hub) = logic::build_hub_bridge_maps(&valid_bridges);

    let mut potential_profits = logic::build_bridges(&hub_to_bridge, &bridge_to_hub);
    let min_profit_frac = 0.05;

    potential_profits.retain(|elem| logic::eval_profit(elem, &base_rates, min_profit_frac));
    potential_profits.sort_by(|a, b| b.3.abs().total_cmp(&a.3.abs()));

    let num_elements: usize = 15;
    // let end_idx = cmp::min(num_elements, potential_profits.len());

    // What I actually want here are either the highest margin items or the
    // top N items from each bridge
    // This is logic and the profit calc should be in logic.rs
    for currency in [TradingCurrencyType::Divine, TradingCurrencyType::Chaos] {
        println!("Top {} vals:", currency);
        let rate = match currency {
            TradingCurrencyType::Divine => base_rates.div_to_exalt,
            TradingCurrencyType::Chaos => base_rates.chaos_to_exalt,
            TradingCurrencyType::Exalt => 1.0 / base_rates.div_to_exalt,
            TradingCurrencyType::Other => 0.0,
        };
        let to_print = logic::get_top_items(&potential_profits, &currency, num_elements);
        for elem in to_print {
            println!(
                "{curr1:<7.7} -> {bridge:^25.25} -> {curr2:<8} | effective ratio: {ratio:>5.1} | profit {profit:>5.1} exalt",
                curr1 = elem.0,
                bridge = elem.1,
                curr2 = elem.2,
                ratio = elem.3,
                profit = elem.3 - rate
            )
        }
    }
}
