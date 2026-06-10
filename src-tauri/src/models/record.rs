use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub minutes: i32,
    pub difficulty_star: i32,
}
