// use rusqlite::{Connection, Result};

use std::path::Path;

// mod db;
mod api;
mod logic;
mod models;

use models::api_models::{ExchangeRecord, ExchangeSnapshot};
use models::logic_models::TradingCurrencyRates;
use reqwest::blocking::Client;

use crate::api::get_newest_snapshot_pairs;
use crate::logic::cache_to_disk;

fn main() {
    let client: Client = reqwest::blocking::Client::builder()
        .user_agent("poe-flip-finder/1.0-camiam144@gmail.com")
        .build()
        .expect("Couldn't build client: ");
    let most_recent_snapshot: ExchangeSnapshot = api::get_exchange_snapshot(&client).unwrap();
    println!(
        "Most recent snapshot number: {}",
        &most_recent_snapshot.epoch
    );
    let data_path = Path::new("data");
    let cached_snapshots: Vec<std::fs::DirEntry> = logic::list_all_snapshots(data_path);
    let mut newest_pairs: Vec<ExchangeRecord> = Vec::new();
    let have_most_recent_snapshot: bool =
        logic::check_if_snapshot_exists(most_recent_snapshot.epoch, cached_snapshots);
    if have_most_recent_snapshot {
        println!(
            "We have the most recent snapshot, number {}",
            &most_recent_snapshot.epoch
        );
        let filename = format!("response_{}.json", &most_recent_snapshot.epoch);
        let json_file: std::fs::File =
            std::fs::File::open(data_path.join(filename)).expect("Couldn't open json: ");
        let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(json_file);
        newest_pairs = serde_json::from_reader(reader).expect("Couldn't deserialize json: ");
    } else {
        println!("We do not have the most recent snapshot, getting newest pairs");
        newest_pairs =
            get_newest_snapshot_pairs(&client).expect("Couldn't get newest set of pairs: ");
        // After we get them cache them to disk so we don't get banned from the api
        let filename = format!("response_{}.json", &most_recent_snapshot.epoch);
        cache_to_disk(&newest_pairs, data_path, &filename)
            .expect("Couldn't cache snapshot to disk:");
    }

    // let path: &Path = Path::new("data/response_1757636788999.json");
    // let json_file: std::fs::File = std::fs::File::open(path).unwrap();
    // let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(json_file);
    //
    // let all_pairs: Vec<ExchangeRecord> = serde_json::from_reader(reader).unwrap();
    // println!("{:?}", my_data[0])
    // These are the base rates we need to compare against.
    let mut base_rates: TradingCurrencyRates = TradingCurrencyRates::default();
    logic::get_base_prices(&newest_pairs, &mut base_rates);
    println!("Divine to Exalt ratio {:?}", base_rates.div_to_exalt);
    println!("Divine to Chaos ratio {:?}", base_rates.div_to_chaos);
    println!("Chaos to Exalt ratio {:?}", base_rates.chaos_to_exalt);

    // These will be configurable via cmd line at some point probably
    let min_vol: f64 = 100.0;
    let valid_bridges: Vec<ExchangeRecord> = newest_pairs
        .into_iter()
        .filter(|exch| exch.volume >= min_vol && exch.is_valid_bridge())
        .collect();

    for elem in &valid_bridges[..=2] {
        println!(
            "Curr 1: {} Curr 2: {}, RP 1: {}, RP 2: {}",
            elem.currency_one.text,
            elem.currency_two.text,
            elem.currency_one_data.relative_price,
            elem.currency_two_data.relative_price
        )
    }
    // Ok(())
}
