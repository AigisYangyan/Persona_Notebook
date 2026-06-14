use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub minutes: i32,
    pub difficulty_star: i32,
    pub parent_id: Option<i64>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub elapsed_seconds: i64,
    pub timer_mode: String,
    pub countdown_target_seconds: Option<i32>,
    pub timer_started_at: Option<String>,
}
