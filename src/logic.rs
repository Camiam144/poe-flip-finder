use std::collections::HashMap;
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

fn get_snapshot_number_from_name(snapshot_name: &str) -> Option<u64> {
    let underscore_idx = snapshot_name.find("_")?;
    let dot_idx = snapshot_name.find(".")?;
    snapshot_name[underscore_idx + 1..dot_idx]
        .parse::<u64>()
        .ok()
}

pub fn check_if_snapshot_exists(newest_snapshot: u64, snapshot_list: &[DirEntry]) -> bool {
    for snapshot in snapshot_list {
        if newest_snapshot
            == get_snapshot_number_from_name(snapshot.file_name().to_str().unwrap()).unwrap()
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

pub fn build_hub_bridge_maps(
    records: &[ExchangeRecord],
) -> (
    HashMap<(TradingCurrencyType, String), f64>,
    HashMap<(String, TradingCurrencyType), f64>,
) {
    // Build our lookup tables here so it's faster to scan every single
    // combination instead of looping through the vec of records a bazillion times
    let mut hub_to_bridge = HashMap::new();
    let mut bridge_to_hub = HashMap::new();

    for record in records {
        if let Some((hub, hub_ex, bridge_str, bridge_ex)) = record.hub_bridge_price() {
            let hub_per_bridge_ratio = hub_ex / bridge_ex;
            hub_to_bridge.insert((hub.clone(), bridge_str.clone()), hub_per_bridge_ratio);
            bridge_to_hub.insert((bridge_str, hub), hub_per_bridge_ratio.recip());
        }
    }
    (hub_to_bridge, bridge_to_hub)
}

pub fn find_profit(
    hub_to_bridge: &HashMap<(TradingCurrencyType, String), f64>,
    bridge_to_hub: &HashMap<(String, TradingCurrencyType), f64>,
    margin_pct: f64,
) -> Vec<(TradingCurrencyType, String, TradingCurrencyType, f64)> {
    let mut results = Vec::new();

    let hubs = [
        TradingCurrencyType::Exalt,
        TradingCurrencyType::Chaos,
        TradingCurrencyType::Divine,
    ];

    for &first_hub in &hubs {
        for &second_hub in &hubs {
            if first_hub == second_hub {
                continue;
            }
            // Now we grind through everything

            for ((hub_one, bridge), rate_one) in
                hub_to_bridge.iter().filter(|((h, _), _)| *h == first_hub)
            {
                if let Some(rate_two) = bridge_to_hub.get(&(bridge.clone(), second_hub)) {
                    // If we have a rate two, this means we went A -> X -> B
                    let cost = rate_one * rate_two;
                    if cost > 1.0 + margin_pct {
                        results.push((first_hub, bridge.clone(), second_hub, cost));
                    }
                }
            }
        }
    }
    results
}
