use crate::db::connection::DbState;
use crate::db::repositories::record_repo;
use crate::models::record::Record;
use chrono::Local;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct RecordItem {
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

#[tauri::command]
pub fn get_records_by_date(state: State<DbState>, date: String) -> Result<Vec<RecordItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let records = record_repo::get_records_by_date(&conn, &date).map_err(|e| e.to_string())?;
    Ok(records.into_iter().map(map_record_item).collect())
}

#[tauri::command]
pub fn create_record(
    state: State<DbState>,
    date: String,
    title: String,
    difficulty_star: i32,
    _timer_mode: Option<String>,
    _countdown_target_seconds: Option<i32>,
) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = record_repo::create_record_with_options(
        &conn,
        &date,
        &title,
        0,
        difficulty_star,
        None,
        "stopwatch",
        None,
    )
    .map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

#[tauri::command]
pub fn create_sub_record(
    state: State<DbState>,
    parent_id: i64,
    title: String,
    difficulty_star: i32,
    _timer_mode: Option<String>,
    _countdown_target_seconds: Option<i32>,
) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = record_repo::create_sub_record(
        &conn,
        parent_id,
        &title,
        difficulty_star,
        "stopwatch",
        None,
    )
    .map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

#[tauri::command]
pub fn create_manual_record_entry(
    state: State<DbState>,
    date: String,
    title: String,
    minutes: i32,
    difficulty_star: i32,
    parent_id: Option<i64>,
) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = match parent_id {
        Some(parent_id) => record_repo::create_sub_record_with_minutes(
            &conn,
            parent_id,
            &title,
            minutes,
            difficulty_star,
        ),
        None => record_repo::create_record_with_options(
            &conn,
            &date,
            &title,
            minutes,
            difficulty_star,
            None,
            "stopwatch",
            None,
        ),
    }
    .map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

#[tauri::command]
pub fn delete_record(state: State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    record_repo::delete_record(&conn, id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn start_record_timer(state: State<DbState>, id: i64) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = now_ts();
    record_repo::start_record_timer(&conn, id, &now).map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

#[tauri::command]
pub fn pause_record_timer(state: State<DbState>, id: i64) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = now_ts();
    record_repo::pause_record_timer(&conn, id, &now).map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

#[tauri::command]
pub fn reset_record_timer(state: State<DbState>, id: i64) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    record_repo::reset_record_timer(&conn, id).map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

#[tauri::command]
pub fn toggle_record_completed(
    state: State<DbState>,
    id: i64,
    is_completed: bool,
) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let completed_at = if is_completed { Some(now_ts()) } else { None };
    record_repo::toggle_record_completed(&conn, id, is_completed, completed_at.as_deref())
        .map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

#[tauri::command]
pub fn update_record_timer_mode(
    state: State<DbState>,
    id: i64,
    _timer_mode: String,
    _countdown_target_seconds: Option<i32>,
) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    record_repo::update_record_timer_mode(&conn, id, "stopwatch", None)
        .map_err(|e| e.to_string())?;
    let record = record_repo::get_record_by_id(&conn, id).map_err(|e| e.to_string())?;
    Ok(map_record_item(record))
}

fn map_record_item(record: Record) -> RecordItem {
    RecordItem {
        id: record.id,
        date: record.date,
        title: record.title,
        minutes: record.minutes,
        difficulty_star: record.difficulty_star,
        parent_id: record.parent_id,
        is_completed: record.is_completed,
        completed_at: record.completed_at,
        elapsed_seconds: record.elapsed_seconds,
        timer_mode: record.timer_mode,
        countdown_target_seconds: record.countdown_target_seconds,
        timer_started_at: record.timer_started_at,
    }
}

fn now_ts() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
