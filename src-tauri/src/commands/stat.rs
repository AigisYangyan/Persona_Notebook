use serde::Serialize;
use tauri::State;

use crate::db::connection::DbState;
use crate::db::repositories::{
    daily_review_repo, dimension_repo, ledger_repo, plan_repo, record_repo,
};

#[derive(Serialize)]
pub struct DimensionTotal {
    pub key: String,
    pub name: String,
    pub total: i32,
}

#[tauri::command]
pub fn get_dimension_totals(state: State<DbState>) -> Result<Vec<DimensionTotal>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let dims = dimension_repo::get_all_dimensions(&conn).map_err(|e| e.to_string())?;
    let totals = ledger_repo::get_dimension_totals(&conn).map_err(|e| e.to_string())?;
    let total_map: std::collections::HashMap<String, i32> = totals.into_iter().collect();

    Ok(dims
        .into_iter()
        .map(|d| DimensionTotal {
            key: d.key.clone(),
            name: d.name,
            total: total_map.get(&d.key).copied().unwrap_or(0),
        })
        .collect())
}

#[derive(Serialize)]
pub struct LedgerItem {
    pub id: i64,
    pub date: String,
    pub dimension_key: String,
    pub dimension_name: String,
    pub change_value: i32,
    pub source_title: String,
    pub reason: String,
    pub engine: String,
}

#[tauri::command]
pub fn get_ledger_by_date(state: State<DbState>, date: String) -> Result<Vec<LedgerItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let dims = dimension_repo::get_all_dimensions(&conn).map_err(|e| e.to_string())?;
    let dim_map: std::collections::HashMap<String, String> =
        dims.into_iter().map(|d| (d.key, d.name)).collect();
    let entries = ledger_repo::get_ledger_by_date(&conn, &date).map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| {
            let dimension_key = e.dimension_key;
            LedgerItem {
                id: e.id,
                date: e.date,
                dimension_name: dim_map
                    .get(&dimension_key)
                    .cloned()
                    .unwrap_or_else(|| dimension_key.clone()),
                dimension_key,
                change_value: e.change_value,
                source_title: e.source_title,
                reason: e.reason,
                engine: e.engine,
            }
        })
        .collect())
}

#[derive(Serialize)]
pub struct LedgerEntry {
    pub id: i64,
    pub date: String,
    pub dimension_key: String,
    pub dimension_name: String,
    pub change_value: i32,
    pub source_title: String,
    pub reason: String,
    pub engine: String,
}

#[tauri::command]
pub fn get_all_ledger(state: State<DbState>, limit: i64) -> Result<Vec<LedgerEntry>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let dims = dimension_repo::get_all_dimensions(&conn).map_err(|e| e.to_string())?;
    let dim_map: std::collections::HashMap<String, String> =
        dims.into_iter().map(|d| (d.key, d.name)).collect();

    let mut stmt = conn
        .prepare(
            "SELECT id, date, dimension_key, change_value, source_title, reason, engine
             FROM stat_ledger WHERE is_rollback = 0 ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map([limit], |row| {
            let key: String = row.get(2)?;
            Ok(LedgerEntry {
                id: row.get(0)?,
                date: row.get(1)?,
                dimension_key: key.clone(),
                dimension_name: dim_map.get(&key).cloned().unwrap_or_else(|| key.clone()),
                change_value: row.get(3)?,
                source_title: row.get(4)?,
                reason: row.get(5)?,
                engine: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub fn rollback_ledger(state: State<DbState>, ledger_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let Some(date) = ledger_repo::get_ledger_date(&conn, ledger_id).map_err(|e| e.to_string())?
    else {
        return Err("未找到要回滚的账本记录".into());
    };

    ledger_repo::rollback_ledger(&conn, ledger_id).map_err(|e| e.to_string())?;
    daily_review_repo::recalculate_review(&conn, &date, None).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct StreakInfo {
    pub current_streak: i32,
    pub longest_streak: i32,
}

#[tauri::command]
pub fn get_streak_info(state: State<DbState>) -> Result<StreakInfo, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT date FROM records ORDER BY date ASC")
        .map_err(|e| e.to_string())?;

    let dates: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(calculate_streaks(
        &dates,
        chrono::Local::now().naive_local().date(),
    ))
}

#[derive(Serialize)]
pub struct CalendarOverviewItem {
    pub date: String,
    pub record_count: i32,
    pub is_analyzed: bool,
    pub has_week_plan_update: bool,
    pub has_month_plan_update: bool,
}

#[derive(Serialize)]
pub struct DailyCloseoutStatus {
    pub date: String,
    pub has_ledger: bool,
    pub ledger_count: i32,
}

#[tauri::command]
pub fn get_daily_closeout_status(
    state: State<DbState>,
    date: String,
) -> Result<DailyCloseoutStatus, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let entries = ledger_repo::get_ledger_by_date(&conn, &date).map_err(|e| e.to_string())?;
    Ok(DailyCloseoutStatus {
        date,
        has_ledger: !entries.is_empty(),
        ledger_count: entries.len() as i32,
    })
}

#[tauri::command]
pub fn get_calendar_overview(
    state: State<DbState>,
    year: i32,
    month: u32,
) -> Result<Vec<CalendarOverviewItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| "无效的年月参数".to_string())?;
    let end = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .and_then(|date| date.pred_opt())
            .ok_or_else(|| "无法计算日历范围".to_string())?
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
            .and_then(|date| date.pred_opt())
            .ok_or_else(|| "无法计算日历范围".to_string())?
    };

    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();
    let record_counts = record_repo::get_record_counts_in_range(&conn, &start_str, &end_str)
        .map_err(|e| e.to_string())?;
    let review_flags = daily_review_repo::get_flags_in_range(&conn, &start_str, &end_str)
        .map_err(|e| e.to_string())?;

    let flag_map: std::collections::HashMap<String, bool> = review_flags
        .into_iter()
        .map(|flag| (flag.date, flag.is_analyzed))
        .collect();
    let count_map: std::collections::HashMap<String, i32> = record_counts
        .into_iter()
        .map(|item| (item.date, item.count))
        .collect();

    let mut items = Vec::new();
    let mut cursor = start;
    while cursor <= end {
        let date = cursor.format("%Y-%m-%d").to_string();
        let has_week_plan_update = plan_repo::get_cycle_covering_date(&conn, "week", &date)
            .map_err(|e| e.to_string())?
            .map(|cycle| !cycle.ai_summary.trim().is_empty() || cycle.last_ai_run_at.is_some())
            .unwrap_or(false);
        let has_month_plan_update = plan_repo::get_cycle_covering_date(&conn, "month", &date)
            .map_err(|e| e.to_string())?
            .map(|cycle| !cycle.ai_summary.trim().is_empty() || cycle.last_ai_run_at.is_some())
            .unwrap_or(false);

        items.push(CalendarOverviewItem {
            date: date.clone(),
            record_count: count_map.get(&date).copied().unwrap_or(0),
            is_analyzed: flag_map.get(&date).copied().unwrap_or(false),
            has_week_plan_update,
            has_month_plan_update,
        });
        cursor += chrono::Duration::days(1);
    }

    Ok(items)
}

fn calculate_streaks(dates: &[String], today: chrono::NaiveDate) -> StreakInfo {
    let parsed_dates: Vec<chrono::NaiveDate> = dates
        .iter()
        .filter_map(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        .collect();

    if parsed_dates.is_empty() {
        return StreakInfo {
            current_streak: 0,
            longest_streak: 0,
        };
    }

    let mut longest_streak = 1;
    let mut current_chain = 1;
    for window in parsed_dates.windows(2) {
        if window[1].signed_duration_since(window[0]).num_days() == 1 {
            current_chain += 1;
            longest_streak = longest_streak.max(current_chain);
        } else if window[1] != window[0] {
            current_chain = 1;
        }
    }

    let date_set: std::collections::HashSet<chrono::NaiveDate> =
        parsed_dates.iter().copied().collect();
    let mut current_streak = 0;
    let mut cursor = today;

    if !date_set.contains(&today) {
        let yesterday = today - chrono::Duration::days(1);
        if !date_set.contains(&yesterday) {
            return StreakInfo {
                current_streak: 0,
                longest_streak,
            };
        }
        cursor = yesterday;
    }

    while date_set.contains(&cursor) {
        current_streak += 1;
        cursor -= chrono::Duration::days(1);
    }

    StreakInfo {
        current_streak,
        longest_streak,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_streaks_handles_current_and_longest_separately() {
        let streaks = calculate_streaks(
            &[
                "2026-06-01".into(),
                "2026-06-02".into(),
                "2026-06-05".into(),
                "2026-06-06".into(),
                "2026-06-07".into(),
            ],
            chrono::NaiveDate::from_ymd_opt(2026, 6, 7).expect("valid date"),
        );

        assert_eq!(streaks.current_streak, 3);
        assert_eq!(streaks.longest_streak, 3);
    }

    #[test]
    fn calculate_streaks_returns_zero_when_today_and_yesterday_missing() {
        let streaks = calculate_streaks(
            &["2026-06-01".into(), "2026-06-02".into()],
            chrono::NaiveDate::from_ymd_opt(2026, 6, 10).expect("valid date"),
        );

        assert_eq!(streaks.current_streak, 0);
        assert_eq!(streaks.longest_streak, 2);
    }
}
