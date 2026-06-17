use crate::commands::{api_proxy, daily_insight, plan, score};
use crate::db::connection::{AppDataDirState, DbState};
use crate::db::repositories::{
    daily_review_repo, dimension_repo, ledger_repo, record_repo, rule_repo,
};
use crate::services::ai_response_service;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct CloseoutStepDto {
    pub status: String,
    pub message: String,
    pub report_id: Option<i64>,
    pub session_id: Option<i64>,
    pub questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlobalCloseoutResultDto {
    pub date: String,
    pub scope: String,
    pub score: CloseoutStepDto,
    pub report: CloseoutStepDto,
    pub week_plan: CloseoutStepDto,
    pub month_plan: CloseoutStepDto,
    pub closeout_run_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct ScoreApiResponse {
    summary: String,
    record_results: Vec<ScoreApiRecordResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScoreApiRecordResult {
    title: String,
    category: String,
    changes: std::collections::HashMap<String, i32>,
    difficulty_star: i32,
    confidence: f64,
    reason: String,
}

#[tauri::command]
pub async fn run_global_closeout(
    state: State<'_, DbState>,
    app_data_dir: State<'_, AppDataDirState>,
    date: String,
    scope: String,
) -> Result<GlobalCloseoutResultDto, String> {
    ensure_closeout_table(&state)?;
    pause_running_timers(&state)?;

    let score_step = if matches!(scope.as_str(), "day" | "all") {
        run_score_step(&state, &date).await
    } else {
        skipped_step("Current scope does not recalculate today's points")
    };

    let report_step = if matches!(scope.as_str(), "day" | "all") {
        run_report_step(&state, &app_data_dir, &date).await
    } else {
        skipped_step("Current scope does not generate the daily report")
    };

    let week_step = if matches!(scope.as_str(), "week" | "all") {
        run_plan_step(&state, &date, "week").await
    } else {
        skipped_step("Current scope does not refresh the week plan")
    };

    let month_step = if matches!(scope.as_str(), "month" | "all") {
        run_plan_step(&state, &date, "month").await
    } else {
        skipped_step("Current scope does not refresh the month plan")
    };

    let closeout_run_id = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        insert_closeout_run(
            &conn,
            &date,
            &scope,
            &score_step,
            &report_step,
            &week_step,
            &month_step,
        )?
    };

    Ok(GlobalCloseoutResultDto {
        date,
        scope,
        score: score_step,
        report: report_step,
        week_plan: week_step,
        month_plan: month_step,
        closeout_run_id,
    })
}

#[tauri::command]
pub fn get_latest_closeout_run(
    state: State<'_, DbState>,
    date: String,
) -> Result<Option<GlobalCloseoutResultDto>, String> {
    ensure_closeout_table(&state)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT
            date,
            scope,
            score_status,
            score_message,
            report_status,
            report_message,
            report_id,
            week_status,
            week_message,
            week_session_id,
            month_status,
            month_message,
            month_session_id,
            id
         FROM closeout_runs
         WHERE date = ?1
         ORDER BY id DESC
         LIMIT 1",
        params![date],
        |row| {
            Ok(GlobalCloseoutResultDto {
                date: row.get(0)?,
                scope: row.get(1)?,
                score: CloseoutStepDto {
                    status: row.get(2)?,
                    message: row.get(3)?,
                    report_id: None,
                    session_id: None,
                    questions: Vec::new(),
                },
                report: CloseoutStepDto {
                    status: row.get(4)?,
                    message: row.get(5)?,
                    report_id: row.get(6)?,
                    session_id: None,
                    questions: Vec::new(),
                },
                week_plan: CloseoutStepDto {
                    status: row.get(7)?,
                    message: row.get(8)?,
                    report_id: None,
                    session_id: row.get(9)?,
                    questions: Vec::new(),
                },
                month_plan: CloseoutStepDto {
                    status: row.get(10)?,
                    message: row.get(11)?,
                    report_id: None,
                    session_id: row.get(12)?,
                    questions: Vec::new(),
                },
                closeout_run_id: row.get(13)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn ensure_closeout_table(state: &State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS closeout_runs (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            date              TEXT NOT NULL,
            scope             TEXT NOT NULL,
            score_status      TEXT NOT NULL,
            score_message     TEXT NOT NULL DEFAULT '',
            report_status     TEXT NOT NULL,
            report_message    TEXT NOT NULL DEFAULT '',
            report_id         INTEGER,
            week_status       TEXT NOT NULL,
            week_message      TEXT NOT NULL DEFAULT '',
            week_session_id   INTEGER,
            month_status      TEXT NOT NULL,
            month_message     TEXT NOT NULL DEFAULT '',
            month_session_id  INTEGER,
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_closeout_runs_date ON closeout_runs(date, created_at DESC);
        ",
    )
    .map_err(|e| e.to_string())
}

fn pause_running_timers(state: &State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut stmt = conn
        .prepare("SELECT id FROM records WHERE timer_started_at IS NOT NULL ORDER BY id")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for id in ids {
        record_repo::pause_record_timer(&conn, id, &now).map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn run_score_step(state: &State<'_, DbState>, date: &str) -> CloseoutStepDto {
    let (records, rule_preview, request_json) = {
        let conn = match state.0.lock() {
            Ok(conn) => conn,
            Err(error) => return error_step(error.to_string()),
        };

        match ledger_repo::has_active_entries_for_date(&conn, date) {
            Ok(true) => {
                return skipped_step(
                    "Today's growth ledger already exists, so point recalculation was skipped",
                )
            }
            Ok(false) => {}
            Err(error) => return error_step(error.to_string()),
        }

        let records = match record_repo::get_records_by_date(&conn, date) {
            Ok(records) => records
                .into_iter()
                .filter(|record| record.minutes > 0)
                .collect::<Vec<_>>(),
            Err(error) => return error_step(error.to_string()),
        };

        if records.is_empty() {
            return skipped_step("There are no timed records to analyze today");
        }

        let rule_preview = match build_local_rule_preview(&conn, &records) {
            Ok(preview) => preview,
            Err(error) => return error_step(error),
        };

        let request_json =
            match build_daily_analysis_request_json(&conn, date, &records, &rule_preview) {
                Ok(value) => value,
                Err(error) => return error_step(error),
            };

        (records, rule_preview, request_json)
    };

    let raw_response = match api_proxy::call_scoring_api(state.clone(), request_json).await {
        Ok(response) => response,
        Err(error) => {
            let fallback_summary = format!(
                "AI scoring failed, so the system wrote points using local rules instead. Original issue: {error}"
            );
            let conn = match state.0.lock() {
                Ok(conn) => conn,
                Err(lock_error) => return error_step(lock_error.to_string()),
            };

            return match confirm_score_items(&conn, date, rule_preview, Some(fallback_summary.as_str())) {
                Ok(()) => success_step(
                    "Today's growth points were updated. AI scoring failed, so local rules were applied automatically",
                ),
                Err(confirm_error) => error_step(format!(
                    "AI scoring failed, and local fallback writing also failed: {confirm_error} (original issue: {error})"
                )),
            };
        }
    };

    let parsed: ScoreApiResponse = match parse_score_api_response(&raw_response) {
        Ok(parsed) => parsed,
        Err(error) => {
            let fallback_summary = format!(
                "AI scoring returned an invalid response, so the system wrote points using local rules instead. Original issue: {error}"
            );
            let conn = match state.0.lock() {
                Ok(conn) => conn,
                Err(lock_error) => return error_step(lock_error.to_string()),
            };

            return match confirm_score_items(&conn, date, rule_preview, Some(fallback_summary.as_str())) {
                Ok(()) => success_step(
                    "Today's growth points were updated. AI response was invalid, so local rules were applied automatically",
                ),
                Err(confirm_error) => error_step(format!(
                    "AI response parsing failed, and local fallback writing also failed: {confirm_error} (original issue: {error})"
                )),
            };
        }
    };

    let items = parsed
        .record_results
        .iter()
        .enumerate()
        .map(|(index, item)| score::ScorePreviewItem {
            record_id: records.get(index).map(|record| record.id),
            title: item.title.clone(),
            category: item.category.clone(),
            changes: item
                .changes
                .iter()
                .map(|(dimension_key, change_value)| score::ScorePreviewChange {
                    dimension_key: dimension_key.clone(),
                    change_value: *change_value,
                })
                .collect(),
            difficulty_star: item.difficulty_star,
            confidence: item.confidence,
            reason: item.reason.clone(),
            engine: "rules_api".into(),
        })
        .collect::<Vec<_>>();

    let conn = match state.0.lock() {
        Ok(conn) => conn,
        Err(error) => return error_step(error.to_string()),
    };
    match confirm_score_items(&conn, date, items, Some(parsed.summary.as_str())) {
        Ok(()) => success_step("Today's growth points were updated"),
        Err(error) => error_step(error),
    }
}

async fn run_report_step(
    state: &State<'_, DbState>,
    app_data_dir: &State<'_, AppDataDirState>,
    date: &str,
) -> CloseoutStepDto {
    match daily_insight::generate_period_report(
        state.clone(),
        app_data_dir.clone(),
        "day".into(),
        date.to_string(),
    )
    .await
    {
        Ok(report) => CloseoutStepDto {
            status: "success".into(),
            message: "Daily report generated".into(),
            report_id: Some(report.id),
            session_id: None,
            questions: Vec::new(),
        },
        Err(error) => error_step(error),
    }
}

async fn run_plan_step(
    state: &State<'_, DbState>,
    date: &str,
    period_type: &str,
) -> CloseoutStepDto {
    match plan::refresh_plan_progress(state.clone(), period_type.to_string(), date.to_string())
        .await
    {
        Ok(outcome) if outcome.requires_clarification => CloseoutStepDto {
            status: "needs_clarification".into(),
            message: format!("{period_type} plan needs clarification before it can be applied"),
            report_id: None,
            session_id: Some(outcome.session_id),
            questions: outcome.questions,
        },
        Ok(outcome) => {
            let summary = outcome
                .proposal
                .as_ref()
                .map(|proposal| proposal.ai_summary.trim().to_string())
                .filter(|value| !value.is_empty());

            match plan::apply_plan_ai_update(state.clone(), outcome.session_id) {
                Ok(_) => CloseoutStepDto {
                    status: "success".into(),
                    message: summary
                        .map(|value| format!("{period_type} plan updated: {value}"))
                        .unwrap_or_else(|| format!("{period_type} plan updated")),
                    report_id: None,
                    session_id: Some(outcome.session_id),
                    questions: Vec::new(),
                },
                Err(error) => error_step(error),
            }
        }
        Err(error) => error_step(error),
    }
}

fn build_local_rule_preview(
    conn: &Connection,
    records: &[crate::models::record::Record],
) -> Result<Vec<score::ScorePreviewItem>, String> {
    let rules = rule_repo::get_active_rules(conn).map_err(|e| e.to_string())?;
    Ok(records
        .iter()
        .map(|record| {
            let matched_rule = rules.iter().find(|rule| {
                rule.keywords
                    .split(',')
                    .map(str::trim)
                    .filter(|keyword| !keyword.is_empty())
                    .any(|keyword| record.title.contains(keyword))
            });

            let base_score = match record.minutes {
                0..=15 => 1,
                16..=30 => 2,
                31..=60 => 3,
                61..=90 => 4,
                91..=120 => 5,
                121..=180 => 6,
                _ => 7,
            };
            let multiplier = match record.difficulty_star {
                0 => 1.0,
                1 => 0.9,
                2 => 1.0,
                3 => 1.2,
                _ => 1.0,
            };
            let final_score = (base_score as f64 * multiplier).round() as i32;

            let (category, changes, confidence, reason) = if let Some(rule) = matched_rule {
                let changes = if let Some(secondary_dim) = &rule.secondary_dim {
                    let primary_score = ((final_score as f64) * 0.7).round() as i32;
                    let secondary_score = final_score - primary_score;
                    let mut values = Vec::new();
                    if primary_score > 0 {
                        values.push(score::ScorePreviewChange {
                            dimension_key: rule.primary_dim.clone(),
                            change_value: primary_score,
                        });
                    }
                    if secondary_score > 0 {
                        values.push(score::ScorePreviewChange {
                            dimension_key: secondary_dim.clone(),
                            change_value: secondary_score,
                        });
                    }
                    values
                } else {
                    vec![score::ScorePreviewChange {
                        dimension_key: rule.primary_dim.clone(),
                        change_value: final_score.max(0),
                    }]
                };

                (
                    format!("rule-cache: {}", rule.primary_dim),
                    changes,
                    0.82,
                    "matched local rule cache; waiting for AI verification".to_string(),
                )
            } else {
                (
                    "rule-cache: fallback".to_string(),
                    vec![score::ScorePreviewChange {
                        dimension_key: "willpower".into(),
                        change_value: 1,
                    }],
                    0.35,
                    "no local rule cache matched, using a conservative suggestion".to_string(),
                )
            };

            score::ScorePreviewItem {
                record_id: Some(record.id),
                title: record.title.clone(),
                category,
                changes,
                difficulty_star: record.difficulty_star,
                confidence,
                reason,
                engine: "rules".into(),
            }
        })
        .collect())
}

fn build_daily_analysis_request_json(
    conn: &Connection,
    date: &str,
    records: &[crate::models::record::Record],
    rule_preview: &[score::ScorePreviewItem],
) -> Result<String, String> {
    let dims = dimension_repo::get_all_dimensions(conn).map_err(|e| e.to_string())?;
    let rule_hint_summary = format!(
        "The deterministic rules cache produced suggestions for {} records with a total of {} points. The API must stay within these constraints and return the final result.",
        rule_preview.len(),
        rule_preview
            .iter()
            .flat_map(|item| item.changes.iter().map(|change| change.change_value))
            .sum::<i32>()
    );
    let suggested_totals = rule_preview.iter().fold(
        std::collections::BTreeMap::<String, i32>::new(),
        |mut totals, item| {
            for change in &item.changes {
                *totals.entry(change.dimension_key.clone()).or_insert(0) += change.change_value;
            }
            totals
        },
    );

    serde_json::to_string(&json!({
        "version": "1.0",
        "feedback_mode": "rules_api",
        "stat_dimensions": dims.iter().map(|dim| json!({
            "key": dim.key,
            "name": dim.name,
            "daily_cap": dim.daily_cap,
        })).collect::<Vec<_>>(),
        "score_rules": {
            "time_base": {
                "0-15": 1, "16-30": 2, "31-60": 3, "61-90": 4, "91-120": 5, "121-180": 6, "181+": 7
            },
            "difficulty_multiplier": { "0": 1.0, "1": 0.9, "2": 1.0, "3": 1.2 },
            "max_dims_per_record": 3,
            "allocation_ratio": { "primary": 0.7, "secondary": 0.3 }
        },
        "rule_hints": {
            "source": "deterministic_rules_cache",
            "summary": rule_hint_summary,
            "suggested_totals": suggested_totals,
            "record_hints": rule_preview.iter().enumerate().map(|(index, item)| json!({
                "record_index": index,
                "title": item.title,
                "category": item.category,
                "suggested_dimensions": item.changes.iter().map(|change| change.dimension_key.clone()).collect::<Vec<_>>(),
                "suggested_changes": item.changes.iter().map(|change| (change.dimension_key.clone(), change.change_value)).collect::<std::collections::BTreeMap<_, _>>(),
                "confidence": item.confidence,
                "reason": item.reason,
            })).collect::<Vec<_>>()
        },
        "records": records.iter().map(|record| json!({
            "title": record.title,
            "minutes": record.minutes,
            "difficulty_star": record.difficulty_star,
        })).collect::<Vec<_>>(),
        "date": date
    }))
    .map_err(|e| e.to_string())
}

fn confirm_score_items(
    conn: &Connection,
    date: &str,
    items: Vec<score::ScorePreviewItem>,
    summary: Option<&str>,
) -> Result<(), String> {
    if ledger_repo::has_active_entries_for_date(conn, date).map_err(|e| e.to_string())? {
        return Err("Today's growth ledger has already been written".into());
    }

    let dimensions = dimension_repo::get_all_dimensions(conn).map_err(|e| e.to_string())?;
    let mut remaining_caps = std::collections::HashMap::new();
    for dimension in dimensions {
        remaining_caps.insert(dimension.key, dimension.daily_cap.max(0));
    }

    let mut normalized_items = Vec::with_capacity(items.len());
    for item in items {
        let mut changes = Vec::new();
        for change in item.changes {
            let Some(remaining) = remaining_caps.get_mut(&change.dimension_key) else {
                continue;
            };
            let allowed = change.change_value.max(0).min(*remaining);
            *remaining -= allowed;
            if allowed > 0 {
                changes.push(score::ScorePreviewChange {
                    dimension_key: change.dimension_key,
                    change_value: allowed,
                });
            }
        }
        normalized_items.push(score::ScorePreviewItem { changes, ..item });
    }

    if !normalized_items
        .iter()
        .any(|item| item.changes.iter().any(|change| change.change_value > 0))
    {
        return Err("There are no growth changes left to write".into());
    }

    for item in &normalized_items {
        for change in &item.changes {
            ledger_repo::insert_ledger(
                conn,
                date,
                item.record_id,
                &change.dimension_key,
                change.change_value,
                &item.title,
                &item.reason,
                Some(item.confidence),
                &item.engine,
            )
            .map_err(|e| e.to_string())?;
        }
    }

    daily_review_repo::recalculate_review(conn, date, summary).map_err(|e| e.to_string())?;
    Ok(())
}

fn insert_closeout_run(
    conn: &Connection,
    date: &str,
    scope: &str,
    score: &CloseoutStepDto,
    report: &CloseoutStepDto,
    week: &CloseoutStepDto,
    month: &CloseoutStepDto,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO closeout_runs (
            date, scope, score_status, score_message, report_status, report_message, report_id,
            week_status, week_message, week_session_id, month_status, month_message, month_session_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            date,
            scope,
            score.status,
            score.message,
            report.status,
            report.message,
            report.report_id,
            week.status,
            week.message,
            week.session_id,
            month.status,
            month.message,
            month.session_id,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

fn skipped_step(message: &str) -> CloseoutStepDto {
    CloseoutStepDto {
        status: "skipped".into(),
        message: message.into(),
        report_id: None,
        session_id: None,
        questions: Vec::new(),
    }
}

fn success_step(message: &str) -> CloseoutStepDto {
    CloseoutStepDto {
        status: "success".into(),
        message: message.into(),
        report_id: None,
        session_id: None,
        questions: Vec::new(),
    }
}

fn error_step<E: ToString>(error: E) -> CloseoutStepDto {
    CloseoutStepDto {
        status: "error".into(),
        message: error.to_string(),
        report_id: None,
        session_id: None,
        questions: Vec::new(),
    }
}

fn parse_score_api_response(raw_response: &str) -> Result<ScoreApiResponse, String> {
    ai_response_service::parse_ai_json(raw_response, "scoring api")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_score_api_response_rejects_empty_content() {
        let result = parse_score_api_response("   ");
        assert!(result.is_err());
    }

    #[test]
    fn parse_score_api_response_strips_code_fences() {
        let payload = r#"```json
{
  "summary": "ok",
  "record_results": []
}
```"#;

        let parsed = parse_score_api_response(payload).expect("should parse fenced json");
        assert_eq!(parsed.summary, "ok");
        assert!(parsed.record_results.is_empty());
    }
}
