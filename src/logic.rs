use std::fs::{self, DirEntry, File};
use std::io::BufWriter;
use std::path::Path;

use serde::Serialize;

use crate::models::api_models::ExchangeRecord;
use crate::models::logic_models::{TradingCurrencyRates, TradingCurrencyType};

pub fn get_base_prices(records: &[ExchangeRecord], rates: &mut TradingCurrencyRates) {
    for record in records {
        let currency_pair = record.trading_currency();
        match currency_pair {
            (TradingCurrencyType::Divine, TradingCurrencyType::Exalt) => {
                rates.div_to_exalt = record.currency_one_data.relative_price
            }
            (TradingCurrencyType::Chaos, TradingCurrencyType::Exalt) => {
                rates.chaos_to_exalt = record.currency_one_data.relative_price
            }
            (_, _) => {}
        }
    }
    rates.div_to_chaos = rates.div_to_exalt / rates.chaos_to_exalt
}

pub fn list_all_snapshots(path: &Path) -> Vec<DirEntry> {
    let paths = fs::read_dir(path).unwrap();
    let mut out_vec: Vec<DirEntry> = vec![];
    for file_name in paths.filter(|f| {
        f.as_ref().unwrap().path().is_file()
            && f.as_ref().unwrap().path().extension().unwrap() == "json"
    }) {
        out_vec.push(file_name.unwrap());
    }
    out_vec
}

fn get_snapshot_number_from_name(snapshot_name: String) -> Option<u64> {
    let underscore_idx = snapshot_name.find("_")?;
    let dot_idx = snapshot_name.find(".")?;
    snapshot_name[underscore_idx + 1..dot_idx]
        .parse::<u64>()
        .ok()
}

pub fn check_if_snapshot_exists(newest_snapshot: u64, snapshot_list: Vec<DirEntry>) -> bool {
    for snapshot in snapshot_list {
        if newest_snapshot
            == get_snapshot_number_from_name(snapshot.file_name().into_string().unwrap()).unwrap()
        {
            return true;
        }
    }
    false
}

pub fn cache_to_disk(
    data: &impl Serialize,
    path_dir: &Path,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = path_dir.join(filename);
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, data)?;
    Ok(())
}
// What do we have to do once we have the values?
// I think we're going to iterate over the filtered vector one time
// Maybe we can do it after the filter step.
// for each record we get the trading currency rate and we compare
// the absolute difference of that rate to the matching tradingcurrencyrate value.
// I think we then have another struct that's like {diff, ExchangeRecord} and
// push that into a new vec. Sort that vec by absolute difference, then we can
// pretty print the output? We need to compute the expected return at some point.
