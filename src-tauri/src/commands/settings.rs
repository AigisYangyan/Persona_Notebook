use crate::db::connection::DbState;
use crate::db::repositories::setting_repo;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

fn normalize_scoring_engine(engine: &str) -> String {
    match engine {
        "rules_api" | "local" | "api" => "rules_api".into(),
        _ => "rules_api".into(),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub scoring_engine: String,
    pub deepseek_base_url: String,
    pub deepseek_flash_model: String,
    pub deepseek_pro_model: String,
    pub api_key_configured: bool,
}

#[tauri::command]
pub fn get_settings(state: State<DbState>) -> Result<AppSettings, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let raw_engine = setting_repo::get_setting(&conn, "scoring_engine")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "rules_api".into());
    let engine = normalize_scoring_engine(&raw_engine);
    let deepseek_base_url = resolve_deepseek_base_url(&conn)?;
    let (deepseek_flash_model, deepseek_pro_model) = resolve_deepseek_models(&conn)?;
    let api_key = setting_repo::get_setting(&conn, "api_key")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    Ok(AppSettings {
        scoring_engine: engine,
        deepseek_base_url,
        deepseek_flash_model,
        deepseek_pro_model,
        api_key_configured: !api_key.trim().is_empty(),
    })
}

#[tauri::command]
pub fn save_general_settings(
    state: State<DbState>,
    scoring_engine: String,
    deepseek_base_url: String,
    deepseek_flash_model: String,
    deepseek_pro_model: String,
) -> Result<(), String> {
    if scoring_engine != "rules_api" && scoring_engine != "local" && scoring_engine != "api" {
        return Err("鏃犳晥鐨勮瘎鍒嗗紩鎿?".into());
    }

    let normalized_engine = normalize_scoring_engine(&scoring_engine);
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "scoring_engine", &normalized_engine)
        .map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "deepseek_base_url", &deepseek_base_url)
        .map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "deepseek_flash_model", &deepseek_flash_model)
        .map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "deepseek_pro_model", &deepseek_pro_model)
        .map_err(|e| e.to_string())?;

    // Keep legacy keys in sync for compatibility with older data exports or old binaries.
    setting_repo::set_setting(&conn, "api_base_url", &deepseek_base_url)
        .map_err(|e| e.to_string())?;
    setting_repo::set_setting(&conn, "api_model", &deepseek_pro_model)
        .map_err(|e| e.to_string())?;
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

fn resolve_deepseek_base_url(conn: &Connection) -> Result<String, String> {
    let configured = setting_repo::get_setting(conn, "deepseek_base_url")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    if !configured.trim().is_empty() {
        return Ok(configured);
    }

    let legacy_base_url = setting_repo::get_setting(conn, "api_base_url")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let resolved = if is_deepseek_like(&legacy_base_url) {
        legacy_base_url
    } else {
        "https://api.deepseek.com/v1".to_string()
    };

    setting_repo::set_setting(conn, "deepseek_base_url", &resolved).map_err(|e| e.to_string())?;
    Ok(resolved)
}

fn resolve_deepseek_models(conn: &Connection) -> Result<(String, String), String> {
    let mut flash_model = setting_repo::get_setting(conn, "deepseek_flash_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let mut pro_model = setting_repo::get_setting(conn, "deepseek_pro_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let legacy_model = setting_repo::get_setting(conn, "api_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    if flash_model.trim().is_empty() && is_deepseek_like(&legacy_model) {
        flash_model = legacy_model.clone();
        setting_repo::set_setting(conn, "deepseek_flash_model", &flash_model)
            .map_err(|e| e.to_string())?;
    }
    if pro_model.trim().is_empty() && is_deepseek_like(&legacy_model) {
        pro_model = legacy_model;
        setting_repo::set_setting(conn, "deepseek_pro_model", &pro_model)
            .map_err(|e| e.to_string())?;
    }

    Ok((flash_model, pro_model))
}

fn is_deepseek_like(value: &str) -> bool {
    value.to_ascii_lowercase().contains("deepseek")
}
