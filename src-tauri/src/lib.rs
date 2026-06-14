pub mod commands;
pub mod db;
pub mod models;
pub mod services;

use commands::{
    api_proxy, daily_insight, global_closeout, import_export, mind, personal_memory, plan, record,
    score, settings, stat,
};
use db::connection::{AppDataDirState, DbState};
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
            app.manage(AppDataDirState(app_data_dir));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            record::get_records_by_date,
            record::create_record,
            record::create_sub_record,
            record::create_manual_record_entry,
            record::delete_record,
            record::start_record_timer,
            record::pause_record_timer,
            record::reset_record_timer,
            record::toggle_record_completed,
            record::update_record_timer_mode,
            stat::get_dimension_totals,
            stat::get_ledger_by_date,
            stat::get_all_ledger,
            stat::get_streak_info,
            stat::get_calendar_overview,
            stat::get_daily_closeout_status,
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
            api_proxy::call_plan_api,
            plan::get_week_plan,
            plan::get_month_plan,
            plan::get_calendar_plan_history,
            plan::save_plan_cycle,
            plan::save_plan_item,
            plan::delete_plan_item,
            plan::refresh_plan_progress,
            plan::submit_plan_ai_answers,
            plan::apply_plan_ai_update,
            mind::get_bond_people,
            mind::save_bond_person,
            mind::delete_bond_person,
            mind::get_bond_entries,
            mind::save_bond_entry,
            mind::delete_bond_entry,
            mind::get_daily_journal_by_date,
            mind::get_recent_daily_journals,
            mind::save_daily_journal,
            mind::delete_daily_journal,
            personal_memory::get_personal_profile,
            personal_memory::save_personal_profile,
            personal_memory::get_personal_memory_overview,
            personal_memory::search_personal_memory,
            personal_memory::build_personal_context_pack,
            personal_memory::apply_personal_memory_patch,
            personal_memory::export_rag_memory,
            personal_memory::rebuild_rag_memory_files,
            daily_insight::generate_tarot_insight,
            daily_insight::generate_period_report,
            daily_insight::list_insight_reports,
            daily_insight::get_insight_context_snapshot,
            daily_insight::delete_insight_report,
            daily_insight::get_calendar_insight_history,
            global_closeout::run_global_closeout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
