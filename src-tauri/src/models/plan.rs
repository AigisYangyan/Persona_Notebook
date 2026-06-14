use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCycle {
    pub id: i64,
    pub period_type: String,
    pub start_date: String,
    pub end_date: String,
    pub title: String,
    pub summary: String,
    pub ai_summary: String,
    pub last_ai_run_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub id: i64,
    pub cycle_id: i64,
    pub title: String,
    pub description: String,
    pub dimension_key: Option<String>,
    pub progress_percent: i32,
    pub ai_comment: String,
    pub sort_order: i32,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAiSession {
    pub id: i64,
    pub cycle_id: i64,
    pub status: String,
    pub request_payload: String,
    pub response_payload: Option<String>,
    pub questions_json: String,
    pub answers_json: String,
    pub proposal_json: Option<String>,
}
