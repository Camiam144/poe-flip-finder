use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::models::{GGGBaseItem, UpdateOutcome};
use crate::db::transform::clean_raw_row;
use crate::ggg_api::get_specified_cxapi_from_ggg;
use crate::ggg_api::models::{GGGLeague, Realm};
use crate::AppState;

async fn fetch_and_save_hour(state: &AppState, realm: Realm, change_id: i64) -> Result<()> {
    dbg!(format!("Pulling data for {}", &change_id));
    let ggg_response = get_specified_cxapi_from_ggg(&state.http_client, realm, change_id).await?;
    let ggg_response_text = serde_json::to_string(&ggg_response)?;

    state
        .db_client
        .insert_data(change_id, realm, &ggg_response_text)
        .await?;
    Ok(())
}

fn previous_hour_change_id() -> Result<i64> {
    let past_hour = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    let rounded = (past_hour / 3600) * 3600 - 3600;

    Ok(rounded)
}

/// List hours between (start, end] inclusive of end but exclusive of start.
fn list_hrs_between(start: i64, end: i64) -> Vec<i64> {
    let start_vec = start + 3600;

    if start_vec > end {
        return Vec::new();
    }

    (start_vec..=end).step_by(3600).collect()
}

/// Update the database from the most recent recorded timestamp to the most recent available hour
/// Use timers to avoid getting rate limited
pub async fn get_update_data(state: &AppState, realm: Realm) -> anyhow::Result<UpdateOutcome> {
    //TODO: Better error handling
    let past_hour = previous_hour_change_id()?;
    dbg!(format!("Most recent hour should be {}", past_hour));

    let most_recent_entry = state.db_client.get_latest_raw(realm).await?;

    // TODO: Need an "out" to pull from a specific timestamp if we don't have history
    // for some reason.
    let most_recent_change_id = match most_recent_entry {
        Some(entry) => entry.change_id,
        None => return Err(anyhow!("No existing sync history for realm {realm:?}")),
    };
    dbg!(format!(
        "Most recent entry in db time is {}",
        most_recent_change_id
    ));

    let change_ids = list_hrs_between(most_recent_change_id, past_hour);
    if change_ids.is_empty() {
        dbg!("Database already up to date");
        return Ok(UpdateOutcome::NoUpdateNeeded);
    }

    dbg!(format!(
        "Pulling {} change IDS b/w {} and {}",
        &change_ids.len(),
        &most_recent_change_id,
        &past_hour
    ));

    // Run sequentially, the 1 request per 2 second rate limit is the bottleneck.
    for change_id in change_ids {
        fetch_and_save_hour(state, realm, change_id).await?;
    }

    dbg!("Database up to date");
    Ok(UpdateOutcome::Success)
}

fn map_items(original_map: &HashMap<String, GGGBaseItem>) -> HashMap<String, String> {
    original_map
        .iter()
        .filter_map(|m| {
            if m.1.name.is_some() {
                Some((m.0.to_owned(), m.1.name.clone().unwrap()))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>()
}

async fn load_item_mappings(realm: Realm) -> Result<HashMap<String, String>> {
    // TODO: These shouldn't be hardcoded eventually.
    let filepath = match realm {
        Realm::Poe1 => Path::new("./data/base_items.min.json"),
        Realm::Poe2 => Path::new("./data/base_items_poe2.min.json"),
    };

    let file = File::open(filepath)?;
    let buf = BufReader::new(file);

    let raw_mappings: HashMap<String, GGGBaseItem> = serde_json::from_reader(buf)?;

    Ok(map_items(&raw_mappings))
}

pub async fn clean_raw_responses(
    state: &AppState,
    realm: Realm,
    leagues: &[GGGLeague],
) -> anyhow::Result<()> {
    // This needs to determine which leagues we're going to filter, then load
    // those raw responses in from the db and clean them. I also think
    // this means we need a flag in the DB to determine if we've already cleaned
    // the response or not.

    // Get up to 500 records, will need to check if we should keep going
    // TODO: Put in check, if we hit 500 records we need to go again.
    let unprocessed_rows = state.db_client.get_unprocessed_rows(realm).await?;

    if unprocessed_rows.is_empty() {
        dbg!("Nothing to process");
        return Ok(());
    } else {
        dbg!("Cleaning {} rows", &unprocessed_rows.len());
    }

    // If we have records, we need to load our item map
    let item_map = load_item_mappings(realm).await?;
    let mut rows_processed: usize = 0;

    for row in unprocessed_rows {
        let cleaned_rows = clean_raw_row(&row, &item_map, leagues)?;

        // Push parsed records
        state
            .db_client
            .insert_multiple_processed_rows(&cleaned_rows, realm)
            .await?;

        // Mark row as parsed
        state
            .db_client
            .mark_record_as_processed(row.change_id, realm)
            .await?;

        rows_processed += cleaned_rows.len();
    }
    dbg!("Added {} processed rows", rows_processed);
    Ok(())
}

/// Run the whole ELT pipeline on new data
pub async fn update_and_run_elt(state: &AppState, realm: Realm) -> anyhow::Result<UpdateOutcome> {
    get_update_data(state, realm).await?;

    {
        let cached_leagues = state.league_cache.lock().unwrap();

        let maybe_cache = match realm {
            Realm::Poe1 => cached_leagues[0].as_ref(),
            Realm::Poe2 => cached_leagues[1].as_ref(),
        };

        if maybe_cache.is_none() {
            return Err(anyhow!("No leagues in league cache"));
        }
    }

    let bad_names = ["SSF", "HC", "Ruthless", "Hardcore"];

    let active_leagues: Vec<GGGLeague> = {
        let cached_leagues = state.league_cache.lock().unwrap();
        let cache = match realm {
            Realm::Poe1 => cached_leagues[0].as_ref(),
            Realm::Poe2 => cached_leagues[1].as_ref(),
        };
        cache
            .unwrap()
            .leagues
            .iter()
            .filter(|&l| {
                l.is_active() && l.event.is_none() && !bad_names.iter().any(|s| l.id.contains(s))
            })
            .cloned()
            .collect()
    };

    // dbg!("Cleaning data for {}", &active_leagues);
    clean_raw_responses(state, realm, &active_leagues).await?;
    Ok(UpdateOutcome::Success)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_list_hours() {
        let start_time: i64 = 1787252400;
        let should_be_3 = start_time + 3600 * 3;
        let should_be_empty = start_time - 3600;
        assert_eq!(
            list_hrs_between(start_time, should_be_3),
            vec![
                start_time + 3600,
                start_time + 3600 * 2,
                start_time + 3600 * 3
            ]
        );
        assert_eq!(
            list_hrs_between(start_time, should_be_empty),
            Vec::<i64>::new()
        );
    }

    #[test]
    fn test_map_items() {
        let item = GGGBaseItem {
            name: Some("Exalted Orb".to_string()),
            ..Default::default()
        };

        let map: HashMap<String, GGGBaseItem> = HashMap::from([(
            "Metadata/Items/Currency/CurrencyAddModToRare".to_string(),
            item,
        )]);
        let final_map: HashMap<String, String> = HashMap::from([(
            "Metadata/Items/Currency/CurrencyAddModToRare".to_string(),
            "Exalted Orb".to_string(),
        )]);

        assert_eq!(final_map, map_items(&map));
    }
}
