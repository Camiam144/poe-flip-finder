use rusqlite::{Connection, Result, named_params};
use serde::Deserialize;

use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Deserialize)]
struct ExchangeRecord {
    #[serde(rename = "CurrencyExchangeSnapshotPairId")]
    pair_id: u64,
    #[serde(rename = "CurrencyExchangeSnapshotId")]
    snapshot_id: u64,
    #[serde(rename = "Volume")]
    volume: String,
    #[serde(rename = "CurrencyOne")]
    currency_one: CurrencyInfo,
    #[serde(rename = "CurrencyTwo")]
    currency_two: CurrencyInfo,
    #[serde(rename = "CurrencyOneData")]
    currency_one_data: CurrencyData,
    #[serde(rename = "CurrencyTwoData")]
    currency_two_data: CurrencyData,
}

#[derive(Debug, Deserialize)]
struct CurrencyInfo {
    #[serde(rename = "id")]
    id: u64,
    #[serde(rename = "itemId")]
    item_id: u64,
    #[serde(rename = "apiId")]
    api_id: String,
    #[serde(rename = "text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct CurrencyData {
    #[serde(rename = "RelativePrice")]
    relative_price: String,
    #[serde(rename = "StockValue")]
    stock_value: String,
    #[serde(rename = "VolumeTraded")]
    volume_traded: u64,
}

#[derive(Debug)]
struct ExchangeQueryResult {
    ts: u64,
    pair_id: u64,
    snapshot_id: u64,
    from_currency: String,
    to_currency: String,
    from_relative_price: f64,
    to_relative_price: f64,
    volume: f64,
}

fn new_schema(conn: &Connection) -> Result<()> {
    let schema = "DROP TABLE IF EXISTS exchange_rates;
    CREATE TABLE IF NOT EXISTS exchange_rates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    pair_id INTEGER NOT NULL,
    snapshot_id INTEGER NOT NULL,
    from_currency TEXT NOT NULL,
    to_currency TEXT NOT NULL,
    from_relative_price REAL,
    to_relative_price REAL,
    volume REAL)";

    conn.execute_batch(schema)?;
    Ok(())
}

fn insert_all_rows(records: &[ExchangeRecord], conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut insert_statement = tx.prepare("INSERT INTO exchange_rates
        (timestamp, pair_id, snapshot_id, from_currency, to_currency, from_relative_price, to_relative_price, volume)
        VALUES
        (:ts, :pair_id, :snapshot_id, :from_currency, :to_currency, :from_relative_price, :to_relative_price, :volume)")?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for entry in records {
            insert_statement.execute((
                &now,
                entry.pair_id,
                entry.snapshot_id,
                entry.currency_one.text.clone(),
                entry.currency_two.text.clone(),
                entry
                    .currency_one_data
                    .relative_price
                    .parse::<f64>()
                    .unwrap(),
                entry
                    .currency_two_data
                    .relative_price
                    .parse::<f64>()
                    .unwrap(),
                entry.volume.parse::<f64>().unwrap(),
            ))?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn main() -> Result<()> {
    let path = Path::new("data/response_1757636788999.json");
    let json_file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(json_file);

    let my_data: Vec<ExchangeRecord> = serde_json::from_reader(reader).unwrap();
    // println!("{:?}", my_data[0])
    let db_path = Path::new("data/exchangedata.db");
    let mut conn = Connection::open(db_path).expect("Couldn't open db");
    new_schema(&conn).expect("Couldn't make schema");

    insert_all_rows(&my_data, &mut conn)?;
    conn.close().expect("Couldn't close db: ");
    let conn2 = Connection::open(db_path).expect("Couldn't open db:");
    {
        let mut query = conn2.prepare("SELECT * FROM exchange_rates").unwrap();
        let elem_iter: Vec<ExchangeQueryResult> = query
            .query_map([], |row| {
                Ok(ExchangeQueryResult {
                    ts: row.get_unwrap(1),
                    pair_id: row.get_unwrap(2),
                    snapshot_id: row.get_unwrap(3),
                    from_currency: row.get_unwrap(4),
                    to_currency: row.get_unwrap(5),
                    from_relative_price: row.get_unwrap(6),
                    to_relative_price: row.get_unwrap(7),
                    volume: row.get_unwrap(8),
                })
            })
            .expect("Couldn't unwrap query result: ")
            .collect::<Result<Vec<ExchangeQueryResult>>>()?;
        for elem in &elem_iter[..=3] {
            println!("{:?}", elem)
        }
    }
    conn2.close().expect("Couldn't close connection: ");

    Ok(())
}
