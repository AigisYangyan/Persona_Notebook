use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::connection::DbState;
use crate::db::repositories::record_repo;

#[derive(Deserialize)]
pub struct ImportRecord {
    pub title: String,
    pub minutes: i32,
    pub date: String,
}

#[derive(Serialize)]
pub struct ImportResult {
    pub imported: i32,
    pub errors: Vec<String>,
}

#[tauri::command]
pub fn import_csv(state: State<DbState>, content: String) -> Result<ImportResult, String> {
    let mut imported = 0;
    let mut errors = Vec::new();
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    for (idx, result) in rdr.records().enumerate() {
        match result {
            Ok(record) => {
                let title = record.get(0).unwrap_or("").trim();
                let minutes: i32 = record.get(1).unwrap_or("0").parse().unwrap_or(0);
                let date = record.get(2).unwrap_or("").trim();
                if title.is_empty() || date.is_empty() {
                    errors.push(format!("第 {} 行: 标题或日期为空", idx + 2));
                    continue;
                }
                if let Err(e) = record_repo::create_record(&conn, date, title, minutes, 0) {
                    errors.push(format!("第 {} 行: {}", idx + 2, e));
                } else {
                    imported += 1;
                }
            }
            Err(e) => {
                errors.push(format!("第 {} 行解析错误: {}", idx + 2, e));
            }
        }
    }

    Ok(ImportResult { imported, errors })
}

#[tauri::command]
pub fn import_json(state: State<DbState>, content: String) -> Result<ImportResult, String> {
    let mut imported = 0;
    let mut errors = Vec::new();
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let records: Vec<ImportRecord> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    for (idx, record) in records.iter().enumerate() {
        if record.title.is_empty() || record.date.is_empty() {
            errors.push(format!("第 {} 项: 标题或日期为空", idx + 1));
            continue;
        }
        if let Err(e) = record_repo::create_record(&conn, &record.date, &record.title, record.minutes, 0) {
            errors.push(format!("第 {} 项: {}", idx + 1, e));
        } else {
            imported += 1;
        }
    }

    Ok(ImportResult { imported, errors })
}

#[tauri::command]
pub fn export_data(state: State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, date, title, minutes, difficulty_star FROM records ORDER BY date, created_at")
        .map_err(|e| e.to_string())?;
    
    #[derive(Serialize)]
    struct ExportRecord {
        id: i64,
        date: String,
        title: String,
        minutes: i32,
        difficulty_star: i32,
    }

    let records = stmt.query_map([], |row| {
        Ok(ExportRecord {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            minutes: row.get(3)?,
            difficulty_star: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&records).map_err(|e| e.to_string())
}
