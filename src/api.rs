use reqwest::blocking::Client;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::Path;

use crate::models::api_models::{ExchangeRecord, ExchangeSnapshot};

// TODO: Eventually want to pass the league into these requests

pub fn get_exchange_snapshot(client: &Client) -> reqwest::Result<ExchangeSnapshot> {
    let url = "https://poe2scout.com/api/currencyExchangeSnapshot?league=Rise%20of%20the%20Abyssal";

    let response = client.get(url).send()?;
    response.error_for_status_ref()?;
    response.json()
}

pub fn get_newest_snapshot_pairs(client: &Client) -> reqwest::Result<Vec<ExchangeRecord>> {
    let url =
        "https://poe2scout.com/api/currencyExchange/SnapshotPairs?league=Rise%20of%20the%20Abyssal";

    let response = client.get(url).send()?;
    response.error_for_status_ref()?;
    response.json()
}

pub fn get_freshest_data(
    most_recent_epoch: u64,
    list_cached_snapshots: &[fs::DirEntry],
    client: &Client,
    data_path: &Path,
) -> Vec<ExchangeRecord> {
    // TODO: error handling.
    if check_if_snapshot_exists(most_recent_epoch, list_cached_snapshots) {
        println!(
            "We have the most recent snapshot, number {}",
            &most_recent_epoch
        );
        let filename = format!("response_{}.json", &most_recent_epoch);
        let json_file: fs::File =
            fs::File::open(data_path.join(filename)).expect("Couldn't open json: ");
        let reader: io::BufReader<fs::File> = io::BufReader::new(json_file);
        serde_json::from_reader(reader).expect("Couldn't deserialize json: ")
    } else {
        println!("We do not have the most recent snapshot, getting newest pairs");
        let fresh_data =
            get_newest_snapshot_pairs(client).expect("Couldn't get newest set of pairs: ");
        // After we get them cache them to disk so we don't get banned from the api
        let filename = format!("response_{}.json", &most_recent_epoch);
        cache_to_disk(&fresh_data, data_path, &filename).expect("Couldn't cache snapshot to disk:");
        fresh_data
    }
}

pub fn get_snapshot_number_from_name(snapshot_name: &str) -> Result<u64, std::num::ParseIntError> {
    let underscore_idx = snapshot_name.find("_").unwrap();
    let dot_idx = snapshot_name.find(".").unwrap();
    snapshot_name[underscore_idx + 1..dot_idx].parse::<u64>()
}
pub fn list_all_snapshots(path: &Path) -> Vec<fs::DirEntry> {
    let paths = fs::read_dir(path).unwrap();
    let mut out_vec: Vec<fs::DirEntry> = vec![];
    for file_name in paths.filter(|f| {
        f.as_ref().unwrap().path().is_file()
            && f.as_ref().unwrap().path().extension().unwrap() == "json"
    }) {
        out_vec.push(file_name.unwrap());
    }
    out_vec
}

pub fn check_if_snapshot_exists(newest_snapshot: u64, snapshot_list: &[fs::DirEntry]) -> bool {
    // TODO: Error handling
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
    let file = fs::File::create(file_path)?;
    let writer = io::BufWriter::new(file);

    serde_json::to_writer(writer, data)?;
    Ok(())
}
