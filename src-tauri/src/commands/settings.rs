use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::connection::DbState;
use crate::db::repositories::setting_repo;

fn normalize_scoring_engine(engine: &str) -> String {
    match engine {
        "rules_api" | "local" | "api" => "rules_api".into(),
        _ => "rules_api".into(),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub scoring_engine: String,
    pub api_base_url: String,
    pub api_model: String,
    pub api_key_configured: bool,
}

#[tauri::command]
pub fn get_settings(state: State<DbState>) -> Result<AppSettings, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let raw_engine = setting_repo::get_setting(&conn, "scoring_engine")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "rules_api".into());
    let engine = normalize_scoring_engine(&raw_engine);
    let api_base_url = setting_repo::get_setting(&conn, "api_base_url")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let api_model = setting_repo::get_setting(&conn, "api_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "gpt-4o-mini".into());
    let api_key = setting_repo::get_setting(&conn, "api_key")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    Ok(AppSettings {
        scoring_engine: engine,
        api_base_url,
        api_model,
        api_key_configured: !api_key.trim().is_empty(),
    })
}

#[tauri::command]
pub fn save_general_settings(
    state: State<DbState>,
    scoring_engine: String,
    api_base_url: String,
    api_model: String,
) -> Result<(), String> {
    if scoring_engine != "rules_api" && scoring_engine != "local" && scoring_engine != "api" {
        return Err("无效的评分引擎".into());
    }

    let normalized_engine = normalize_scoring_engine(&scoring_engine);
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "scoring_engine", &normalized_engine).map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "api_base_url", &api_base_url).map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "api_model", &api_model).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn save_api_key(state: State<DbState>, api_key: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "api_key", &api_key).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_api_key(state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "api_key", "").map_err(|e| e.to_string())?;
    Ok(())
}
