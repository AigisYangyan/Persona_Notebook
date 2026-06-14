use crate::db::connection::{AppDataDirState, DbState};
use crate::db::repositories::personal_memory_repo;
use crate::models::personal_memory::{
    PersonalContextPack, PersonalMemoryOverview, PersonalMemoryPatchApplyResult,
    PersonalMemoryViewItem, PersonalProfile,
};
use crate::services::rag_memory_service;
use tauri::State;

#[tauri::command]
pub fn get_personal_profile(state: State<DbState>) -> Result<PersonalProfile, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    personal_memory_repo::get_personal_profile(&conn)
}

#[tauri::command]
pub fn save_personal_profile(
    state: State<DbState>,
    app_data_dir: State<AppDataDirState>,
    birthday: String,
    personality: String,
    experiences: String,
    personal_notes: String,
) -> Result<PersonalProfile, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let profile = personal_memory_repo::save_personal_profile(
        &conn,
        &PersonalProfile {
            birthday,
            personality,
            experiences,
            personal_notes,
            updated_at: None,
        },
    )?;
    rag_memory_service::rebuild_rag_memory_files(&conn, &app_data_dir.0.join("rag_memory"))?;
    Ok(profile)
}

#[tauri::command]
pub fn get_personal_memory_overview(state: State<DbState>) -> Result<PersonalMemoryOverview, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    personal_memory_repo::get_personal_memory_overview(&conn)
}

#[tauri::command]
pub fn search_personal_memory(
    state: State<DbState>,
    query: Option<String>,
    tags: Option<Vec<String>>,
    limit: Option<usize>,
) -> Result<Vec<PersonalMemoryViewItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    personal_memory_repo::search_personal_memory(
        &conn,
        query.as_deref(),
        tags.as_deref().unwrap_or(&[]),
        limit.unwrap_or(50),
    )
}

#[tauri::command]
pub fn build_personal_context_pack(
    state: State<DbState>,
    date: Option<String>,
    mode: Option<String>,
) -> Result<PersonalContextPack, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let target_date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let target_mode = mode.unwrap_or_else(|| "general".to_string());
    personal_memory_repo::build_personal_context_pack(&conn, &target_date, &target_mode)
}

#[tauri::command]
pub fn apply_personal_memory_patch(
    state: State<DbState>,
    app_data_dir: State<AppDataDirState>,
    patch_json: String,
    source_context_id: String,
) -> Result<PersonalMemoryPatchApplyResult, String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let result = personal_memory_repo::apply_memory_patch(&mut conn, &patch_json, &source_context_id)?;
    let rag_memory_dir = app_data_dir.0.join("rag_memory");
    rag_memory_service::write_patch_run_file(
        &rag_memory_dir,
        &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        &patch_json,
    )?;
    rag_memory_service::rebuild_rag_memory_files(&conn, &rag_memory_dir)?;
    Ok(result)
}

#[tauri::command]
pub fn export_rag_memory(
    state: State<DbState>,
    app_data_dir: State<AppDataDirState>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    rag_memory_service::export_rag_memory_snapshot(&conn, &app_data_dir.0.join("rag_memory"))
}

#[tauri::command]
pub fn rebuild_rag_memory_files(
    state: State<DbState>,
    app_data_dir: State<AppDataDirState>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let manifest = rag_memory_service::rebuild_rag_memory_files(&conn, &app_data_dir.0.join("rag_memory"))?;
    serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())
}
