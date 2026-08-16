#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct DbRow {
    pub id: i64,
    pub change_id: i64,
    pub game_version: String,
    pub payload: String,
}
