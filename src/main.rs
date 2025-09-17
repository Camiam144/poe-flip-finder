use rusqlite::{Connection, Result};

use std::path::Path;

// mod db;
mod api;
mod logic;
mod models;
// use db::{get_most_recent_entry, insert_all_rows, new_schema};
use api::get_exchange_snapshot;
use logic::get_base_prices;
use models::{ExchangeRecord, ExchangeSnapshot, TradingCurrencyRates};

fn main() -> Result<()> {
    let most_recent_snapshot: ExchangeSnapshot = get_exchange_snapshot().unwrap();
    println!("{}", &most_recent_snapshot.epoch);
    let path: &Path = Path::new("data/response_1757636788999.json");
    let json_file: std::fs::File = std::fs::File::open(path).unwrap();
    let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(json_file);

    let all_pairs: Vec<ExchangeRecord> = serde_json::from_reader(reader).unwrap();
    // println!("{:?}", my_data[0])
    // These are the base rates we need to compare against.
    let mut base_rates: TradingCurrencyRates = TradingCurrencyRates::default();
    get_base_prices(&all_pairs, &mut base_rates);
    println!("Divine to Exalt ratio {:?}", base_rates.div_to_exalt);
    println!("Divine to Chaos ratio {:?}", base_rates.div_to_chaos);
    println!("Chaos to Exalt ratio {:?}", base_rates.chaos_to_exalt);

    // These will be configurable via cmd line at some point probably
    let min_vol: f64 = 100.0;
    let valid_bridges: Vec<ExchangeRecord> = all_pairs
        .into_iter()
        .filter(|exch| exch.volume >= min_vol && exch.is_valid_bridge())
        .collect();
    // let db_path: &Path = Path::new("data/exchangedata.db");
    // let mut conn: Connection = Connection::open(db_path).expect("Couldn't open db");
    // new_schema(&conn).expect("Couldn't make schema");
    //
    // insert_all_rows(&filtered_trades, &mut conn)?;
    // conn.close().expect("Couldn't close db: ");
    // let mut conn2: Connection = Connection::open(db_path).expect("Couldn't open db:");
    // let query_res: Vec<models::ExchangeQueryResult> = get_most_recent_entry(&mut conn2);
    // conn2.close().expect("Couldn't close connection: ");
    // for elem in &query_res[..=2] {
    //     println!("{:?}", elem)
    // }
    for elem in &valid_bridges[..=2] {
        println!(
            "Curr 1: {} Curr 2: {}, RP 1: {}, RP 2: {}",
            elem.currency_one.text,
            elem.currency_two.text,
            elem.currency_one_data.relative_price,
            elem.currency_two_data.relative_price
        )
    }
    Ok(())
}
