//! This module holds some convenience functions for interacting with the on-disk
//! Sqlite database. This will hold/cache some recent data so we can do some things
//! with a little bit of history. Might be able to catch some explosive price
//! changes in near-real time (with an hour or two lag) or overnight streamer-driven
//! changes.
pub mod models;
pub mod transform;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{migrate, query, query_as, QueryBuilder, Sqlite};
use std::path::PathBuf;

use crate::db::models::{DbRow, ParsedDbRow};
use crate::ggg_api::models::Realm;

static MIGRATOR: migrate::Migrator = migrate!("./migrations");

const SQLITE_BIND_LIMIT: usize = 32766;

#[derive(Debug)]
pub struct DbClient {
    pool: SqlitePool,
}

impl DbClient {
    /// Initialize the database connection
    /// if the database does not exist, create it and run the migration
    pub async fn try_from_path(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;
        MIGRATOR.run(&pool).await?;
        Ok(DbClient { pool })
    }

    /// Insert data into the database
    /// Does not check for duplicate data, that must be done elsewhere
    pub async fn insert_data(
        &self,
        change_id: i64,
        realm: &Realm,
        payload: &str,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let _ = query(
            "INSERT INTO data (change_id, game_version, payload, parsed_bool)
        VALUES ($1, $2, $3, $4)
    ",
        )
        .bind(change_id)
        .bind(realm.to_string())
        .bind(payload)
        .bind(0)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Get a specific change_id and game version
    pub async fn get_specific_change_id(
        &self,
        change_id: i64,
        realm: &Realm,
    ) -> Result<Option<DbRow>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let results: Option<DbRow> = query_as(
            "SELECT id, change_id, game_version, payload, parsed_bool
        FROM data
        WHERE change_id = $1
        AND game_version = $2;",
        )
        .bind(change_id)
        .bind(realm.to_string())
        .fetch_optional(&mut *conn)
        .await?;

        Ok(results)
    }

    /// Get up to 500 unprocessed payloads for a given realm, limited to 500 values for
    /// memory issues, may need to run multiple times if it's been weeks/months since
    /// last update (24 points per day).
    pub async fn get_unprocessed_rows(&self, realm: &Realm) -> Result<Vec<DbRow>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let results: Vec<DbRow> = query_as(
            "SELECT id, change_id, game_version, payload, parsed_bool
        FROM data
        WHERE game_version = $1
        AND parsed_bool = 0
        LIMIT 500;",
        )
        .bind(realm.to_string())
        .fetch_all(&mut *conn)
        .await?;

        Ok(results)
    }

    pub async fn mark_record_as_processed(
        &self,
        change_id: i64,
        realm: &Realm,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let _ = query(
            "UPDATE data SET parsed_bool = 1
            WHERE change_id = $1
            AND game_version = $2;",
        )
        .bind(change_id)
        .bind(realm.to_string())
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Insert a single processed row into the appropriate realm's table
    pub async fn insert_processed_row(
        &self,
        parsed_row: &ParsedDbRow,
        realm: &Realm,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let table = get_table_name(realm);

        let query_str = format!(
            "INSERT INTO {} (
            change_id,
            league,
            market_id,
            currency_a_name_ggg,
            currency_b_name_ggg,
            currency_a_name_common,
            currency_b_name_common,
            volume_traded_currency_a,
            volume_traded_currency_b,
            lowest_stock_currency_a,
            lowest_stock_currency_b,
            highest_stock_currency_a,
            highest_stock_currency_b,
            lowest_ratio_currency_a,
            lowest_ratio_currency_b,
            highest_ratio_currency_a,
            highest_ratio_currency_b,
            is_hub_curr_a,
            is_hub_curr_b)
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)",
            table
        );

        let _ = query(&query_str)
            .bind(parsed_row.change_id)
            .bind(&parsed_row.league)
            .bind(&parsed_row.market_id)
            .bind(&parsed_row.currency_a_name_ggg)
            .bind(&parsed_row.currency_b_name_ggg)
            .bind(&parsed_row.currency_a_name_common)
            .bind(&parsed_row.currency_b_name_common)
            .bind(parsed_row.volume_traded_currency_a)
            .bind(parsed_row.volume_traded_currency_b)
            .bind(parsed_row.lowest_stock_currency_a)
            .bind(parsed_row.lowest_stock_currency_b)
            .bind(parsed_row.highest_stock_currency_a)
            .bind(parsed_row.highest_stock_currency_b)
            .bind(parsed_row.lowest_ratio_currency_a)
            .bind(parsed_row.lowest_ratio_currency_b)
            .bind(parsed_row.highest_ratio_currency_a)
            .bind(parsed_row.lowest_ratio_currency_b)
            .bind(parsed_row.is_hub_curr_a)
            .bind(parsed_row.is_hub_curr_b)
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn insert_multiple_processed_rows(
        &self,
        processed_rows: &[ParsedDbRow],
        realm: &Realm,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let table = get_table_name(realm);

        // Trailing space here is intentional
        let query_str = format!(
            "INSERT INTO {} (
            change_id,
            league,
            market_id,
            currency_a_name_ggg,
            currency_b_name_ggg,
            currency_a_name_common,
            currency_b_name_common,
            volume_traded_currency_a,
            volume_traded_currency_b,
            lowest_stock_currency_a,
            lowest_stock_currency_b,
            highest_stock_currency_a,
            highest_stock_currency_b,
            lowest_ratio_currency_a,
            lowest_ratio_currency_b,
            highest_ratio_currency_a,
            highest_ratio_currency_b,
            is_hub_curr_a,
            is_hub_curr_b) ",
            table
        );

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(query_str);
        // INFO: This must be changed if the struct is ever changed
        let fields_in_row = 19; // no good way to do this programatically?
        for chunk in processed_rows.chunks(SQLITE_BIND_LIMIT / fields_in_row) {
            query_builder.push_values(chunk, |mut b, row| {
                b.push_bind(row.change_id)
                    .push_bind(&row.league)
                    .push_bind(&row.market_id)
                    .push_bind(&row.currency_a_name_ggg)
                    .push_bind(&row.currency_b_name_ggg)
                    .push_bind(&row.currency_a_name_common)
                    .push_bind(&row.currency_b_name_common)
                    .push_bind(row.volume_traded_currency_a)
                    .push_bind(row.volume_traded_currency_b)
                    .push_bind(row.lowest_stock_currency_a)
                    .push_bind(row.lowest_stock_currency_b)
                    .push_bind(row.highest_stock_currency_a)
                    .push_bind(row.highest_stock_currency_b)
                    .push_bind(row.lowest_ratio_currency_a)
                    .push_bind(row.lowest_ratio_currency_b)
                    .push_bind(row.highest_ratio_currency_a)
                    .push_bind(row.lowest_ratio_currency_b)
                    .push_bind(row.is_hub_curr_a)
                    .push_bind(row.is_hub_curr_b);
            });

            let query = query_builder.build();
            query.execute(&mut *conn).await?;

            query_builder.reset();
        }

        Ok(())
    }

    /// Retrieve the single most recent raw entry for the given game version
    pub async fn get_latest_raw(&self, realm: &Realm) -> Result<Option<DbRow>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let results: Option<DbRow> = query_as(
            "SELECT id, change_id, game_version, payload, parsed_bool FROM data
        WHERE game_version = $1
        ORDER BY change_id DESC
        LIMIT 1;",
        )
        .bind(realm.to_string().to_lowercase())
        .fetch_optional(&mut *conn)
        .await?;

        Ok(results)
    }

    /// Retrieve the entire processed marketplace for a given change_id, realm, and league
    pub async fn get_parsed_marketplace(
        &self,
        change_id: i64,
        realm: &Realm,
        league_id: &str,
    ) -> Result<Vec<ParsedDbRow>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let table_name = get_table_name(realm);
        let query_str = format!(
            "SELECT * FROM {} WHERE league = $1 AND change_id = $2;",
            table_name
        );

        let results: Vec<ParsedDbRow> = query_as(&query_str)
            .bind(league_id)
            .bind(change_id)
            .fetch_all(&mut *conn)
            .await?;

        Ok(results)
    }

    /// Retrieve the latest parsed data for a given realm and league
    pub async fn get_latest_parsed_marketplace(
        &self,
        realm: &Realm,
        league_id: &str,
    ) -> Result<Vec<ParsedDbRow>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let table_name = get_table_name(realm);
        let query_str = format!(
            "SELECT max(change_id) FROM {} WHERE league = $1;",
            table_name
        );

        let max_change_id: Option<(i64,)> = query_as(&query_str)
            .bind(league_id)
            .fetch_optional(&mut *conn)
            .await?;

        if let Some(id) = max_change_id {
            self.get_parsed_marketplace(id.0, realm, league_id).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Delete parsed entries older than a cutoff to prevent the DB from growing too large
    pub async fn delete_old_entries(&self, cutoff_timestamp: i64) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let _ = query(
            "DELETE FROM data
        WHERE change_id < $1 and parsed_bool = 1;",
        )
        .bind(cutoff_timestamp)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}
fn get_table_name(realm: &Realm) -> String {
    match realm {
        Realm::Poe1 => "poe1_markets".to_string(),
        Realm::Poe2 => "poe2_markets".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    impl DbClient {
        async fn try_in_memory() -> Result<Self, sqlx::Error> {
            // These settings are needed to prevent the db from immediately closing
            // or something weird with migrations not applying
            // See here: https://github.com/launchbadge/sqlx/issues/2510
            let pool = SqlitePoolOptions::new()
                .min_connections(3)
                .max_connections(10)
                .connect("sqlite::memory:")
                .await?;
            MIGRATOR.run(&pool).await?;

            Ok(DbClient { pool })
        }
    }

    #[tokio::test]
    async fn test_insert_data() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");

        let res = client
            .insert_data(12345, &Realm::Poe2, "{item : chaos_orb}")
            .await;
        println!("res is {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_empty_latest() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");
        let res = client.get_latest_raw(&Realm::Poe1).await;
        assert!(res.is_ok_and(|x| x.is_none()));
    }

    #[tokio::test]
    async fn test_get_latest() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");

        client
            .insert_data(12345, &Realm::Poe1, "{}")
            .await
            .expect("Should have inserted row");
        let latest = client.get_latest_raw(&Realm::Poe1).await;
        let expected = DbRow {
            change_id: 12345,
            id: 1,
            game_version: "poe1".to_string(),
            payload: "{}".to_string(),
            parsed_bool: 0,
        };

        assert_eq!(latest.unwrap().unwrap(), expected);
    }

    #[tokio::test]
    async fn test_get_specific_change_id() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");
        client
            .insert_data(12345, &Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do first insert");
        client
            .insert_data(12346, &Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do second insert");

        let res = client.get_specific_change_id(12346, &Realm::Poe1).await;
        let expected = DbRow {
            id: 2,
            change_id: 12346,
            game_version: "poe1".to_string(),
            payload: "{}".to_string(),
            parsed_bool: 0,
        };
        println!("result {:?}", res);
        assert!(res.is_ok_and(|f| f.is_some_and(|v| v == expected)))
    }

    #[tokio::test]
    async fn test_mark_record_as_processed() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");

        client
            .insert_data(12345, &Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do first insert");

        client
            .mark_record_as_processed(12345, &Realm::Poe1)
            .await
            .expect("Should have been able to mark record");

        let row = client
            .get_specific_change_id(12345, &Realm::Poe1)
            .await
            .expect("Should have retrieved value");
        assert!(row.is_some_and(|r| r.parsed_bool == 1));
    }

    #[tokio::test]
    async fn test_delete_old_entries() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");
        client
            .insert_data(12345, &Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do first insert");
        client
            .insert_data(12346, &Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do second insert");

        client
            .mark_record_as_processed(12345, &Realm::Poe1)
            .await
            .expect("Should have been able to mark record.");

        let del = client.delete_old_entries(12346).await;
        assert!(del.is_ok());

        let missing = client.get_specific_change_id(12345, &Realm::Poe1).await;
        assert!(missing.is_ok_and(|m| m.is_none()));

        let res = client.get_specific_change_id(12346, &Realm::Poe1).await;
        let expected = DbRow {
            id: 2,
            change_id: 12346,
            game_version: "poe1".to_string(),
            payload: "{}".to_string(),
            parsed_bool: 0,
        };
        // println!("result {:?}", res);
        assert!(res.is_ok_and(|f| f.is_some_and(|v| v == expected)));
    }

    #[tokio::test]
    async fn test_insert_processed_row() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");

        let data = ParsedDbRow {
            id: None,
            change_id: 12345,
            league: "Test".to_string(),
            currency_a_name_ggg: "Metadata/Items/Currency/CurrencyAddModToRare".to_string(),
            currency_b_name_ggg: "Metadata/Items/Currency/CurrencyRerollRare".to_string(),
            currency_a_name_common: "Exalted Orb".to_string(),
            currency_b_name_common: "Chaos Orb".to_string(),
            ..Default::default()
        };

        let mut expected = data.clone();
        expected.id = Some(1);

        client
            .insert_processed_row(&data, &Realm::Poe2)
            .await
            .expect("Should have been able to insert");

        let result = client
            .get_parsed_marketplace(12345, &Realm::Poe2, "Test")
            .await
            .expect("Should have been able to query db");

        assert_eq!(result.first().expect("Expected an item"), &expected);
    }

    #[tokio::test]
    async fn test_insert_multiple_processed_rows() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");

        let data: Vec<ParsedDbRow> = (0..3)
            .map(|n| ParsedDbRow {
                id: None,
                change_id: n,
                league: "Test".to_string(),
                currency_a_name_ggg: "Metadata/Items/Currency/CurrencyAddModToRare".to_string(),
                currency_b_name_ggg: "Metadata/Items/Currency/CurrencyRerollRare".to_string(),
                currency_a_name_common: "Exalted Orb".to_string(),
                currency_b_name_common: "Chaos Orb".to_string(),
                ..Default::default()
            })
            .collect();

        client
            .insert_multiple_processed_rows(&data, &Realm::Poe2)
            .await
            .expect("Should have entered multiple rows");

        let mut expected_first = data.first().unwrap().clone();
        expected_first.id = Some(1);
        let mut expected_last = data.last().unwrap().clone();
        expected_last.id = Some(3);

        let result_first = client
            .get_parsed_marketplace(0, &Realm::Poe2, "Test")
            .await
            .expect("Should have been able to query");

        let result_last = client
            .get_parsed_marketplace(2, &Realm::Poe2, "Test")
            .await
            .expect("Should have been able to query");

        assert_eq!(result_first.first().unwrap(), &expected_first);
        assert_eq!(result_last.first().unwrap(), &expected_last);
    }
}
