use crate::commands::api_proxy;
use crate::db::connection::{AppDataDirState, DbState};
use crate::db::repositories::personal_memory_repo;
use crate::services::{ai_response_service, rag_memory_service};
use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct InsightReportDto {
    pub id: i64,
    pub report_kind: String,
    pub period_type: String,
    pub start_date: String,
    pub end_date: String,
    pub title: String,
    pub summary: String,
    pub content_json: Value,
    pub context_snapshot_id: Option<i64>,
    pub status: String,
    pub error_message: Option<String>,
    pub memory_patch_apply_status: Option<String>,
    pub memory_patch_apply_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InsightContextSnapshotDto {
    pub id: i64,
    pub report_kind: String,
    pub period_type: String,
    pub start_date: String,
    pub end_date: String,
    pub context_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarInsightHistoryDto {
    pub date: String,
    pub tarot: Option<InsightReportDto>,
    pub daily_report: Option<InsightReportDto>,
    pub week_report: Option<InsightReportDto>,
    pub month_report: Option<InsightReportDto>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParsedInsightResponse {
    #[serde(default)]
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    report: Value,
    memory_delta: Option<Value>,
}

#[derive(Debug, Clone)]
struct PeriodRange {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize)]
struct InsightConstraints<'a> {
    must_cite_evidence_ids: bool,
    no_evidence_policy: &'a str,
    memory_patch_schema: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct InsightPeriodData {
    records: Vec<Value>,
    ledger: Vec<Value>,
    journals: Vec<Value>,
    bond_entries: Vec<Value>,
    plans: Vec<Value>,
    previous_insight_reports: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct InsightContextPayload<'a> {
    schema_version: &'a str,
    constraints: InsightConstraints<'a>,
    personal_context: crate::models::personal_memory::PersonalContextPack,
    evidence_index: Vec<Value>,
    period_data: InsightPeriodData,
    start_date: String,
    end_date: String,
    report_kind: &'a str,
    period_type: &'a str,
    generated_at: String,
}

#[tauri::command]
pub async fn generate_tarot_insight(
    state: State<'_, DbState>,
    app_data_dir: State<'_, AppDataDirState>,
    date: String,
) -> Result<InsightReportDto, String> {
    generate_insight_report(
        state,
        app_data_dir,
        "tarot".to_string(),
        "day".to_string(),
        date,
    )
    .await
}

#[tauri::command]
pub async fn generate_period_report(
    state: State<'_, DbState>,
    app_data_dir: State<'_, AppDataDirState>,
    period_type: String,
    anchor_date: String,
) -> Result<InsightReportDto, String> {
    generate_insight_report(
        state,
        app_data_dir,
        "report".to_string(),
        period_type,
        anchor_date,
    )
    .await
}

#[tauri::command]
pub fn list_insight_reports(
    state: State<DbState>,
    report_kind: Option<String>,
    period_type: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<InsightReportDto>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    list_reports(
        &conn,
        report_kind.as_deref(),
        period_type.as_deref(),
        limit.unwrap_or(20),
    )
}

#[tauri::command]
pub fn get_insight_context_snapshot(
    state: State<DbState>,
    snapshot_id: i64,
) -> Result<InsightContextSnapshotDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_context_snapshot(&conn, snapshot_id)?
        .ok_or_else(|| "insight context snapshot not found".to_string())
}

#[tauri::command]
pub fn delete_insight_report(state: State<DbState>, report_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    delete_report_and_orphan_context(&conn, report_id)
}

#[tauri::command]
pub fn get_calendar_insight_history(
    state: State<DbState>,
    date: String,
) -> Result<CalendarInsightHistoryDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    Ok(CalendarInsightHistoryDto {
        date: date.clone(),
        tarot: find_report_covering_date(&conn, "tarot", "day", &date)?,
        daily_report: find_report_covering_date(&conn, "report", "day", &date)?,
        week_report: find_report_covering_date(&conn, "report", "week", &date)?,
        month_report: find_report_covering_date(&conn, "report", "month", &date)?,
    })
}

async fn generate_insight_report(
    state: State<'_, DbState>,
    app_data_dir: State<'_, AppDataDirState>,
    report_kind: String,
    period_type: String,
    anchor_date: String,
) -> Result<InsightReportDto, String> {
    let range = resolve_period_range(&period_type, &anchor_date)?;
    let (snapshot_id, request_json) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let context_json = build_insight_context(&conn, &report_kind, &period_type, &range)?;
        let snapshot_id = insert_context_snapshot(
            &conn,
            &report_kind,
            &period_type,
            &fmt_date(range.start_date),
            &fmt_date(range.end_date),
            &context_json,
        )?;
        (snapshot_id, context_json)
    };

    let mut last_error: Option<(String, String)> = None;
    let mut last_payload: Option<String> = None;
    let mut last_parsed: Option<ParsedInsightResponse> = None;

    // One retry if the model returns an overly short-but-valid payload.
    for attempt in 0..2 {
        let response_payload = match api_proxy::execute_daily_insight_api_request(
            &state,
            request_json.clone(),
            resolve_insight_task_kind(&report_kind, &period_type),
        )
        .await
        {
            Ok(payload) => payload,
            Err(error) => {
                last_error = Some(("Generation Failed".to_string(), error.clone()));
                last_payload = Some(String::new());
                break;
            }
        };

        let parsed = match parse_insight_response(&response_payload) {
            Ok(value) => value,
            Err(error) => {
                last_error = Some(("Parse Failed".to_string(), error.clone()));
                last_payload = Some(response_payload);
                break;
            }
        };

        let parsed = ParsedInsightResponse {
            report: normalize_report_payload(&report_kind, parsed.report),
            ..parsed
        };

        match validate_report_payload(&report_kind, &parsed.report) {
            Ok(()) => {
                last_payload = Some(response_payload);
                last_parsed = Some(parsed);
                break;
            }
            Err(error) => {
                // Retry only for "too short" failures.
                let can_retry = attempt == 0 && error.to_ascii_lowercase().contains("too short");
                if can_retry {
                    continue;
                }
                last_error = Some(("Schema Validation Failed".to_string(), error));
                last_payload = Some(response_payload);
                break;
            }
        }
    }

    let Some(parsed) = last_parsed else {
        let (title, error) = last_error.unwrap_or_else(|| {
            (
                "Generation Failed".to_string(),
                "daily insight generation failed without a detailed error".to_string(),
            )
        });
        let response_payload = last_payload.unwrap_or_default();
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        insert_report(
            &conn,
            NewInsightReport {
                report_kind: &report_kind,
                period_type: &period_type,
                start_date: &fmt_date(range.start_date),
                end_date: &fmt_date(range.end_date),
                title: &title,
                summary: &error,
                content_json: &json!({ "error": error, "raw_response": response_payload }),
                raw_response: &response_payload,
                context_snapshot_id: Some(snapshot_id),
                status: "error",
                error_message: Some(&error),
                memory_patch_json: None,
                memory_patch_apply_status: None,
                memory_patch_apply_message: None,
            },
        )?;
        return Err(error);
    };

    let response_payload = last_payload.unwrap_or_default();

    let full_content = serde_json::to_value(&parsed).map_err(|e| e.to_string())?;
    let memory_patch_json = parsed
        .memory_delta
        .as_ref()
        .filter(|value| !value.is_null())
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| e.to_string())?;

    let report_id = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        insert_report(
            &conn,
            NewInsightReport {
                report_kind: &report_kind,
                period_type: &period_type,
                start_date: &fmt_date(range.start_date),
                end_date: &fmt_date(range.end_date),
                title: if parsed.title.trim().is_empty() {
                    default_title(&report_kind, &period_type)
                } else {
                    &parsed.title
                },
                summary: &parsed.summary,
                content_json: &full_content,
                raw_response: &response_payload,
                context_snapshot_id: Some(snapshot_id),
                status: "success",
                error_message: None,
                memory_patch_json: memory_patch_json.as_deref(),
                memory_patch_apply_status: None,
                memory_patch_apply_message: None,
            },
        )?
    };

    if let Some(patch_json) = memory_patch_json.as_deref() {
        let patch_result = {
            let mut conn = state.0.lock().map_err(|e| e.to_string())?;
            let result = personal_memory_repo::apply_memory_patch(
                &mut conn,
                patch_json,
                &format!("insight_report:{report_id}"),
            );
            let (mut status, mut message) = match result {
                Ok(result) => (result.apply_status, result.message),
                Err(error) => ("rejected".to_string(), error),
            };
            let rag_memory_dir = app_data_dir.0.join("rag_memory");
            if let Err(error) = rag_memory_service::write_patch_run_file(
                &rag_memory_dir,
                &Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                patch_json,
            ) {
                if status == "applied" {
                    status = "partial".to_string();
                }
                if message.trim().is_empty() {
                    message = format!("patch mirror write failed: {error}");
                } else {
                    message = format!("{message}; patch mirror write failed: {error}");
                }
            }
            if let Err(error) = rag_memory_service::rebuild_rag_memory_files(&conn, &rag_memory_dir)
            {
                if status == "applied" {
                    status = "partial".to_string();
                }
                if message.trim().is_empty() {
                    message = format!("rag_memory rebuild failed: {error}");
                } else {
                    message = format!("{message}; rag_memory rebuild failed: {error}");
                }
            }
            update_report_patch_status(&conn, report_id, &status, &message)?;
            (status, message)
        };
        let _ = patch_result;
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_report_by_id(&conn, report_id)?.ok_or_else(|| "insight report was not saved".to_string())
}

fn resolve_insight_task_kind(report_kind: &str, period_type: &str) -> api_proxy::AiTaskKind {
    if report_kind == "tarot" {
        api_proxy::AiTaskKind::Tarot
    } else {
        match period_type {
            "week" => api_proxy::AiTaskKind::PeriodReportWeek,
            "month" => api_proxy::AiTaskKind::PeriodReportMonth,
            _ => api_proxy::AiTaskKind::PeriodReportDay,
        }
    }
}

fn build_insight_context(
    conn: &Connection,
    report_kind: &str,
    period_type: &str,
    range: &PeriodRange,
) -> Result<String, String> {
    let start_date = fmt_date(range.start_date);
    let end_date = fmt_date(range.end_date);
    let mode = if report_kind == "tarot" {
        "tarot"
    } else {
        period_type
    };
    let all_memory_items = personal_memory_repo::list_active_memory_items(conn)?;
    let mut personal_context =
        personal_memory_repo::build_personal_context_pack(conn, &start_date, mode)?;
    let records = query_records(conn, &start_date, &end_date)?;
    let ledger = query_ledger(conn, &start_date, &end_date)?;
    let journals = query_journals(conn, &start_date, &end_date)?;
    let bonds = query_bond_entries(conn, &start_date, &end_date)?;
    let plans = query_plans(conn, &start_date, &end_date)?;
    let previous_reports = query_previous_reports(conn, 12)?;

    let mut query_texts = Vec::new();
    for value in records
        .iter()
        .chain(journals.iter())
        .chain(bonds.iter())
        .chain(plans.iter())
    {
        for key in [
            "title",
            "content",
            "description",
            "item_title",
            "cycle_title",
            "cycle_summary",
            "person_name",
            "relation_label",
        ] {
            if let Some(text) = value.get(key).and_then(Value::as_str) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    query_texts.push(trimmed.to_string());
                }
            }
        }
    }
    personal_memory_repo::fill_query_relevant_memories(
        &mut personal_context,
        &all_memory_items,
        &query_texts,
    );

    let evidence_index = collect_evidence(
        &records,
        &ledger,
        &journals,
        &bonds,
        &plans,
        &previous_reports,
    );

    let payload = InsightContextPayload {
        schema_version: "1.0",
        constraints: InsightConstraints {
            must_cite_evidence_ids: true,
            no_evidence_policy: "write insufficient_evidence instead of guessing",
            memory_patch_schema: "PersonalMemoryPatch v1",
        },
        personal_context,
        evidence_index,
        period_data: InsightPeriodData {
            records,
            ledger,
            journals,
            bond_entries: bonds,
            plans,
            previous_insight_reports: previous_reports,
        },
        start_date,
        end_date,
        report_kind,
        period_type,
        generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    serialize_insight_context(&payload)
}

/// Serialize the top-level insight context for DeepSeek API calls.
///
/// Cache-sensitive: struct field order must be preserved byte-for-byte across
/// repeated requests. Never route this payload through `serde_json::to_value`
/// or `json!` before sending — that destroys order and collapses cache hits.
fn serialize_insight_context(payload: &InsightContextPayload<'_>) -> Result<String, String> {
    serde_json::to_string(payload).map_err(|e| e.to_string())
}

#[cfg(test)]
fn cache_stable_prefix(request_json: &str) -> &str {
    request_json
        .rfind("\"generated_at\"")
        .map(|idx| &request_json[..idx])
        .unwrap_or(request_json)
}

fn query_records(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, date, title, minutes, difficulty_star, parent_id, is_completed,
                    completed_at, elapsed_seconds, timer_mode, countdown_target_seconds
             FROM records
             WHERE date BETWEEN ?1 AND ?2
             ORDER BY date, parent_id, created_at, id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            let id: i64 = row.get(0)?;
            Ok(json!({
                "evidence_id": format!("record:{id}"),
                "id": id,
                "date": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "minutes": row.get::<_, i32>(3)?,
                "difficulty_star": row.get::<_, i32>(4)?,
                "parent_id": row.get::<_, Option<i64>>(5)?,
                "is_completed": row.get::<_, i32>(6)? == 1,
                "completed_at": row.get::<_, Option<String>>(7)?,
                "elapsed_seconds": row.get::<_, i64>(8)?,
                "timer_mode": row.get::<_, String>(9)?,
                "countdown_target_seconds": row.get::<_, Option<i32>>(10)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn query_ledger(conn: &Connection, start_date: &str, end_date: &str) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, date, record_id, dimension_key, change_value, source_title, reason, confidence, engine
             FROM stat_ledger
             WHERE is_rollback = 0 AND date BETWEEN ?1 AND ?2
             ORDER BY date, id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            let id: i64 = row.get(0)?;
            Ok(json!({
                "evidence_id": format!("ledger:{id}"),
                "id": id,
                "date": row.get::<_, String>(1)?,
                "record_id": row.get::<_, Option<i64>>(2)?,
                "dimension_key": row.get::<_, String>(3)?,
                "change_value": row.get::<_, i32>(4)?,
                "source_title": row.get::<_, String>(5)?,
                "reason": row.get::<_, String>(6)?,
                "confidence": row.get::<_, Option<f64>>(7)?,
                "engine": row.get::<_, String>(8)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn query_journals(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, entry_date, title, content, mood
             FROM daily_journals
             WHERE entry_date BETWEEN ?1 AND ?2
             ORDER BY entry_date",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            let id: i64 = row.get(0)?;
            Ok(json!({
                "evidence_id": format!("journal:{id}"),
                "id": id,
                "entry_date": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "content": row.get::<_, String>(3)?,
                "mood": row.get::<_, String>(4)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn query_bond_entries(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.person_id, e.entry_date, e.title, e.content,
                    p.name, p.relation_label, p.score
             FROM bond_entries e
             JOIN bond_people p ON p.id = e.person_id
             WHERE e.entry_date BETWEEN ?1 AND ?2
             ORDER BY e.entry_date, e.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            let id: i64 = row.get(0)?;
            Ok(json!({
                "evidence_id": format!("bond_entry:{id}"),
                "id": id,
                "person_id": row.get::<_, i64>(1)?,
                "entry_date": row.get::<_, String>(2)?,
                "title": row.get::<_, String>(3)?,
                "content": row.get::<_, String>(4)?,
                "person_name": row.get::<_, String>(5)?,
                "relation_label": row.get::<_, String>(6)?,
                "bond_score": row.get::<_, i32>(7)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn query_plans(conn: &Connection, start_date: &str, end_date: &str) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.period_type, c.start_date, c.end_date, c.title, c.summary, c.ai_summary,
                    i.id, i.title, i.description, i.dimension_key, i.progress_percent, i.ai_comment, i.is_completed
             FROM plan_cycles c
             LEFT JOIN plan_items i ON i.cycle_id = c.id
             WHERE c.start_date <= ?2 AND c.end_date >= ?1
             ORDER BY c.period_type, c.start_date, i.sort_order, i.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![start_date, end_date], |row| {
        let cycle_id: i64 = row.get(0)?;
        let item_id: Option<i64> = row.get(7)?;
        Ok(json!({
            "evidence_id": item_id.map(|id| format!("plan_item:{id}")).unwrap_or_else(|| format!("plan_cycle:{cycle_id}")),
            "cycle_id": cycle_id,
            "period_type": row.get::<_, String>(1)?,
            "start_date": row.get::<_, String>(2)?,
            "end_date": row.get::<_, String>(3)?,
            "cycle_title": row.get::<_, String>(4)?,
            "cycle_summary": row.get::<_, String>(5)?,
            "cycle_ai_summary": row.get::<_, String>(6)?,
            "item_id": item_id,
            "item_title": row.get::<_, Option<String>>(8)?,
            "description": row.get::<_, Option<String>>(9)?,
            "dimension_key": row.get::<_, Option<String>>(10)?,
            "progress_percent": row.get::<_, Option<i32>>(11)?,
            "ai_comment": row.get::<_, Option<String>>(12)?,
            "is_completed": row.get::<_, Option<i32>>(13)?.unwrap_or(0) == 1,
        }))
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn extract_tarot_hint(content_json: &str, title: &str, summary: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(content_json) {
        if let Some(card_name) = value
            .pointer("/report/card_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            return Some(card_name.to_string());
        }
        if let Some(theme) = value
            .pointer("/report/psychological_theme")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            return Some(theme.chars().take(64).collect());
        }
    }
    let fallback = if !title.trim().is_empty() {
        title.trim()
    } else {
        summary.trim()
    };
    if fallback.is_empty() {
        None
    } else {
        Some(fallback.chars().take(64).collect())
    }
}

fn query_previous_reports(conn: &Connection, limit: i64) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, report_kind, period_type, start_date, end_date, title, summary, content_json, created_at
             FROM insight_reports
             WHERE status = 'success'
             ORDER BY created_at DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([limit.max(1)], |row| {
            let id: i64 = row.get(0)?;
            let report_kind: String = row.get(1)?;
            let title: String = row.get(5)?;
            let summary: String = row.get(6)?;
            let content_json: String = row.get(7)?;
            let tarot_hint = if report_kind == "tarot" {
                extract_tarot_hint(&content_json, &title, &summary)
            } else {
                None
            };
            Ok(json!({
                "evidence_id": format!("insight_report:{id}"),
                "id": id,
                "report_kind": report_kind,
                "period_type": row.get::<_, String>(2)?,
                "start_date": row.get::<_, String>(3)?,
                "end_date": row.get::<_, String>(4)?,
                "title": title,
                "summary": summary,
                "tarot_hint": tarot_hint,
                "created_at": row.get::<_, String>(8)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn collect_evidence(
    groups: &[Value],
    ledger: &[Value],
    journals: &[Value],
    bonds: &[Value],
    plans: &[Value],
    previous_reports: &[Value],
) -> Vec<Value> {
    groups
        .iter()
        .chain(ledger)
        .chain(journals)
        .chain(bonds)
        .chain(plans)
        .chain(previous_reports)
        .filter_map(|value| {
            value.get("evidence_id").and_then(Value::as_str).map(|id| {
                json!({
                    "evidence_id": id,
                    "source_type": id.split(':').next().unwrap_or("unknown"),
                    "label": evidence_label(value)
                })
            })
        })
        .collect()
}

fn evidence_label(value: &Value) -> String {
    value
        .get("title")
        .or_else(|| value.get("source_title"))
        .or_else(|| value.get("item_title"))
        .or_else(|| value.get("cycle_title"))
        .and_then(Value::as_str)
        .unwrap_or("unnamed-evidence")
        .to_string()
}
struct NewInsightReport<'a> {
    report_kind: &'a str,
    period_type: &'a str,
    start_date: &'a str,
    end_date: &'a str,
    title: &'a str,
    summary: &'a str,
    content_json: &'a Value,
    raw_response: &'a str,
    context_snapshot_id: Option<i64>,
    status: &'a str,
    error_message: Option<&'a str>,
    memory_patch_json: Option<&'a str>,
    memory_patch_apply_status: Option<&'a str>,
    memory_patch_apply_message: Option<&'a str>,
}

fn insert_context_snapshot(
    conn: &Connection,
    report_kind: &str,
    period_type: &str,
    start_date: &str,
    end_date: &str,
    context_json: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO insight_context_snapshots (
            report_kind, period_type, start_date, end_date, context_json
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![report_kind, period_type, start_date, end_date, context_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

fn insert_report(conn: &Connection, report: NewInsightReport<'_>) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO insight_reports (
            report_kind,
            period_type,
            start_date,
            end_date,
            title,
            summary,
            content_json,
            raw_response,
            context_snapshot_id,
            status,
            error_message,
            memory_patch_json,
            memory_patch_apply_status,
            memory_patch_apply_message
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            report.report_kind,
            report.period_type,
            report.start_date,
            report.end_date,
            report.title,
            report.summary,
            serde_json::to_string(report.content_json).map_err(|e| e.to_string())?,
            report.raw_response,
            report.context_snapshot_id,
            report.status,
            report.error_message,
            report.memory_patch_json,
            report.memory_patch_apply_status,
            report.memory_patch_apply_message,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

fn update_report_patch_status(
    conn: &Connection,
    report_id: i64,
    status: &str,
    message: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE insight_reports
         SET memory_patch_apply_status = ?2,
             memory_patch_apply_message = ?3
         WHERE id = ?1",
        params![report_id, status, message],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn delete_report_and_orphan_context(conn: &Connection, report_id: i64) -> Result<(), String> {
    let context_snapshot_id = conn
        .query_row(
            "SELECT context_snapshot_id FROM insight_reports WHERE id = ?1",
            params![report_id],
            |row| row.get::<_, Option<i64>>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "insight report not found".to_string())?;

    conn.execute(
        "DELETE FROM insight_reports WHERE id = ?1",
        params![report_id],
    )
    .map_err(|e| e.to_string())?;

    if let Some(snapshot_id) = context_snapshot_id {
        let remaining_refs: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM insight_reports WHERE context_snapshot_id = ?1",
                params![snapshot_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if remaining_refs == 0 {
            conn.execute(
                "DELETE FROM insight_context_snapshots WHERE id = ?1",
                params![snapshot_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn get_report_by_id(conn: &Connection, report_id: i64) -> Result<Option<InsightReportDto>, String> {
    conn.query_row(
        "SELECT id, report_kind, period_type, start_date, end_date, title, summary,
                content_json, context_snapshot_id, status, error_message,
                memory_patch_apply_status, memory_patch_apply_message, created_at
         FROM insight_reports
         WHERE id = ?1",
        params![report_id],
        map_report_row,
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn find_report_covering_date(
    conn: &Connection,
    report_kind: &str,
    period_type: &str,
    date: &str,
) -> Result<Option<InsightReportDto>, String> {
    conn.query_row(
        "SELECT id, report_kind, period_type, start_date, end_date, title, summary,
                content_json, context_snapshot_id, status, error_message,
                memory_patch_apply_status, memory_patch_apply_message, created_at
         FROM insight_reports
         WHERE report_kind = ?1
           AND period_type = ?2
           AND start_date <= ?3
           AND end_date >= ?3
           AND status = 'success'
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
        params![report_kind, period_type, date],
        map_report_row,
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn list_reports(
    conn: &Connection,
    report_kind: Option<&str>,
    period_type: Option<&str>,
    limit: i64,
) -> Result<Vec<InsightReportDto>, String> {
    let kind_filter = report_kind.unwrap_or("%");
    let period_filter = period_type.unwrap_or("%");
    let mut stmt = conn
        .prepare(
            "SELECT id, report_kind, period_type, start_date, end_date, title, summary,
                    content_json, context_snapshot_id, status, error_message,
                    memory_patch_apply_status, memory_patch_apply_message, created_at
             FROM insight_reports
             WHERE report_kind LIKE ?1 AND period_type LIKE ?2
             ORDER BY created_at DESC, id DESC
             LIMIT ?3",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            params![kind_filter, period_filter, limit.max(1)],
            map_report_row,
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn get_context_snapshot(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<Option<InsightContextSnapshotDto>, String> {
    conn.query_row(
        "SELECT id, report_kind, period_type, start_date, end_date, context_json, created_at
         FROM insight_context_snapshots
         WHERE id = ?1",
        params![snapshot_id],
        |row| {
            Ok(InsightContextSnapshotDto {
                id: row.get(0)?,
                report_kind: row.get(1)?,
                period_type: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                context_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn map_report_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<InsightReportDto> {
    let content_json_raw: String = row.get(7)?;
    Ok(InsightReportDto {
        id: row.get(0)?,
        report_kind: row.get(1)?,
        period_type: row.get(2)?,
        start_date: row.get(3)?,
        end_date: row.get(4)?,
        title: row.get(5)?,
        summary: row.get(6)?,
        content_json: serde_json::from_str(&content_json_raw).unwrap_or_else(|_| json!({})),
        context_snapshot_id: row.get(8)?,
        status: row.get(9)?,
        error_message: row.get(10)?,
        memory_patch_apply_status: row.get(11)?,
        memory_patch_apply_message: row.get(12)?,
        created_at: row.get(13)?,
    })
}

fn normalize_entry_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Array(items) => {
            let values = items
                .iter()
                .filter_map(normalize_entry_text)
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values.join("; "))
            }
        }
        Value::Object(map) => {
            const TITLE_KEYS: &[&str] = &["item", "title", "name", "label", "topic"];
            const BODY_KEYS: &[&str] = &[
                "description",
                "detail",
                "summary",
                "text",
                "content",
                "message",
                "reason",
                "value",
            ];
            let title = TITLE_KEYS
                .iter()
                .filter_map(|key| map.get(*key))
                .find_map(normalize_entry_text)
                .unwrap_or_default();
            let body = BODY_KEYS
                .iter()
                .filter_map(|key| map.get(*key))
                .find_map(normalize_entry_text)
                .unwrap_or_default();
            if !title.is_empty() && !body.is_empty() {
                if body.contains(&title) {
                    return Some(body);
                }
                return Some(format!("{title}: {body}"));
            }
            if !body.is_empty() {
                return Some(body);
            }
            if !title.is_empty() {
                return Some(title);
            }
            map.iter()
                .filter(|(key, _)| *key != "evidence_ids" && *key != "evidence_id")
                .find_map(|(_, inner)| normalize_entry_text(inner))
        }
        _ => None,
    }
}

fn normalize_string_field(report: &mut serde_json::Map<String, Value>, key: &str) {
    if let Some(text) = report.get(key).and_then(normalize_entry_text) {
        report.insert(key.to_string(), Value::String(text));
    }
}

fn normalize_list_field(report: &mut serde_json::Map<String, Value>, key: &str) {
    if let Some(value) = report.get(key) {
        let list = match value {
            Value::Array(items) => items
                .iter()
                .filter_map(normalize_entry_text)
                .map(Value::String)
                .collect::<Vec<_>>(),
            _ => normalize_entry_text(value)
                .map(|text| vec![Value::String(text)])
                .unwrap_or_default(),
        };
        report.insert(key.to_string(), Value::Array(list));
    }
}

fn normalize_report_payload(report_kind: &str, report: Value) -> Value {
    let mut map = match report {
        Value::Object(map) => map,
        other => return other,
    };

    if report_kind == "tarot" {
        for key in [
            "warm_quote",
            "encouragement",
            "psychological_theme",
            "body_signal",
            "deeper_reading",
            "risk_reminder",
            "card_name",
            "card_mark",
            "archetype",
        ] {
            normalize_string_field(&mut map, key);
        }
        normalize_list_field(&mut map, "action");
    } else {
        for key in [
            "emotional_reflection",
            "comfort_message",
            "pressure_sources",
            "inner_pattern",
            "self_compassion",
            "plan_progress",
            "leverage_points",
            "not_enough_data",
            "completed",
            "unfinished",
        ] {
            normalize_string_field(&mut map, key);
        }
        for key in ["gentle_questions", "small_next_steps"] {
            normalize_list_field(&mut map, key);
        }
    }

    Value::Object(map)
}

fn validate_report_payload(report_kind: &str, report: &Value) -> Result<(), String> {
    let map = report
        .as_object()
        .ok_or_else(|| "daily insight report payload must be a JSON object".to_string())?;

    let required_string_fields: &[&str] = if report_kind == "tarot" {
        &[
            "card_name",
            "archetype",
            "psychological_theme",
            "body_signal",
            "warm_quote",
            "encouragement",
            "risk_reminder",
            "deeper_reading",
        ]
    } else {
        &[
            "emotional_reflection",
            "comfort_message",
            "pressure_sources",
            "inner_pattern",
            "self_compassion",
        ]
    };

    for field in required_string_fields {
        let value = map.get(*field).and_then(Value::as_str).map(str::trim).unwrap_or("");
        if value.is_empty() {
            return Err(format!("AI response is missing required field: {field}"));
        }
    }

    // Soft minimum length checks to discourage overly short outputs.
    // The upstream caller may retry once when hitting these.
    let min_length: &[(&str, usize)] = if report_kind == "tarot" {
        &[
            ("encouragement", 80),
            ("deeper_reading", 180),
            ("risk_reminder", 40),
        ]
    } else {
        &[
            ("emotional_reflection", 120),
            ("inner_pattern", 160),
            ("comfort_message", 80),
            ("pressure_sources", 80),
            ("self_compassion", 80),
        ]
    };
    for (field, min_chars) in min_length {
        let value = map.get(*field).and_then(Value::as_str).map(str::trim).unwrap_or("");
        if !value.is_empty() && value.chars().count() < *min_chars {
            return Err(format!("AI response field is too short: {field}"));
        }
    }

    let required_list_fields: &[&str] = if report_kind == "tarot" {
        &["action"]
    } else {
        &["gentle_questions", "small_next_steps"]
    };

    for field in required_list_fields {
        let is_valid = map
            .get(*field)
            .and_then(Value::as_array)
            .map(|items| items.iter().any(|item| item.as_str().map(str::trim).unwrap_or("").len() > 0))
            .unwrap_or(false);
        if !is_valid {
            return Err(format!("AI response is missing required list field: {field}"));
        }
    }

    if report_kind != "tarot" {
        for field in ["gentle_questions", "small_next_steps"] {
            let count = map
                .get(field)
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter(|item| item.as_str().map(str::trim).unwrap_or("").len() > 0)
                        .count()
                })
                .unwrap_or(0);
            if count < 3 {
                return Err(format!("AI response field is too short: {field}"));
            }
        }
    } else if let Some(actions) = map.get("action").and_then(Value::as_array) {
        let count = actions
            .iter()
            .filter(|item| item.as_str().map(str::trim).unwrap_or("").len() > 0)
            .count();
        if count < 3 {
            return Err("AI response field is too short: action".to_string());
        }
    }

    Ok(())
}

fn parse_insight_response(raw_response: &str) -> Result<ParsedInsightResponse, String> {
    let parsed: ParsedInsightResponse =
        ai_response_service::parse_ai_json(raw_response, "daily insight api")?;
    if parsed.report.is_null() {
        return Err("AI response is missing report".into());
    }
    Ok(parsed)
}

fn resolve_period_range(period_type: &str, anchor_date: &str) -> Result<PeriodRange, String> {
    let anchor = NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d")
        .map_err(|_| "invalid date, expected YYYY-MM-DD".to_string())?;
    match period_type {
        "day" => Ok(PeriodRange {
            start_date: anchor,
            end_date: anchor,
        }),
        "week" => {
            let start_date =
                anchor - Duration::days(anchor.weekday().num_days_from_monday() as i64);
            Ok(PeriodRange {
                start_date,
                end_date: start_date + Duration::days(6),
            })
        }
        "month" => {
            let start_date = NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), 1)
                .ok_or_else(|| "invalid month".to_string())?;
            let end_date = if anchor.month() == 12 {
                NaiveDate::from_ymd_opt(anchor.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(anchor.year(), anchor.month() + 1, 1)
            }
            .and_then(|date| date.pred_opt())
            .ok_or_else(|| "invalid month end".to_string())?;
            Ok(PeriodRange {
                start_date,
                end_date,
            })
        }
        _ => Err("period_type must be day, week, or month".into()),
    }
}

fn fmt_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn default_title<'a>(report_kind: &str, period_type: &'a str) -> &'a str {
    if report_kind == "tarot" {
        "Daily Tarot"
    } else {
        match period_type {
            "week" => "Weekly Report",
            "month" => "Monthly Report",
            _ => "Daily Report",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn resolves_week_range_from_anchor_date() {
        let range = resolve_period_range("week", "2026-06-13").expect("range");
        assert_eq!(fmt_date(range.start_date), "2026-06-08");
        assert_eq!(fmt_date(range.end_date), "2026-06-14");
    }

    #[test]
    fn resolves_month_range_for_leap_year() {
        let range = resolve_period_range("month", "2024-02-10").expect("range");
        assert_eq!(fmt_date(range.start_date), "2024-02-01");
        assert_eq!(fmt_date(range.end_date), "2024-02-29");
    }

    #[test]
    fn parses_fenced_ai_json() {
        let parsed = parse_insight_response(
            r#"{
              "schema_version": "1.0",
              "title": "today-card",
              "summary": "steady pace",
              "report": { "card_name": "strength" },
              "memory_delta": { "schema_version": "1.0", "profile_updates": null, "memory_operations": [] }
            }"#,
        )
        .expect("parse");
        assert_eq!(parsed.title, "today-card");
        assert_eq!(parsed.report["card_name"], "strength");
    }
    #[test]
    fn normalizes_report_object_entries_into_strings() {
        let normalized = normalize_report_payload(
            "report",
            json!({
                "completed": [
                    { "description": "reviewed math", "evidence_ids": ["record:1"] }
                ],
                "unfinished": [
                    { "item": "vocab", "detail": "not checked in", "evidence_ids": ["plan_item:1"] }
                ],
                "emotional_reflection": {
                    "description": "felt pressure but stayed in motion",
                    "evidence_ids": ["insight_report:1"]
                }
            }),
        );
        assert_eq!(normalized["completed"], "reviewed math");
        assert_eq!(normalized["unfinished"], "vocab: not checked in");
        assert_eq!(normalized["emotional_reflection"], "felt pressure but stayed in motion");
    }
    #[test]
    fn normalizes_tarot_action_objects_into_plain_text() {
        let normalized = normalize_report_payload(
            "tarot",
            json!({
                "card_name": { "title": "hermit" },
                "action": [
                    { "description": "take 5 minutes to name what you resist" }
                ],
                "risk_reminder": { "description": "do not overpack the plan because of anxiety" }
            }),
        );
        assert_eq!(normalized["card_name"], "hermit");
        assert_eq!(normalized["action"][0], "take 5 minutes to name what you resist");
        assert_eq!(normalized["risk_reminder"], "do not overpack the plan because of anxiety");
    }

    #[test]
    fn report_validation_rejects_missing_psychological_fields() {
        let error = validate_report_payload(
            "report",
            &json!({
                "comfort_message": "be gentle with yourself",
                "gentle_questions": ["what hurt today?"],
                "small_next_steps": ["drink water"]
            }),
        )
        .expect_err("missing required fields");

        assert!(error.contains("emotional_reflection"));
    }

    #[test]
    fn report_validation_accepts_complete_daily_payload() {
        validate_report_payload(
            "report",
            &json!({
                "emotional_reflection": "You kept moving under pressure even when your attention was pulled in multiple directions. You showed steadiness instead of chasing perfection. That matters more than the number of boxes checked today.",
                "comfort_message": "You do not need to do everything tonight. Choose one small step that protects tomorrow's energy, and let the rest be unfinished without guilt.",
                "pressure_sources": "Exam pressure is present, but the deeper weight is the fear of falling behind and being judged by yourself. The tight timeline makes every decision feel high-stakes, which increases inner friction.",
                "inner_pattern": "When uncertainty rises, you try to regain control by overplanning. The plan becomes a shield against anxiety, but it also creates more pressure and makes starting feel heavier.",
                "self_compassion": "Rest is allowed, and rest is also part of progress. You can be serious about growth without being harsh to yourself, especially when the day already asked a lot from you.",
                "gentle_questions": ["what is the smallest honest next step?", "what felt heavy today?", "what would kindness look like tonight?"],
                "small_next_steps": ["open the notebook and write one line", "drink water and stretch for two minutes", "pick one task to defer until tomorrow"]
            }),
        )
        .expect("valid payload");
    }
    #[test]
    fn serializes_insight_context_in_cache_safe_order() {
        use crate::models::personal_memory::{
            PersonalContextPack, PersonalMemoryOverview, PersonalProfile,
        };

        let payload = InsightContextPayload {
            schema_version: "1.0",
            constraints: InsightConstraints {
                must_cite_evidence_ids: true,
                no_evidence_policy: "write insufficient_evidence instead of guessing",
                memory_patch_schema: "PersonalMemoryPatch v1",
            },
            personal_context: PersonalContextPack {
                schema_version: "1.0".to_string(),
                profile: PersonalProfile::default(),
                high_priority_memories: vec![],
                relevant_memories: vec![],
                recent_memories: vec![],
                query_relevant_memories: vec![],
                overview: PersonalMemoryOverview {
                    total_items: 0,
                    active_items: 0,
                    pending_items: 0,
                    rejected_items: 0,
                    top_items: vec![],
                },
                mode: "tarot".to_string(),
                date: "2026-06-13".to_string(),
            },
            evidence_index: vec![],
            period_data: InsightPeriodData {
                records: vec![],
                ledger: vec![],
                journals: vec![],
                bond_entries: vec![],
                plans: vec![],
                previous_insight_reports: vec![],
            },
            start_date: "2026-06-13".to_string(),
            end_date: "2026-06-13".to_string(),
            report_kind: "tarot",
            period_type: "day",
            generated_at: "2026-06-13 12:00:00".to_string(),
        };

        let json = serialize_insight_context(&payload).expect("serialize");
        assert!(
            json.starts_with("{\"schema_version\":\"1.0\",\"constraints\":"),
            "unexpected prefix: {}",
            &json[..json.len().min(120)]
        );

        let ordered_fields = [
            "schema_version",
            "constraints",
            "personal_context",
            "evidence_index",
            "period_data",
            "start_date",
            "end_date",
            "report_kind",
            "period_type",
            "generated_at",
        ];
        let mut last_pos = 0usize;
        for field in ordered_fields {
            let needle = format!("\"{field}\"");
            let pos = json[last_pos..]
                .find(&needle)
                .map(|offset| last_pos + offset)
                .unwrap_or_else(|| panic!("missing field {field}"));
            assert!(
                pos >= last_pos,
                "field {field} is out of cache-safe order in {json}"
            );
            last_pos = pos;
        }

        let generated_at_pos = json.find("\"generated_at\"").expect("generated_at");
        let personal_context_pos = json
            .find("\"personal_context\"")
            .expect("personal_context");
        let period_data_pos = json.find("\"period_data\"").expect("period_data");
        assert!(generated_at_pos > personal_context_pos);
        assert!(generated_at_pos > period_data_pos);
        assert!(json.ends_with("\"generated_at\":\"2026-06-13 12:00:00\"}"));
    }

    #[test]
    fn builds_repeatable_insight_context_prefix() {
        use crate::db::repositories::personal_memory_repo;
        use crate::models::personal_memory::PersonalProfile;

        let conn = Connection::open_in_memory().expect("db");
        run_migrations(&conn).expect("migrations");
        personal_memory_repo::save_personal_profile(
            &conn,
            &PersonalProfile {
                birthday: "2000-01-01".to_string(),
                personality: "steady".to_string(),
                experiences: "cache prefix test".to_string(),
                personal_notes: "stable serialization".to_string(),
                updated_at: None,
            },
        )
        .expect("profile");

        let range = resolve_period_range("day", "2026-06-13").expect("range");
        let first = build_insight_context(&conn, "tarot", "day", &range).expect("first");
        let second = build_insight_context(&conn, "tarot", "day", &range).expect("second");

        assert_eq!(
            cache_stable_prefix(&first),
            cache_stable_prefix(&second),
            "prefix before generated_at must be byte-identical across rebuilds"
        );
        assert!(
            first.contains("\"generated_at\"") && second.contains("\"generated_at\""),
            "generated_at must remain the final cache-volatile field"
        );
    }

    #[test]
    fn deleting_report_removes_orphan_context_snapshot() {
        let conn = Connection::open_in_memory().expect("db");
        run_migrations(&conn).expect("migrations");
        let snapshot_id =
            insert_context_snapshot(&conn, "report", "day", "2026-06-13", "2026-06-13", "{}")
                .expect("snapshot");
        let report_id = insert_test_report(&conn, snapshot_id, "daily-report");

        delete_report_and_orphan_context(&conn, report_id).expect("delete");

        assert!(get_report_by_id(&conn, report_id)
            .expect("report lookup")
            .is_none());
        assert!(get_context_snapshot(&conn, snapshot_id)
            .expect("snapshot lookup")
            .is_none());
    }

    #[test]
    fn deleting_report_keeps_shared_context_snapshot() {
        let conn = Connection::open_in_memory().expect("db");
        run_migrations(&conn).expect("migrations");
        let snapshot_id =
            insert_context_snapshot(&conn, "report", "day", "2026-06-13", "2026-06-13", "{}")
                .expect("snapshot");
        let first_report_id = insert_test_report(&conn, snapshot_id, "daily-report-a");
        let _second_report_id = insert_test_report(&conn, snapshot_id, "daily-report-b");

        delete_report_and_orphan_context(&conn, first_report_id).expect("delete");

        assert!(get_context_snapshot(&conn, snapshot_id)
            .expect("snapshot lookup")
            .is_some());
    }

    fn insert_test_report(conn: &Connection, snapshot_id: i64, title: &str) -> i64 {
        insert_report(
            conn,
            NewInsightReport {
                report_kind: "report",
                period_type: "day",
                start_date: "2026-06-13",
                end_date: "2026-06-13",
                title,
                summary: "",
                content_json: &json!({ "report": {} }),
                raw_response: "{}",
                context_snapshot_id: Some(snapshot_id),
                status: "success",
                error_message: None,
                memory_patch_json: None,
                memory_patch_apply_status: None,
                memory_patch_apply_message: None,
            },
        )
        .expect("report")
    }
}




