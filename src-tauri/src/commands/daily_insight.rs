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
        let context = build_insight_context(&conn, &report_kind, &period_type, &range)?;
        let context_json = serde_json::to_string_pretty(&context).map_err(|e| e.to_string())?;
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

    let response_payload =
        match api_proxy::execute_daily_insight_api_request(&state, request_json.clone()).await {
            Ok(payload) => payload,
            Err(error) => {
                let conn = state.0.lock().map_err(|e| e.to_string())?;
                insert_report(
                    &conn,
                    NewInsightReport {
                        report_kind: &report_kind,
                        period_type: &period_type,
                        start_date: &fmt_date(range.start_date),
                        end_date: &fmt_date(range.end_date),
                        title: "生成失败",
                        summary: &error,
                        content_json: &json!({ "error": error }),
                        raw_response: "",
                        context_snapshot_id: Some(snapshot_id),
                        status: "error",
                        error_message: Some(&error),
                        memory_patch_json: None,
                        memory_patch_apply_status: None,
                        memory_patch_apply_message: None,
                    },
                )?;
                return Err(error);
            }
        };

    let parsed = match parse_insight_response(&response_payload) {
        Ok(value) => value,
        Err(error) => {
            let conn = state.0.lock().map_err(|e| e.to_string())?;
            insert_report(
                &conn,
                NewInsightReport {
                    report_kind: &report_kind,
                    period_type: &period_type,
                    start_date: &fmt_date(range.start_date),
                    end_date: &fmt_date(range.end_date),
                    title: "解析失败",
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
        }
    };

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

fn build_insight_context(
    conn: &Connection,
    report_kind: &str,
    period_type: &str,
    range: &PeriodRange,
) -> Result<Value, String> {
    let start_date = fmt_date(range.start_date);
    let end_date = fmt_date(range.end_date);
    let mode = if report_kind == "tarot" {
        "tarot"
    } else {
        period_type
    };
    let personal_context =
        personal_memory_repo::build_personal_context_pack(conn, &start_date, mode)?;
    let records = query_records(conn, &start_date, &end_date)?;
    let ledger = query_ledger(conn, &start_date, &end_date)?;
    let journals = query_journals(conn, &start_date, &end_date)?;
    let bonds = query_bond_entries(conn, &start_date, &end_date)?;
    let plans = query_plans(conn, &start_date, &end_date)?;
    let previous_reports = query_previous_reports(conn, 12)?;

    let evidence_index = collect_evidence(
        &records,
        &ledger,
        &journals,
        &bonds,
        &plans,
        &previous_reports,
    );

    Ok(json!({
        "schema_version": "1.0",
        "report_kind": report_kind,
        "period_type": period_type,
        "start_date": start_date,
        "end_date": end_date,
        "generated_at": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "personal_context": personal_context,
        "evidence_index": evidence_index,
        "period_data": {
            "records": records,
            "ledger": ledger,
            "journals": journals,
            "bond_entries": bonds,
            "plans": plans,
            "previous_insight_reports": previous_reports
        },
        "constraints": {
            "must_cite_evidence_ids": true,
            "no_evidence_policy": "write insufficient_evidence instead of guessing",
            "memory_patch_schema": "PersonalMemoryPatch v1"
        }
    }))
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

fn query_previous_reports(conn: &Connection, limit: i64) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, report_kind, period_type, start_date, end_date, title, summary, created_at
             FROM insight_reports
             WHERE status = 'success'
             ORDER BY created_at DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([limit.max(1)], |row| {
            let id: i64 = row.get(0)?;
            Ok(json!({
                "evidence_id": format!("insight_report:{id}"),
                "id": id,
                "report_kind": row.get::<_, String>(1)?,
                "period_type": row.get::<_, String>(2)?,
                "start_date": row.get::<_, String>(3)?,
                "end_date": row.get::<_, String>(4)?,
                "title": row.get::<_, String>(5)?,
                "summary": row.get::<_, String>(6)?,
                "created_at": row.get::<_, String>(7)?,
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
        .unwrap_or("未命名证据")
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
        "今日心理原型牌"
    } else {
        match period_type {
            "week" => "本周报告",
            "month" => "本月报告",
            _ => "每日报告",
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
            r#"```json
            {
              "schema_version": "1.0",
              "title": "今日牌",
              "summary": "稳住节奏",
              "report": { "card_name": "力量" },
              "memory_delta": { "schema_version": "1.0", "profile_updates": null, "memory_operations": [] }
            }
            ```"#,
        )
        .expect("parse");
        assert_eq!(parsed.title, "今日牌");
        assert_eq!(parsed.report["card_name"], "力量");
    }

    #[test]
    fn deleting_report_removes_orphan_context_snapshot() {
        let conn = Connection::open_in_memory().expect("db");
        run_migrations(&conn).expect("migrations");
        let snapshot_id =
            insert_context_snapshot(&conn, "report", "day", "2026-06-13", "2026-06-13", "{}")
                .expect("snapshot");
        let report_id = insert_test_report(&conn, snapshot_id, "日报");

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
        let first_report_id = insert_test_report(&conn, snapshot_id, "日报 A");
        let _second_report_id = insert_test_report(&conn, snapshot_id, "日报 B");

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
