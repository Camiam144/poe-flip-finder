use anyhow::{anyhow, Result};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ggg_api::{get_specified_cxapi_from_ggg, Realm};
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
pub async fn get_update_data(state: &AppState, realm: Realm) -> anyhow::Result<()> {
    //TODO: Better error handling
    let past_hour = previous_hour_change_id()?;
    dbg!(format!("Most recent hour should be {}", past_hour));

    let most_recent_entry = state.db_client.get_latest(realm).await?;

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
        return Ok(());
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
    anyhow::Ok(())
}

pub async fn clean_raw_responses() {
    // This needs to determine which realm we're gonna clean and which leagues we're
    // going to filter, then load those raw responses in from the db and clean them. I also think
    // this means we need a flag in the DB to determine if we've already cleaned
    // the response or not.
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
}
