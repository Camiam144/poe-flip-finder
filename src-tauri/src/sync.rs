use anyhow::anyhow;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ggg_api::get_specified_cxapi_from_ggg;
use crate::AppState;

/// Update the database from the most recent recorded timestamp to the most recent available hour
/// Use timers to avoid getting rate limited
pub async fn get_update_data(state: &AppState, realm: &str) -> anyhow::Result<()> {
    let past_hour = ((SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 3600)
        * 3600
        - 3600) as i64;
    dbg!(format!("Most recent hour should be {}", past_hour));

    let most_recent_entry = state.db_client.get_latest(realm).await?;
    // TODO: If most_recent_entry is none, we need to either throw an error or
    // pull data from the beginning?

    // Build a list of which values we need
    if let Some(most_recent) = most_recent_entry {
        dbg!(format!(
            "Most recent entry in db time is {}",
            most_recent.change_id
        ));

        if most_recent.change_id == past_hour {
            dbg!(format!(
                "Most recent time in db {} is equal to past hour {}",
                most_recent.change_id, past_hour
            ));
            return anyhow::Ok(());
        }
        let mut change_ids = Vec::new();

        // Do this to build the list so tokio can run them simultaneously
        for timestamp in ((most_recent.change_id + 3600)..=past_hour).step_by(3600) {
            change_ids.push(timestamp);
        }
        dbg!(format!("Pulling {} change IDS", &change_ids.len()));
        dbg!(&change_ids);

        // Run sequentially, the 1 request per 2 second rate limit is the bottleneck.
        for change_id in change_ids {
            dbg!(format!("Pulling data for {}", &change_id));
            let ggg_response =
                get_specified_cxapi_from_ggg(&state.http_client, realm, change_id).await?;
            let ggg_response_text = serde_json::to_string(&ggg_response)?;

            state
                .db_client
                .insert_data(change_id, realm, &ggg_response_text)
                .await?;
        }
    } else {
        dbg!("No most recent entry, you need to handle this path");
        return Err(anyhow!(
            "No most recent data for realm {} you need to figure it out",
            realm
        ));
    }

    dbg!("Database up to date");
    anyhow::Ok(())
}
