use serde::Serialize;
use tauri::State;
use crate::db::connection::DbState;
use crate::db::repositories::record_repo;

#[derive(Serialize, Clone)]
pub struct RecordItem {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub minutes: i32,
    pub difficulty_star: i32,
}

#[tauri::command]
pub fn get_records_by_date(
    state: State<DbState>,
    date: String,
) -> Result<Vec<RecordItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let records = record_repo::get_records_by_date(&conn, &date).map_err(|e| e.to_string())?;
    Ok(records
        .into_iter()
        .map(|r| RecordItem {
            id: r.id,
            date: r.date,
            title: r.title,
            minutes: r.minutes,
            difficulty_star: r.difficulty_star,
        })
        .collect())
}

#[tauri::command]
pub fn create_record(
    state: State<DbState>,
    date: String,
    title: String,
    minutes: i32,
    difficulty_star: i32,
) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = record_repo::create_record(&conn, &date, &title, minutes, difficulty_star)
        .map_err(|e| e.to_string())?;
    Ok(RecordItem {
        id,
        date,
        title,
        minutes,
        difficulty_star,
    })
}

#[tauri::command]
pub fn delete_record(state: State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    record_repo::delete_record(&conn, id).map_err(|e| e.to_string())?;
    Ok(())
}
