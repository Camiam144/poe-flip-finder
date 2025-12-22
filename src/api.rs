use anyhow::{Context, Ok, Result};
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

use crate::auth;
use crate::auth::AuthorizedScopes;
use crate::models::api_models::{
    ExchangeRecord, ExchangeSnapshot, GGGLeagueList, GGGMarket, Market, RawCxApiResponse,
};

/// Get the entire Cxapi dump from the past hour. The current hour does not yet
/// have information.
pub async fn get_most_recent_cxapi(client: &Client) -> Result<GGGMarket> {
    let past_hour = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600;

    let data_path = Path::new("data/markets/");
    let filename = format!("markets_{}.json", past_hour);
    let market_snapshots = list_all_snapshots(data_path)?;

    if check_if_snapshot_exists(past_hour, &market_snapshots)? {
        println!("Getting market {} from cache.", past_hour);
        let json_file: fs::File = fs::File::open(data_path.join(filename))?;
        let reader: io::BufReader<fs::File> = io::BufReader::new(json_file);
        serde_json::from_reader(reader).context("Couldn't parse json from file.")
    } else {
        // This part runs if we don't have the data cached
        let token = auth::get_api_token(&AuthorizedScopes::Cxapi).await?;
        let base_url = "https://api.pathofexile.com/currency-exchange/poe2/";
        let url = format!("{}{}", base_url, past_hour);

        let response = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .send()
            .await?;

        let raw_data: RawCxApiResponse = response.json().await?;

        // println!("GGG Response {:#?}", this_json);

        let parsed_markets: Result<Vec<Market>, anyhow::Error> = raw_data
            .markets
            .into_iter()
            .map(|m| Market::from_raw(m).context("Couldn't parse a market response."))
            .collect();
        let all_markets = GGGMarket {
            next_change_id: raw_data.next_change_id,
            markets: parsed_markets?,
        };
        let file_path = data_path.join(filename);
        cache_to_disk(&all_markets, &file_path).context("Couldn't cache snapshot to disk:")?;

        Ok(all_markets)
    }
}

/// Get all current leagues from GGG's API. This can probably be cached and
/// updated as needed instead of pinging it every time, but for now we will
/// continue to ping ever time
pub async fn get_leagues(client: &Client, realm: &str) -> Result<GGGLeagueList> {
    let token = auth::get_api_token(&AuthorizedScopes::Leagues).await?;
    let url = "https://api.pathofexile.com/league";
    let params = [("realm", realm)];
    let url = reqwest::Url::parse_with_params(url, &params)?;

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
        .send()
        .await?;

    println!("{:?}", response);

    let result: GGGLeagueList = response.json().await?;

    Ok(result)
}

// TODO: Eventually want to pass the league into these requests
pub async fn get_exchange_snapshot(client: &Client) -> Result<ExchangeSnapshot> {
    let url = "https://poe2scout.com/api/currencyExchangeSnapshot?league=Fate%20of%20the%20Vaal";

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
    if check_if_snapshot_exists(most_recent_epoch, list_cached_snapshots)? {
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
    let underscore_idx = snapshot_name
        .find("_")
        .expect("Name must contain an underscore");
    let dot_idx = snapshot_name
        .find(".")
        .expect("Name must contain a \".\" separating the file ext");
    snapshot_name[underscore_idx + 1..dot_idx].parse::<u64>()
}

pub fn list_all_snapshots(path: &Path) -> Result<Vec<fs::DirEntry>> {
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

pub fn check_if_snapshot_exists(
    snapshot_to_check: u64,
    snapshot_list: &[fs::DirEntry],
) -> Result<bool> {
    // TODO: Error handling
    for snapshot in snapshot_list {
        if snapshot_to_check
            == get_snapshot_number_from_name(
                snapshot
                    .file_name()
                    .to_str()
                    .context("Couldn't get filename")?,
            )
            .context("Couldn't get snapshot number")?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn cache_to_disk(data: &impl Serialize, file_path: &Path) -> Result<()> {
    let file = fs::File::create(file_path)?;
    let writer = io::BufWriter::new(file);

    serde_json::to_writer(writer, data)?;
    Ok(())
}
