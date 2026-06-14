use crate::db::connection::DbState;
use crate::db::repositories::{personal_memory_repo, record_repo, setting_repo};
use crate::models::personal_memory::PersonalProfile;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Clone, Deserialize)]
pub struct ImportRecord {
    pub title: String,
    pub minutes: i32,
    pub date: String,
    pub difficulty_star: Option<i32>,
    pub parent_id: Option<i64>,
    pub timer_mode: Option<String>,
    pub countdown_target_seconds: Option<i32>,
}

#[derive(Default, Deserialize)]
struct ImportPayload {
    #[serde(default)]
    records: Vec<ImportRecord>,
    personal_profile: Option<ImportPersonalProfile>,
}

#[derive(Default, Deserialize)]
struct ImportPersonalProfile {
    birthday: Option<String>,
    personality: Option<String>,
    experiences: Option<String>,
    personal_notes: Option<String>,
}

#[derive(Serialize)]
pub struct ImportResult {
    pub imported: i32,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
struct ExportRecord {
    id: i64,
    date: String,
    title: String,
    minutes: i32,
    difficulty_star: i32,
    parent_id: Option<i64>,
    is_completed: bool,
    completed_at: Option<String>,
    elapsed_seconds: i64,
    timer_mode: String,
    countdown_target_seconds: Option<i32>,
    timer_started_at: Option<String>,
}

#[derive(Serialize)]
struct ExportRecordTimerSession {
    id: i64,
    record_id: i64,
    started_at: String,
    ended_at: Option<String>,
    duration_seconds: Option<i64>,
}

#[derive(Serialize)]
struct ExportLedgerEntry {
    id: i64,
    date: String,
    record_id: Option<i64>,
    dimension_key: String,
    change_value: i32,
    source_title: String,
    reason: String,
    confidence: Option<f64>,
    engine: String,
    is_rollback: bool,
    rollback_ref: Option<i64>,
}

#[derive(Serialize)]
struct ExportPlanCycle {
    id: i64,
    period_type: String,
    start_date: String,
    end_date: String,
    title: String,
    summary: String,
    ai_summary: String,
    last_ai_run_at: Option<String>,
}

#[derive(Serialize)]
struct ExportPlanItem {
    id: i64,
    cycle_id: i64,
    title: String,
    description: String,
    dimension_key: Option<String>,
    progress_percent: i32,
    ai_comment: String,
    sort_order: i32,
    is_completed: bool,
}

#[derive(Serialize)]
struct ExportPlanAiSession {
    id: i64,
    cycle_id: i64,
    status: String,
    request_payload: String,
    response_payload: Option<String>,
    questions_json: String,
    answers_json: String,
    proposal_json: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ExportPlans {
    cycles: Vec<ExportPlanCycle>,
    items: Vec<ExportPlanItem>,
    ai_sessions: Vec<ExportPlanAiSession>,
}

#[derive(Serialize)]
struct ExportBondPerson {
    id: i64,
    name: String,
    relation_label: String,
    score: i32,
    note: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ExportBondEntry {
    id: i64,
    person_id: i64,
    entry_date: String,
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ExportBondData {
    people: Vec<ExportBondPerson>,
    entries: Vec<ExportBondEntry>,
}

#[derive(Serialize)]
struct ExportDailyJournal {
    id: i64,
    entry_date: String,
    title: String,
    content: String,
    mood: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ExportPersonalMemoryItem {
    id: i64,
    memory_type: String,
    title: String,
    summary: String,
    detail: String,
    tags_json: String,
    importance: i32,
    confidence: f64,
    first_seen_date: Option<String>,
    last_seen_date: Option<String>,
    status: String,
    supersedes_id: Option<i64>,
    created_by: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ExportPersonalMemorySource {
    id: i64,
    memory_id: i64,
    source_type: String,
    source_id: String,
    source_date: Option<String>,
    evidence_id: String,
    excerpt: String,
    created_at: String,
}

#[derive(Serialize)]
struct ExportPersonalMemoryEvent {
    id: i64,
    memory_id: Option<i64>,
    event_type: String,
    payload_json: String,
    created_at: String,
}

#[derive(Serialize)]
struct ExportPersonalMemoryPatchRun {
    id: i64,
    source_context_id: String,
    patch_json: String,
    validation_status: String,
    apply_status: String,
    rejected_reason: Option<String>,
    applied_operations: i32,
    rejected_operations: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ExportInsightContextSnapshot {
    id: i64,
    report_kind: String,
    period_type: String,
    start_date: String,
    end_date: String,
    context_json: String,
    created_at: String,
}

#[derive(Serialize)]
struct ExportInsightReport {
    id: i64,
    report_kind: String,
    period_type: String,
    start_date: String,
    end_date: String,
    title: String,
    summary: String,
    content_json: String,
    context_snapshot_id: Option<i64>,
    status: String,
    error_message: Option<String>,
    memory_patch_json: Option<String>,
    memory_patch_apply_status: Option<String>,
    memory_patch_apply_message: Option<String>,
    created_at: String,
}

#[derive(Serialize)]
struct ExportInsights {
    context_snapshots: Vec<ExportInsightContextSnapshot>,
    reports: Vec<ExportInsightReport>,
}

#[derive(Serialize)]
struct ExportCloseoutRun {
    id: i64,
    date: String,
    scope: String,
    score_status: String,
    score_message: String,
    report_status: String,
    report_message: String,
    report_id: Option<i64>,
    week_status: String,
    week_message: String,
    week_session_id: Option<i64>,
    month_status: String,
    month_message: String,
    month_session_id: Option<i64>,
    created_at: String,
}

#[derive(Serialize)]
struct ExportSettingsPublic {
    scoring_engine: String,
    api_base_url: String,
    api_model: String,
    api_key_configured: bool,
}

#[derive(Serialize)]
struct ExportDataEnvelope {
    schema_version: String,
    exported_at: String,
    records: Vec<ExportRecord>,
    record_timer_sessions: Vec<ExportRecordTimerSession>,
    ledger: Vec<ExportLedgerEntry>,
    plans: ExportPlans,
    bond: ExportBondData,
    journals: Vec<ExportDailyJournal>,
    personal_profile: PersonalProfile,
    personal_memory_items: Vec<ExportPersonalMemoryItem>,
    personal_memory_events: Vec<ExportPersonalMemoryEvent>,
    personal_memory_sources: Vec<ExportPersonalMemorySource>,
    personal_memory_patch_runs: Vec<ExportPersonalMemoryPatchRun>,
    insights: ExportInsights,
    closeout_runs: Vec<ExportCloseoutRun>,
    settings_public: ExportSettingsPublic,
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
                    errors.push(format!("row {} is missing title or date", idx + 2));
                    continue;
                }
                if let Err(e) = record_repo::create_record(&conn, date, title, minutes, 0) {
                    errors.push(format!("row {}: {}", idx + 2, e));
                } else {
                    imported += 1;
                }
            }
            Err(e) => {
                errors.push(format!("row {} parse error: {}", idx + 2, e));
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

    let payload = parse_import_payload(&content)?;

    if let Some(profile) = payload.personal_profile {
        personal_memory_repo::save_personal_profile(
            &conn,
            &PersonalProfile {
                birthday: profile.birthday.unwrap_or_default(),
                personality: profile.personality.unwrap_or_default(),
                experiences: profile.experiences.unwrap_or_default(),
                personal_notes: profile.personal_notes.unwrap_or_default(),
                updated_at: None,
            },
        )?;
    }

    for (idx, record) in payload.records.iter().enumerate() {
        if record.title.is_empty() || record.date.is_empty() {
            errors.push(format!("item {} is missing title or date", idx + 1));
            continue;
        }
        let difficulty_star = record.difficulty_star.unwrap_or(0);
        if let Err(e) = record_repo::create_record_with_options(
            &conn,
            &record.date,
            &record.title,
            record.minutes,
            difficulty_star,
            record.parent_id,
            "stopwatch",
            None,
        ) {
            errors.push(format!("item {}: {}", idx + 1, e));
        } else {
            imported += 1;
        }
    }

    Ok(ImportResult { imported, errors })
}

#[tauri::command]
pub fn export_data(state: State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    export_data_json(&conn)
}

fn export_data_json(conn: &Connection) -> Result<String, String> {
    let payload = build_export_envelope(conn)?;
    serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())
}

fn parse_import_payload(content: &str) -> Result<ImportPayload, String> {
    let value: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    match value {
        Value::Array(_) => Ok(ImportPayload {
            records: serde_json::from_value(value).map_err(|e| e.to_string())?,
            personal_profile: None,
        }),
        Value::Object(ref map) => Ok(ImportPayload {
            records: map
                .get("records")
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or_default(),
            personal_profile: map
                .get("personal_profile")
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| e.to_string())?,
        }),
        _ => Err("json import expects a record array or an export object".into()),
    }
}

fn build_export_envelope(conn: &Connection) -> Result<ExportDataEnvelope, String> {
    Ok(ExportDataEnvelope {
        schema_version: "1.0".into(),
        exported_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        records: export_records(conn)?,
        record_timer_sessions: export_record_timer_sessions(conn)?,
        ledger: export_ledger(conn)?,
        plans: ExportPlans {
            cycles: export_plan_cycles(conn)?,
            items: export_plan_items(conn)?,
            ai_sessions: export_plan_ai_sessions(conn)?,
        },
        bond: ExportBondData {
            people: export_bond_people(conn)?,
            entries: export_bond_entries(conn)?,
        },
        journals: export_daily_journals(conn)?,
        personal_profile: personal_memory_repo::get_personal_profile(conn)?,
        personal_memory_items: export_personal_memory_items(conn)?,
        personal_memory_events: export_personal_memory_events(conn)?,
        personal_memory_sources: export_personal_memory_sources(conn)?,
        personal_memory_patch_runs: export_personal_memory_patch_runs(conn)?,
        insights: ExportInsights {
            context_snapshots: export_insight_context_snapshots(conn)?,
            reports: export_insight_reports(conn)?,
        },
        closeout_runs: export_closeout_runs(conn)?,
        settings_public: export_settings_public(conn)?,
    })
}

fn export_records(conn: &Connection) -> Result<Vec<ExportRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                date,
                title,
                minutes,
                difficulty_star,
                parent_id,
                is_completed,
                completed_at,
                elapsed_seconds,
                timer_mode,
                countdown_target_seconds,
                timer_started_at
             FROM records
             ORDER BY date, created_at, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportRecord {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            minutes: row.get(3)?,
            difficulty_star: row.get(4)?,
            parent_id: row.get(5)?,
            is_completed: row.get(6)?,
            completed_at: row.get(7)?,
            elapsed_seconds: row.get(8)?,
            timer_mode: row.get(9)?,
            countdown_target_seconds: row.get(10)?,
            timer_started_at: row.get(11)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_record_timer_sessions(conn: &Connection) -> Result<Vec<ExportRecordTimerSession>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                record_id,
                started_at,
                ended_at,
                duration_seconds
             FROM record_timer_sessions
             ORDER BY record_id, started_at, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportRecordTimerSession {
            id: row.get(0)?,
            record_id: row.get(1)?,
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            duration_seconds: row.get(4)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_ledger(conn: &Connection) -> Result<Vec<ExportLedgerEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                date,
                record_id,
                dimension_key,
                change_value,
                source_title,
                reason,
                confidence,
                engine,
                is_rollback,
                rollback_ref
             FROM stat_ledger
             ORDER BY date, created_at, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportLedgerEntry {
            id: row.get(0)?,
            date: row.get(1)?,
            record_id: row.get(2)?,
            dimension_key: row.get(3)?,
            change_value: row.get(4)?,
            source_title: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            reason: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            confidence: row.get(7)?,
            engine: row.get(8)?,
            is_rollback: row.get(9)?,
            rollback_ref: row.get(10)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_plan_cycles(conn: &Connection) -> Result<Vec<ExportPlanCycle>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                period_type,
                start_date,
                end_date,
                title,
                summary,
                ai_summary,
                last_ai_run_at
             FROM plan_cycles
             ORDER BY period_type, start_date, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportPlanCycle {
            id: row.get(0)?,
            period_type: row.get(1)?,
            start_date: row.get(2)?,
            end_date: row.get(3)?,
            title: row.get(4)?,
            summary: row.get(5)?,
            ai_summary: row.get(6)?,
            last_ai_run_at: row.get(7)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_plan_items(conn: &Connection) -> Result<Vec<ExportPlanItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                cycle_id,
                title,
                description,
                dimension_key,
                progress_percent,
                ai_comment,
                sort_order,
                is_completed
             FROM plan_items
             ORDER BY cycle_id, sort_order, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportPlanItem {
            id: row.get(0)?,
            cycle_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            dimension_key: row.get(4)?,
            progress_percent: row.get(5)?,
            ai_comment: row.get(6)?,
            sort_order: row.get(7)?,
            is_completed: row.get(8)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_plan_ai_sessions(conn: &Connection) -> Result<Vec<ExportPlanAiSession>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                cycle_id,
                status,
                request_payload,
                response_payload,
                questions_json,
                answers_json,
                proposal_json,
                created_at,
                updated_at
             FROM plan_ai_sessions
             ORDER BY cycle_id, created_at, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportPlanAiSession {
            id: row.get(0)?,
            cycle_id: row.get(1)?,
            status: row.get(2)?,
            request_payload: row.get(3)?,
            response_payload: row.get(4)?,
            questions_json: row.get(5)?,
            answers_json: row.get(6)?,
            proposal_json: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_bond_people(conn: &Connection) -> Result<Vec<ExportBondPerson>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                name,
                relation_label,
                score,
                note,
                created_at,
                updated_at
             FROM bond_people
             ORDER BY updated_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportBondPerson {
            id: row.get(0)?,
            name: row.get(1)?,
            relation_label: row.get(2)?,
            score: row.get(3)?,
            note: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_bond_entries(conn: &Connection) -> Result<Vec<ExportBondEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                person_id,
                entry_date,
                title,
                content,
                created_at,
                updated_at
             FROM bond_entries
             ORDER BY entry_date DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportBondEntry {
            id: row.get(0)?,
            person_id: row.get(1)?,
            entry_date: row.get(2)?,
            title: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_daily_journals(conn: &Connection) -> Result<Vec<ExportDailyJournal>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                entry_date,
                title,
                content,
                mood,
                created_at,
                updated_at
             FROM daily_journals
             ORDER BY entry_date DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportDailyJournal {
            id: row.get(0)?,
            entry_date: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            mood: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_personal_memory_items(conn: &Connection) -> Result<Vec<ExportPersonalMemoryItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                memory_type,
                title,
                summary,
                detail,
                tags_json,
                importance,
                confidence,
                first_seen_date,
                last_seen_date,
                status,
                supersedes_id,
                created_by,
                created_at,
                updated_at
             FROM personal_memory_items
             ORDER BY updated_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportPersonalMemoryItem {
            id: row.get(0)?,
            memory_type: row.get(1)?,
            title: row.get(2)?,
            summary: row.get(3)?,
            detail: row.get(4)?,
            tags_json: row.get(5)?,
            importance: row.get(6)?,
            confidence: row.get(7)?,
            first_seen_date: row.get(8)?,
            last_seen_date: row.get(9)?,
            status: row.get(10)?,
            supersedes_id: row.get(11)?,
            created_by: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_personal_memory_sources(conn: &Connection) -> Result<Vec<ExportPersonalMemorySource>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                memory_id,
                source_type,
                source_id,
                source_date,
                evidence_id,
                excerpt,
                created_at
             FROM personal_memory_sources
             ORDER BY memory_id, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportPersonalMemorySource {
            id: row.get(0)?,
            memory_id: row.get(1)?,
            source_type: row.get(2)?,
            source_id: row.get(3)?,
            source_date: row.get(4)?,
            evidence_id: row.get(5)?,
            excerpt: row.get(6)?,
            created_at: row.get(7)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_personal_memory_events(conn: &Connection) -> Result<Vec<ExportPersonalMemoryEvent>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                memory_id,
                event_type,
                payload_json,
                created_at
             FROM personal_memory_events
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportPersonalMemoryEvent {
            id: row.get(0)?,
            memory_id: row.get(1)?,
            event_type: row.get(2)?,
            payload_json: row.get(3)?,
            created_at: row.get(4)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_personal_memory_patch_runs(conn: &Connection) -> Result<Vec<ExportPersonalMemoryPatchRun>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                source_context_id,
                patch_json,
                validation_status,
                apply_status,
                rejected_reason,
                applied_operations,
                rejected_operations,
                created_at,
                updated_at
             FROM personal_memory_patch_runs
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
        Ok(ExportPersonalMemoryPatchRun {
            id: row.get(0)?,
            source_context_id: row.get(1)?,
            patch_json: row.get(2)?,
            validation_status: row.get(3)?,
            apply_status: row.get(4)?,
            rejected_reason: row.get(5)?,
            applied_operations: row.get(6)?,
            rejected_operations: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_insight_context_snapshots(conn: &Connection) -> Result<Vec<ExportInsightContextSnapshot>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, report_kind, period_type, start_date, end_date, context_json, created_at
             FROM insight_context_snapshots
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ExportInsightContextSnapshot {
                id: row.get(0)?,
                report_kind: row.get(1)?,
                period_type: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                context_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_insight_reports(conn: &Connection) -> Result<Vec<ExportInsightReport>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                report_kind,
                period_type,
                start_date,
                end_date,
                title,
                summary,
                content_json,
                context_snapshot_id,
                status,
                error_message,
                memory_patch_json,
                memory_patch_apply_status,
                memory_patch_apply_message,
                created_at
             FROM insight_reports
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ExportInsightReport {
                id: row.get(0)?,
                report_kind: row.get(1)?,
                period_type: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                title: row.get(5)?,
                summary: row.get(6)?,
                content_json: row.get(7)?,
                context_snapshot_id: row.get(8)?,
                status: row.get(9)?,
                error_message: row.get(10)?,
                memory_patch_json: row.get(11)?,
                memory_patch_apply_status: row.get(12)?,
                memory_patch_apply_message: row.get(13)?,
                created_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_closeout_runs(conn: &Connection) -> Result<Vec<ExportCloseoutRun>, String> {
    let has_table: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'closeout_runs'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_table == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare(
            "SELECT
                id,
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
                created_at
             FROM closeout_runs
             ORDER BY created_at, id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ExportCloseoutRun {
                id: row.get(0)?,
                date: row.get(1)?,
                scope: row.get(2)?,
                score_status: row.get(3)?,
                score_message: row.get(4)?,
                report_status: row.get(5)?,
                report_message: row.get(6)?,
                report_id: row.get(7)?,
                week_status: row.get(8)?,
                week_message: row.get(9)?,
                week_session_id: row.get(10)?,
                month_status: row.get(11)?,
                month_message: row.get(12)?,
                month_session_id: row.get(13)?,
                created_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn export_settings_public(conn: &Connection) -> Result<ExportSettingsPublic, String> {
    let scoring_engine = setting_repo::get_setting(conn, "scoring_engine")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "rules_api".to_string());
    let api_base_url = setting_repo::get_setting(conn, "api_base_url")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let api_model = setting_repo::get_setting(conn, "api_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "gpt-4o-mini".to_string());
    let api_key_configured = setting_repo::get_setting(conn, "api_key")
        .map_err(|e| e.to_string())?
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

    Ok(ExportSettingsPublic {
        scoring_engine,
        api_base_url,
        api_model,
        api_key_configured,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn parse_import_payload_accepts_legacy_array() {
        let payload = parse_import_payload(r#"[{"title":"A","minutes":10,"date":"2026-06-13"}]"#)
            .expect("parse payload");

        assert_eq!(payload.records.len(), 1);
        assert_eq!(payload.records[0].title, "A");
    }

    #[test]
    fn export_payload_hides_api_key_and_includes_profile() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");
        setting_repo::set_setting(&conn, "api_base_url", "https://api.example.com").expect("base url");
        setting_repo::set_setting(&conn, "api_model", "gpt-4o-mini").expect("model");
        setting_repo::set_setting(&conn, "api_key", "secret-value").expect("key");
        personal_memory_repo::save_personal_profile(
            &conn,
            &PersonalProfile {
                birthday: "1998-01-02".into(),
                personality: "calm".into(),
                experiences: "studied design".into(),
                personal_notes: "likes routine".into(),
                updated_at: None,
            },
        )
        .expect("save profile");

        let payload = build_export_envelope(&conn).expect("export payload");

        assert_eq!(payload.personal_profile.birthday, "1998-01-02");
        assert!(payload.settings_public.api_key_configured);

        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(!json.contains("secret-value"));
    }
}
