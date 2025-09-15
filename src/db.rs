use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, Result};

use crate::models::{ExchangeQueryResult, ExchangeRecord};

pub fn new_schema(conn: &Connection) -> Result<()> {
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

pub fn insert_all_rows(records: &[ExchangeRecord], conn: &mut Connection) -> Result<()> {
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
                entry.volume,
            ))?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_most_recent_entry(conn: &mut Connection) -> Vec<ExchangeQueryResult> {
    let mut query = conn.prepare("SELECT * FROM exchange_rates").unwrap();
    let elem_iter = query
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
        .expect("Couldn't unwrap query result: ");
    elem_iter
        .collect::<Result<Vec<ExchangeQueryResult>>>()
        .expect("Couldn't collect mapped rows: ")
}
