use rusqlite::{Connection, Result};

use std::path::Path;

mod models;
use models::ExchangeRecord;
mod db;
use db::{insert_all_rows, new_schema};

use crate::db::get_most_recent_entry;

fn main() -> Result<()> {
    let path: &Path = Path::new("data/response_1757636788999.json");
    let json_file: std::fs::File = std::fs::File::open(path).unwrap();
    let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(json_file);

    let all_pairs: Vec<ExchangeRecord> = serde_json::from_reader(reader).unwrap();
    // println!("{:?}", my_data[0])
    // These will be configurable via cmd line at some point probably

    let min_vol: f64 = 100.0;
    let filtered_trades: Vec<ExchangeRecord> = all_pairs
        .into_iter()
        .filter(|exch| exch.volume >= min_vol && exch.is_valid_bridge())
        .collect();
    let db_path: &Path = Path::new("data/exchangedata.db");
    let mut conn: Connection = Connection::open(db_path).expect("Couldn't open db");
    new_schema(&conn).expect("Couldn't make schema");

    insert_all_rows(&filtered_trades, &mut conn)?;
    conn.close().expect("Couldn't close db: ");
    let mut conn2: Connection = Connection::open(db_path).expect("Couldn't open db:");
    let query_res: Vec<models::ExchangeQueryResult> = get_most_recent_entry(&mut conn2);
    conn2.close().expect("Couldn't close connection: ");
    for elem in &query_res[..=2] {
        println!("{:?}", elem)
    }

    Ok(())
}
