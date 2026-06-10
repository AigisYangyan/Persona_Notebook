use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Dimension {
    pub key: String,
    pub name: String,
    pub daily_cap: i32,
}

#[derive(Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: i64,
    pub date: String,
    pub dimension_key: String,
    pub change_value: i32,
    pub reason: String,
}
