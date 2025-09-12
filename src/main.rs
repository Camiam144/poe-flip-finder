use rusqlite::{Connection, Result};

use std::path::Path;

mod models;
use models::ExchangeRecord;
mod db;
use db::{insert_all_rows, new_schema};

use crate::db::get_most_recent_entry;

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
    let mut conn2 = Connection::open(db_path).expect("Couldn't open db:");
    let query_res = get_most_recent_entry(&mut conn2);
    conn2.close().expect("Couldn't close connection: ");
    for elem in &query_res[..=2] {
        println!("{:?}", elem)
    }

    Ok(())
}
