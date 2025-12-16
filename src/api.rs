use anyhow::Context;
use anyhow::Result;
use reqwest::Client;
use reqwest::Url;
use reqwest::header::AUTHORIZATION;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

use crate::auth;
use crate::models::api_models::{ExchangeRecord, ExchangeSnapshot};

// TODO: Eventually want to pass the league into these requests

pub async fn test_ggg_api(client: &Client) -> Result<()> {
    // TODO: find a way to get the unix timestamp code of the most recent hour
    // gotta do oauth here
    let token = auth::get_cxapi_cred().await?;

    println!("Pinging GGG for new data");
    let url = Url::parse("https://api.pathofexile.com/currency-exchange/poe2/1765839600").unwrap();
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
        .send()
        .await?;

    let this_json: Value = response.json().await?;

    // TODO: this needs to get parsed into a struct and cached to disk like the others
    // then need to filter on league

    println!("GGG Response {:#?}", this_json);

    Ok(())
}

pub async fn get_exchange_snapshot(client: &Client) -> Result<ExchangeSnapshot> {
    let url = "https://poe2scout.com/api/currencyExchangeSnapshot?league=Rise%20of%20the%20Abyssal";

    let response = client.get(url).send().await?.json().await?;
    Ok(response)
}

pub async fn get_newest_snapshot_pairs(client: &Client) -> Result<Vec<ExchangeRecord>> {
    let url =
        "https://poe2scout.com/api/currencyExchange/SnapshotPairs?league=Fate%20of%20the%Vaal";

    let response = client.get(url).send().await?.json().await?;
    Ok(response)
}

pub async fn get_freshest_data(
    most_recent_epoch: u64,
    list_cached_snapshots: &[fs::DirEntry],
    client: &Client,
    data_path: &Path,
) -> Result<Vec<ExchangeRecord>> {
    // TODO: error handling.
    if check_if_snapshot_exists(most_recent_epoch, list_cached_snapshots) {
        println!(
            "We have the most recent snapshot, number {}",
            &most_recent_epoch
        );
        let filename = format!("response_{}.json", &most_recent_epoch);
        let json_file: fs::File = fs::File::open(data_path.join(filename))?;
        let reader: io::BufReader<fs::File> = io::BufReader::new(json_file);
        serde_json::from_reader(reader).context("Couldn't parse json from file.")
    } else {
        println!("We do not have the most recent snapshot, getting newest pairs");
        let fresh_data = get_newest_snapshot_pairs(client).await?;
        // After we get them cache them to disk so we don't get banned from the api
        let filename = format!("response_{}.json", &most_recent_epoch);
        let file_path = data_path.join(filename);
        cache_to_disk(&fresh_data, &file_path).context("Couldn't cache snapshot to disk:")?;
        Ok(fresh_data)
    }
}

pub fn get_snapshot_number_from_name(snapshot_name: &str) -> Result<u64, std::num::ParseIntError> {
    let underscore_idx = snapshot_name.find("_").unwrap();
    let dot_idx = snapshot_name.find(".").unwrap();
    snapshot_name[underscore_idx + 1..dot_idx].parse::<u64>()
}

pub fn list_all_snapshots(path: &Path) -> Result<Vec<fs::DirEntry>, io::Error> {
    let mut out_vec: Vec<fs::DirEntry> = Vec::new();
    for entry_result in fs::read_dir(path)? {
        let entry = entry_result?;
        // This bit stolen from BurntSushi on the rust forums
        if entry.path().is_file()
            && entry
                .path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        {
            out_vec.push(entry);
        }
    }
    Ok(out_vec)
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

pub fn cache_to_disk(data: &impl Serialize, file_path: &Path) -> Result<()> {
    let file = fs::File::create(file_path)?;
    let writer = io::BufWriter::new(file);

    serde_json::to_writer(writer, data)?;
    Ok(())
}
