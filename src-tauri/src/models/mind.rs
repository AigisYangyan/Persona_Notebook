use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondPerson {
    pub id: i64,
    pub name: String,
    pub relation_label: String,
    pub score: i32,
    pub note: String,
    pub latest_entry_date: Option<String>,
    pub entry_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondEntry {
    pub id: i64,
    pub person_id: i64,
    pub entry_date: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyJournal {
    pub id: i64,
    pub entry_date: String,
    pub title: String,
    pub content: String,
    pub mood: String,
}
