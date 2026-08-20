//! This module holds some convenience functions for interacting with the on-disk
//! Sqlite database. This will hold/cache some recent data so we can do some things
//! with a little bit of history. Might be able to catch some explosive price
//! changes in near-real time (with an hour or two lag) or overnight streamer-driven
//! changes.
pub mod models;
pub mod transform;
use models::DbRow;
use sqlx::migrate;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{query, query_as};
use std::path::PathBuf;

use crate::ggg_api::Realm;

static MIGRATOR: migrate::Migrator = migrate!("./migrations");

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
        realm: Realm,
        payload: &str,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let _ = query(
            "INSERT INTO data (change_id, game_version, payload)
        VALUES ($1, $2, $3)
    ",
        )
        .bind(change_id)
        .bind(realm.to_string())
        .bind(payload)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Get a specific change_id and game version
    pub async fn get_specific_change_id(
        &self,
        change_id: i64,
        realm: Realm,
    ) -> Result<Option<DbRow>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let results: Option<DbRow> = query_as(
            "SELECT id, change_id, game_version, payload
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

    /// Retrieve the single most recent entry for the given game version
    pub async fn get_latest(&self, realm: Realm) -> Result<Option<DbRow>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let results: Option<DbRow> = query_as(
            "SELECT id, change_id, game_version, payload FROM data
        WHERE game_version = $1
        ORDER BY change_id DESC
        LIMIT 1;",
        )
        .bind(realm.to_string().to_lowercase())
        .fetch_optional(&mut *conn)
        .await?;

        Ok(results)
    }

    // Delete entries older than a cutoff to prevent the DB from growing too large
    pub async fn delete_old_entries(&self, cutoff_timestamp: i64) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let _ = query(
            "DELETE FROM data
        WHERE change_id < $1;",
        )
        .bind(cutoff_timestamp)
        .execute(&mut *conn)
        .await?;

        Ok(())
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
            .insert_data(12345, Realm::Poe2, "{item : chaos_orb}")
            .await;
        println!("res is {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_empty_latest() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");
        let res = client.get_latest(Realm::Poe1).await;
        assert!(res.is_ok_and(|x| x.is_none()));
    }

    #[tokio::test]
    async fn test_get_latest() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");

        client
            .insert_data(12345, Realm::Poe1, "{}")
            .await
            .expect("Should have inserted row");
        let latest = client.get_latest(Realm::Poe1).await;
        let expected = DbRow {
            change_id: 12345,
            id: 1,
            game_version: "poe1".to_string(),
            payload: "{}".to_string(),
        };

        assert_eq!(latest.unwrap().unwrap(), expected);
    }

    #[tokio::test]
    async fn test_get_specific_change_id() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");
        client
            .insert_data(12345, Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do first insert");
        client
            .insert_data(12346, Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do second insert");

        let res = client.get_specific_change_id(12346, Realm::Poe1).await;
        let expected = DbRow {
            id: 2,
            change_id: 12346,
            game_version: "poe1".to_string(),
            payload: "{}".to_string(),
        };
        println!("result {:?}", res);
        assert!(res.is_ok_and(|f| f.is_some_and(|v| v == expected)))
    }

    #[tokio::test]
    async fn test_delete_old_entries() {
        let client = DbClient::try_in_memory()
            .await
            .expect("Should have created in-memory db");
        client
            .insert_data(12345, Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do first insert");
        client
            .insert_data(12346, Realm::Poe1, "{}")
            .await
            .expect("Should have been able to do second insert");

        let del = client.delete_old_entries(12346).await;
        assert!(del.is_ok());

        let missing = client.get_specific_change_id(12345, Realm::Poe1).await;
        assert!(missing.is_ok_and(|m| m.is_none()));

        let res = client.get_specific_change_id(12346, Realm::Poe1).await;
        let expected = DbRow {
            id: 2,
            change_id: 12346,
            game_version: "poe1".to_string(),
            payload: "{}".to_string(),
        };
        println!("result {:?}", res);
        assert!(res.is_ok_and(|f| f.is_some_and(|v| v == expected)));
    }
}
