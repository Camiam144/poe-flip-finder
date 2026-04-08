//! This module holds some convenience functions for interacting with the on-disk
//! Sqlite database. This will hold/cache some recent data so we can do some things
//! with a little bit of history. Might be able to catch some explosive price
//! changes in near-real time (with an hour or two lag) or overnight streamer-driven
//! changes.
use sqlx::migrate;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{query, query_as};
use std::path::PathBuf;

const DB_URL: &str = "sqlite://poe-flip-finder-database.db";
static MIGRATOR: migrate::Migrator = migrate!("./migrations");

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
struct DbRow {
    id: i64,
    change_id: i64,
    game_version: String,
    payload: String,
}

/// Initialize the database connection
/// if the database does not exist, create it and run the migration
async fn initialize_db(db_path: PathBuf) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

// TODO:Honestly everything below here should probably be attached to a struct

// Insert data into the database
async fn insert_data(
    pool: &SqlitePool,
    change_id: i64,
    game_version: &str,
    payload: &str,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let _ = query(
        "INSERT INTO data (change_id, game_version, payload)
        VALUES ($1, $2, $3)
    ",
    )
    .bind(change_id)
    .bind(game_version)
    .bind(payload)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn get_specific_change_id(
    pool: &SqlitePool,
    change_id: i64,
    game_version: &str,
) -> Result<Option<DbRow>, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let results: Option<DbRow> = query_as(
        "SELECT id, change_id, game_version, payload
        FROM data
        WHERE change_id = $1
        AND game_version = $2;",
    )
    .bind(change_id)
    .bind(game_version)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(results)
}

// Retrieve the single most recent entry for the given game version
async fn get_latest(pool: &SqlitePool, game_version: &str) -> Result<Option<DbRow>, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let results: Option<DbRow> = query_as(
        "SELECT id, change_id, game_version, payload FROM data
        WHERE game_version = $1
        ORDER BY change_id DESC
        LIMIT 1;",
    )
    .bind(game_version)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(results)
}

// Delete entries older than a cutoff to prevent the DB from growing too large
async fn delete_old_entries(pool: &SqlitePool, cutoff_timestamp: i64) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let _ = query(
        "DELETE FROM data
        WHERE change_id < $1;",
    )
    .bind(cutoff_timestamp)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_insert_data(pool: SqlitePool) {
        // let pool = setup_testing_db().await;

        let res = insert_data(&pool, 12345, "POE2", "{item : chaos_orb}").await;
        println!("res is {:?}", res);
        assert!(res.is_ok());
    }

    #[sqlx::test]
    async fn test_get_empty_latest(pool: SqlitePool) {
        let res = get_latest(&pool, "POE1").await;
        assert!(res.is_ok_and(|x| x.is_none()));
    }

    #[sqlx::test]
    async fn test_get_latest(pool: SqlitePool) {
        insert_data(&pool, 12345, "POE1", "{}")
            .await
            .expect("Should have inserted row");
        let latest = get_latest(&pool, "POE1").await;
        let expected = DbRow {
            change_id: 12345,
            id: 1,
            game_version: "POE1".to_string(),
            payload: "{}".to_string(),
        };

        assert!(latest.is_ok_and(|l| l.is_some_and(|x| x == expected)))
    }

    #[sqlx::test]
    async fn test_get_specific_change_id(pool: SqlitePool) {
        insert_data(&pool, 12345, "POE1", "{}")
            .await
            .expect("Should have been able to do first insert");
        insert_data(&pool, 12346, "POE1", "{}")
            .await
            .expect("Should have been able to do second insert");

        let res = get_specific_change_id(&pool, 12346, "POE1").await;
        let expected = DbRow {
            id: 2,

            change_id: 12346,
            game_version: "POE1".to_string(),
            payload: "{}".to_string(),
        };
        println!("result {:?}", res);
        assert!(res.is_ok_and(|f| f.is_some_and(|v| v == expected)))
    }

    #[sqlx::test]
    async fn test_delete_old_entries(pool: SqlitePool) {
        insert_data(&pool, 12345, "POE1", "{}")
            .await
            .expect("Should have been able to do first insert");
        insert_data(&pool, 12346, "POE1", "{}")
            .await
            .expect("Should have been able to do second insert");

        let del = delete_old_entries(&pool, 12346).await;
        assert!(del.is_ok());

        let missing = get_specific_change_id(&pool, 12345, "POE1").await;
        assert!(missing.is_ok_and(|m| m.is_none()));

        let res = get_specific_change_id(&pool, 12346, "POE1").await;
        let expected = DbRow {
            id: 2,

            change_id: 12346,
            game_version: "POE1".to_string(),
            payload: "{}".to_string(),
        };
        println!("result {:?}", res);
        assert!(res.is_ok_and(|f| f.is_some_and(|v| v == expected)));
    }
}
