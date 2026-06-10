pub mod commands;
pub mod db;
pub mod models;
pub mod services;

use commands::{api_proxy, import_export, record, score, settings, stat};
use db::connection::DbState;
use db::migrations::run_migrations;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("data.db");
            let conn = rusqlite::Connection::open(&db_path)?;
            run_migrations(&conn)?;
            app.manage(DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            record::get_records_by_date,
            record::create_record,
            record::delete_record,
            stat::get_dimension_totals,
            stat::get_ledger_by_date,
            stat::get_all_ledger,
            stat::get_streak_info,
            stat::get_calendar_overview,
            stat::rollback_ledger,
            settings::get_settings,
            settings::save_general_settings,
            settings::save_api_key,
            settings::clear_api_key,
            score::preview_score_with_local_rules,
            score::confirm_score_preview,
            import_export::export_data,
            import_export::import_csv,
            import_export::import_json,
            api_proxy::call_scoring_api,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
