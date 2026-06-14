use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalProfile {
    pub birthday: String,
    pub personality: String,
    pub experiences: String,
    pub personal_notes: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemoryItem {
    pub id: i64,
    pub memory_type: String,
    pub title: String,
    pub summary: String,
    pub detail: String,
    pub tags_json: String,
    pub importance: i32,
    pub confidence: f64,
    pub first_seen_date: Option<String>,
    pub last_seen_date: Option<String>,
    pub status: String,
    pub supersedes_id: Option<i64>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemorySource {
    pub id: i64,
    pub memory_id: i64,
    pub source_type: String,
    pub source_id: String,
    pub source_date: Option<String>,
    pub evidence_id: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemoryViewItem {
    pub id: i64,
    pub memory_type: String,
    pub title: String,
    pub summary: String,
    pub detail: String,
    pub tags: Vec<String>,
    pub importance: i32,
    pub confidence: f64,
    pub first_seen_date: Option<String>,
    pub last_seen_date: Option<String>,
    pub status: String,
    pub supersedes_id: Option<i64>,
    pub created_by: String,
    pub source_count: i32,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemoryOverview {
    pub total_items: i32,
    pub active_items: i32,
    pub pending_items: i32,
    pub rejected_items: i32,
    pub top_items: Vec<PersonalMemoryViewItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalContextPack {
    pub schema_version: String,
    pub date: String,
    pub mode: String,
    pub generated_at: String,
    pub profile: PersonalProfile,
    pub overview: PersonalMemoryOverview,
    pub high_priority_memories: Vec<PersonalMemoryViewItem>,
    pub recent_memories: Vec<PersonalMemoryViewItem>,
    pub relevant_memories: Vec<PersonalMemoryViewItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalProfilePatch {
    pub birthday: Option<String>,
    pub personality: Option<String>,
    pub experiences: Option<String>,
    pub personal_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemoryPatchSource {
    pub source_type: String,
    pub source_id: String,
    pub source_date: Option<String>,
    pub evidence_id: String,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemoryPatchOperation {
    pub op: String,
    pub target_id: Option<i64>,
    pub memory_type: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub detail: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub importance: Option<i32>,
    pub confidence: Option<f64>,
    pub reason: String,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub sources: Vec<PersonalMemoryPatchSource>,
    pub first_seen_date: Option<String>,
    pub last_seen_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemoryPatch {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub profile_updates: Option<PersonalProfilePatch>,
    #[serde(default)]
    pub memory_operations: Vec<PersonalMemoryPatchOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMemoryPatchApplyResult {
    pub patch_run_id: i64,
    pub validation_status: String,
    pub apply_status: String,
    pub applied_operations: i32,
    pub rejected_operations: i32,
    pub message: String,
}

pub fn default_schema_version() -> String {
    "1.0".to_string()
}
