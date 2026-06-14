use crate::commands::api_proxy;
use crate::db::connection::DbState;
use crate::db::repositories::{dimension_repo, ledger_repo, plan_repo, record_repo};
use crate::models::plan::PlanCycle;
use crate::services::ai_response_service;
use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct PlanCycleDto {
    pub id: i64,
    pub period_type: String,
    pub start_date: String,
    pub end_date: String,
    pub title: String,
    pub summary: String,
    pub ai_summary: String,
    pub last_ai_run_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanItemDto {
    pub id: i64,
    pub cycle_id: i64,
    pub title: String,
    pub description: String,
    pub dimension_key: Option<String>,
    pub progress_percent: i32,
    pub ai_comment: String,
    pub sort_order: i32,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanGrowthDimensionDto {
    pub key: String,
    pub name: String,
    pub total: i32,
    pub daily_cap: i32,
    pub max_total: i32,
    pub progress_percent: i32,
    pub headline: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanGrowthSnapshotDto {
    pub start_date: String,
    pub end_date: String,
    pub total_days: i32,
    pub active_days: i32,
    pub analyzed_days: i32,
    pub record_count: i32,
    pub total_change: i32,
    pub dimensions: Vec<PlanGrowthDimensionDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoalProgressSummaryDto {
    pub total_items: i32,
    pub completed_items: i32,
    pub active_goal_count: i32,
    pub average_progress_percent: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelatedWeekPlanDto {
    pub cycle_id: i64,
    pub start_date: String,
    pub end_date: String,
    pub title: String,
    pub summary: String,
    pub average_progress: i32,
    pub total_items: i32,
    pub completed_items: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanSnapshotDto {
    pub cycle: PlanCycleDto,
    pub items: Vec<PlanItemDto>,
    pub goal_progress: GoalProgressSummaryDto,
    pub growth: PlanGrowthSnapshotDto,
    pub related_weeks: Vec<RelatedWeekPlanDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarPlanHistoryItemDto {
    pub cycle: PlanCycleDto,
    pub goal_progress: GoalProgressSummaryDto,
    pub items: Vec<PlanItemDto>,
    pub is_historical: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarPlanHistoryDto {
    pub date: String,
    pub week_plan: Option<CalendarPlanHistoryItemDto>,
    pub month_plan: Option<CalendarPlanHistoryItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAiProposalItemDto {
    pub title: String,
    pub description: String,
    pub dimension_key: Option<String>,
    pub progress_percent: i32,
    pub ai_comment: String,
    pub sort_order: i32,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAiProposalDto {
    pub title: String,
    pub summary: String,
    pub ai_summary: String,
    pub items: Vec<PlanAiProposalItemDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanAiOutcomeDto {
    pub session_id: i64,
    pub status: String,
    pub requires_clarification: bool,
    pub questions: Vec<String>,
    pub proposal: Option<PlanAiProposalDto>,
}

#[derive(Debug, Deserialize)]
struct PlanAiApiResponse {
    requires_clarification: bool,
    questions: Vec<String>,
    proposal: Option<PlanAiProposalDto>,
}

#[derive(Debug, Serialize)]
struct PlanAiRequest<'a> {
    mode: &'a str,
    stage: &'a str,
    cycle: &'a PlanCycleDto,
    items: &'a [PlanItemDto],
    growth: &'a PlanGrowthSnapshotDto,
    related_weeks: &'a [RelatedWeekPlanDto],
    clarification: Option<PlanClarificationPayload<'a>>,
}

#[derive(Debug, Serialize)]
struct PlanClarificationPayload<'a> {
    questions: &'a [String],
    answers: &'a [String],
}

#[derive(Debug, Clone)]
struct PeriodRange {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[tauri::command]
pub fn get_week_plan(
    state: State<DbState>,
    anchor_date: Option<String>,
) -> Result<PlanSnapshotDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    load_plan_snapshot(&conn, "week", anchor_date.as_deref())
}

#[tauri::command]
pub fn get_month_plan(
    state: State<DbState>,
    anchor_date: Option<String>,
) -> Result<PlanSnapshotDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    load_plan_snapshot(&conn, "month", anchor_date.as_deref())
}

#[tauri::command]
pub fn get_calendar_plan_history(
    state: State<DbState>,
    date: String,
) -> Result<CalendarPlanHistoryDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    Ok(CalendarPlanHistoryDto {
        date: date.clone(),
        week_plan: load_calendar_plan_history_item(&conn, "week", &date)?,
        month_plan: load_calendar_plan_history_item(&conn, "month", &date)?,
    })
}

#[tauri::command]
pub fn save_plan_cycle(
    state: State<DbState>,
    period_type: String,
    anchor_date: String,
    title: String,
    summary: String,
) -> Result<PlanSnapshotDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let range = resolve_period_range(&period_type, Some(&anchor_date))?;
    let cycle = plan_repo::ensure_cycle(
        &conn,
        &period_type,
        &fmt_date(range.start_date),
        &fmt_date(range.end_date),
    )
    .map_err(|e| e.to_string())?;
    plan_repo::update_cycle_details(&conn, cycle.id, &title, &summary)
        .map_err(|e| e.to_string())?;
    load_plan_snapshot(&conn, &period_type, Some(&anchor_date))
}

#[tauri::command]
pub fn save_plan_item(
    state: State<DbState>,
    cycle_id: i64,
    item_id: Option<i64>,
    title: String,
    description: String,
    dimension_key: Option<String>,
    progress_percent: Option<i32>,
    ai_comment: Option<String>,
    sort_order: Option<i32>,
    is_completed: bool,
) -> Result<PlanSnapshotDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let cycle = plan_repo::get_cycle_by_id(&conn, cycle_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "plan cycle not found".to_string())?;
    let final_sort_order = match (item_id, sort_order) {
        (_, Some(value)) => value,
        (Some(id), None) => plan_repo::get_item_by_id(&conn, id)
            .map_err(|e| e.to_string())?
            .map(|item| item.sort_order)
            .unwrap_or(0),
        (None, None) => plan_repo::next_sort_order(&conn, cycle_id).map_err(|e| e.to_string())?,
    };
    let existing_item = match item_id {
        Some(id) => Some(
            plan_repo::get_item_by_id(&conn, id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "plan item not found".to_string())?,
        ),
        None => None,
    };
    let resolved_progress_percent =
        resolve_goal_progress(existing_item.as_ref(), progress_percent, is_completed);
    plan_repo::save_item(
        &conn,
        item_id,
        cycle_id,
        &title,
        &description,
        dimension_key.as_deref(),
        resolved_progress_percent,
        ai_comment.as_deref().unwrap_or(""),
        final_sort_order,
        is_completed,
    )
    .map_err(|e| e.to_string())?;
    load_plan_snapshot(&conn, &cycle.period_type, Some(&cycle.start_date))
}

#[tauri::command]
pub fn delete_plan_item(
    state: State<DbState>,
    cycle_id: i64,
    item_id: i64,
) -> Result<PlanSnapshotDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let cycle = plan_repo::get_cycle_by_id(&conn, cycle_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "plan cycle not found".to_string())?;
    plan_repo::delete_item(&conn, item_id).map_err(|e| e.to_string())?;
    load_plan_snapshot(&conn, &cycle.period_type, Some(&cycle.start_date))
}

#[tauri::command]
pub async fn refresh_plan_progress(
    state: State<'_, DbState>,
    period_type: String,
    anchor_date: String,
) -> Result<PlanAiOutcomeDto, String> {
    let snapshot = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        load_plan_snapshot(&conn, &period_type, Some(&anchor_date))?
    };

    let request_payload = serde_json::to_string(&PlanAiRequest {
        mode: &period_type,
        stage: "refresh",
        cycle: &snapshot.cycle,
        items: &snapshot.items,
        growth: &snapshot.growth,
        related_weeks: &snapshot.related_weeks,
        clarification: None,
    })
    .map_err(|e| e.to_string())?;

    let response_payload = api_proxy::execute_plan_api_request(
        &state,
        request_payload.clone(),
        api_proxy::AiTaskKind::PlanRefresh,
    )
    .await?;
    let parsed = parse_plan_ai_response(&response_payload)?;
    let proposal_json = parsed
        .proposal
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| e.to_string())?;
    let questions_json = serde_json::to_string(&parsed.questions).map_err(|e| e.to_string())?;

    let conn = snapshot_lock(&state)?;
    let session = plan_repo::create_ai_session(
        &conn,
        snapshot.cycle.id,
        &request_payload,
        Some(&response_payload),
        &questions_json,
        "[]",
        proposal_json.as_deref(),
        if parsed.requires_clarification {
            "clarifying"
        } else {
            "ready"
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(PlanAiOutcomeDto {
        session_id: session.id,
        status: session.status,
        requires_clarification: parsed.requires_clarification,
        questions: parsed.questions,
        proposal: parsed.proposal,
    })
}

#[tauri::command]
pub async fn submit_plan_ai_answers(
    state: State<'_, DbState>,
    session_id: i64,
    answers: Vec<String>,
) -> Result<PlanAiOutcomeDto, String> {
    let (session, snapshot, questions) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let session = plan_repo::get_ai_session_by_id(&conn, session_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "plan ai session not found".to_string())?;
        let cycle = plan_repo::get_cycle_by_id(&conn, session.cycle_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "plan cycle not found".to_string())?;
        let snapshot = load_plan_snapshot(&conn, &cycle.period_type, Some(&cycle.start_date))?;
        let questions: Vec<String> =
            serde_json::from_str(&session.questions_json).map_err(|e| e.to_string())?;
        (session, snapshot, questions)
    };

    let request_payload = serde_json::to_string(&PlanAiRequest {
        mode: &snapshot.cycle.period_type,
        stage: "clarification",
        cycle: &snapshot.cycle,
        items: &snapshot.items,
        growth: &snapshot.growth,
        related_weeks: &snapshot.related_weeks,
        clarification: Some(PlanClarificationPayload {
            questions: &questions,
            answers: &answers,
        }),
    })
    .map_err(|e| e.to_string())?;

    {
        let conn = snapshot_lock(&state)?;
        let answers_json = serde_json::to_string(&answers).map_err(|e| e.to_string())?;
        plan_repo::update_ai_session_answers(&conn, session.id, &answers_json, &request_payload)
            .map_err(|e| e.to_string())?;
    }

    let response_payload = api_proxy::execute_plan_api_request(
        &state,
        request_payload,
        api_proxy::AiTaskKind::PlanClarification,
    )
    .await?;
    let parsed = parse_plan_ai_response(&response_payload)?;
    let proposal_json = parsed
        .proposal
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| e.to_string())?;
    let questions_json = serde_json::to_string(&parsed.questions).map_err(|e| e.to_string())?;

    let conn = snapshot_lock(&state)?;
    plan_repo::update_ai_session_resolution(
        &conn,
        session.id,
        &response_payload,
        &questions_json,
        proposal_json.as_deref(),
        if parsed.requires_clarification {
            "clarifying"
        } else {
            "ready"
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(PlanAiOutcomeDto {
        session_id: session.id,
        status: if parsed.requires_clarification {
            "clarifying".into()
        } else {
            "ready".into()
        },
        requires_clarification: parsed.requires_clarification,
        questions: parsed.questions,
        proposal: parsed.proposal,
    })
}

#[tauri::command]
pub fn apply_plan_ai_update(
    state: State<DbState>,
    session_id: i64,
) -> Result<PlanSnapshotDto, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let session = plan_repo::get_ai_session_by_id(&conn, session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "plan ai session not found".to_string())?;
    let proposal_json = session
        .proposal_json
        .ok_or_else(|| "plan ai session has no proposal to apply".to_string())?;
    let proposal: PlanAiProposalDto =
        serde_json::from_str(&proposal_json).map_err(|e| e.to_string())?;
    let cycle = plan_repo::get_cycle_by_id(&conn, session.cycle_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "plan cycle not found".to_string())?;

    plan_repo::update_cycle_ai_summary(
        &conn,
        cycle.id,
        &proposal.ai_summary,
        &proposal.title,
        &proposal.summary,
    )
    .map_err(|e| e.to_string())?;

    let items = proposal
        .items
        .into_iter()
        .map(|item| plan_repo::PlanItemInput {
            title: item.title,
            description: item.description,
            dimension_key: item.dimension_key,
            progress_percent: item.progress_percent,
            ai_comment: item.ai_comment,
            sort_order: item.sort_order,
            is_completed: item.is_completed,
        })
        .collect::<Vec<_>>();

    plan_repo::replace_items_for_cycle(&conn, cycle.id, &items).map_err(|e| e.to_string())?;
    load_plan_snapshot(&conn, &cycle.period_type, Some(&cycle.start_date))
}

fn snapshot_lock<'a>(
    state: &'a State<'a, DbState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.0.lock().map_err(|e| e.to_string())
}

fn load_plan_snapshot(
    conn: &rusqlite::Connection,
    period_type: &str,
    anchor_date: Option<&str>,
) -> Result<PlanSnapshotDto, String> {
    let range = resolve_period_range(period_type, anchor_date)?;
    let cycle = plan_repo::ensure_cycle(
        conn,
        period_type,
        &fmt_date(range.start_date),
        &fmt_date(range.end_date),
    )
    .map_err(|e| e.to_string())?;
    let items = plan_repo::list_items_by_cycle(conn, cycle.id).map_err(|e| e.to_string())?;
    let goal_progress = build_goal_progress_summary(&items);
    let growth = build_growth_snapshot(conn, &range)?;
    let related_weeks = if period_type == "month" {
        build_related_week_plans(conn, &range)?
    } else {
        Vec::new()
    };

    Ok(PlanSnapshotDto {
        cycle: map_cycle(cycle),
        items: items.into_iter().map(map_item).collect(),
        goal_progress,
        growth,
        related_weeks,
    })
}

fn load_calendar_plan_history_item(
    conn: &rusqlite::Connection,
    period_type: &str,
    date: &str,
) -> Result<Option<CalendarPlanHistoryItemDto>, String> {
    let Some(cycle) =
        plan_repo::get_cycle_covering_date(conn, period_type, date).map_err(|e| e.to_string())?
    else {
        return Ok(None);
    };

    let items = plan_repo::list_items_by_cycle(conn, cycle.id).map_err(|e| e.to_string())?;
    let today = Local::now().date_naive();
    let cycle_end =
        NaiveDate::parse_from_str(&cycle.end_date, "%Y-%m-%d").map_err(|e| e.to_string())?;

    Ok(Some(CalendarPlanHistoryItemDto {
        cycle: map_cycle(cycle),
        goal_progress: build_goal_progress_summary(&items),
        items: items.into_iter().map(map_item).collect(),
        is_historical: cycle_end < today,
    }))
}

fn build_growth_snapshot(
    conn: &rusqlite::Connection,
    range: &PeriodRange,
) -> Result<PlanGrowthSnapshotDto, String> {
    let start_str = fmt_date(range.start_date);
    let end_str = fmt_date(range.end_date);
    let total_days = (range.end_date - range.start_date).num_days() as i32 + 1;
    let dims = dimension_repo::get_all_dimensions(conn).map_err(|e| e.to_string())?;
    let totals_map = ledger_repo::get_totals_in_range(conn, &start_str, &end_str)
        .map_err(|e| e.to_string())?
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
    let record_counts = record_repo::get_record_counts_in_range(conn, &start_str, &end_str)
        .map_err(|e| e.to_string())?;
    let review_flags =
        crate::db::repositories::daily_review_repo::get_flags_in_range(conn, &start_str, &end_str)
            .map_err(|e| e.to_string())?;
    let analyzed_days = review_flags.iter().filter(|flag| flag.is_analyzed).count() as i32;
    let record_count = record_counts.iter().map(|item| item.count).sum::<i32>();
    let active_days = record_counts.len() as i32;
    let dimensions = dims
        .into_iter()
        .map(|dim| {
            let total = totals_map.get(&dim.key).copied().unwrap_or(0);
            let max_total = dim.daily_cap * total_days;
            let progress_percent = if max_total <= 0 {
                0
            } else {
                ((total as f64 / max_total as f64) * 100.0).round() as i32
            }
            .clamp(0, 100);
            PlanGrowthDimensionDto {
                headline: build_growth_headline(total, max_total),
                key: dim.key,
                name: dim.name,
                total,
                daily_cap: dim.daily_cap,
                max_total,
                progress_percent,
            }
        })
        .collect::<Vec<_>>();
    let total_change = dimensions.iter().map(|item| item.total).sum();

    Ok(PlanGrowthSnapshotDto {
        start_date: start_str,
        end_date: end_str,
        total_days,
        active_days,
        analyzed_days,
        record_count,
        total_change,
        dimensions,
    })
}

fn build_related_week_plans(
    conn: &rusqlite::Connection,
    range: &PeriodRange,
) -> Result<Vec<RelatedWeekPlanDto>, String> {
    let cycles = plan_repo::list_week_cycles_in_range(
        conn,
        &fmt_date(range.start_date),
        &fmt_date(range.end_date),
    )
    .map_err(|e| e.to_string())?;
    let mut result = Vec::with_capacity(cycles.len());
    for cycle in cycles {
        let items = plan_repo::list_items_by_cycle(conn, cycle.id).map_err(|e| e.to_string())?;
        let goal_progress = build_goal_progress_summary(&items);
        let title = fallback_cycle_title(&cycle);
        result.push(RelatedWeekPlanDto {
            cycle_id: cycle.id,
            start_date: cycle.start_date,
            end_date: cycle.end_date,
            title,
            summary: cycle.summary,
            average_progress: goal_progress.average_progress_percent,
            total_items: goal_progress.total_items,
            completed_items: goal_progress.completed_items,
        });
    }
    Ok(result)
}

fn build_goal_progress_summary(items: &[crate::models::plan::PlanItem]) -> GoalProgressSummaryDto {
    let total_items = items.len() as i32;
    let completed_items = items.iter().filter(|item| item.is_completed).count() as i32;
    let active_goal_count = total_items - completed_items;
    let average_progress_percent = if total_items == 0 {
        0
    } else {
        let total_progress = items
            .iter()
            .map(|item| {
                if item.is_completed {
                    100
                } else {
                    item.progress_percent.clamp(0, 100)
                }
            })
            .sum::<i32>();
        (total_progress as f64 / total_items as f64).round() as i32
    };

    GoalProgressSummaryDto {
        total_items,
        completed_items,
        active_goal_count: active_goal_count.max(0),
        average_progress_percent,
    }
}

fn resolve_goal_progress(
    existing_item: Option<&crate::models::plan::PlanItem>,
    requested_progress: Option<i32>,
    is_completed: bool,
) -> i32 {
    if is_completed {
        return 100;
    }

    requested_progress
        .or_else(|| existing_item.map(|item| item.progress_percent))
        .unwrap_or(0)
        .clamp(0, 100)
}

fn map_cycle(cycle: PlanCycle) -> PlanCycleDto {
    let title = fallback_cycle_title(&cycle);
    PlanCycleDto {
        id: cycle.id,
        period_type: cycle.period_type,
        start_date: cycle.start_date,
        end_date: cycle.end_date,
        title,
        summary: cycle.summary,
        ai_summary: cycle.ai_summary,
        last_ai_run_at: cycle.last_ai_run_at,
    }
}

fn map_item(item: crate::models::plan::PlanItem) -> PlanItemDto {
    PlanItemDto {
        id: item.id,
        cycle_id: item.cycle_id,
        title: item.title,
        description: item.description,
        dimension_key: item.dimension_key,
        progress_percent: item.progress_percent,
        ai_comment: item.ai_comment,
        sort_order: item.sort_order,
        is_completed: item.is_completed,
    }
}

fn fallback_cycle_title(cycle: &PlanCycle) -> String {
    if !cycle.title.trim().is_empty() {
        return cycle.title.clone();
    }
    if cycle.period_type == "week" {
        format!("Week {}", cycle.start_date)
    } else {
        let month = cycle.start_date.get(0..7).unwrap_or(&cycle.start_date);
        format!("Month {}", month)
    }
}

fn build_growth_headline(total: i32, max_total: i32) -> String {
    if total <= 0 {
        return "本期沉默".into();
    }
    if max_total <= 0 {
        return "轻微增长".into();
    }
    let ratio = total as f64 / max_total as f64;
    if ratio >= 0.65 {
        "高强推进".into()
    } else if ratio >= 0.3 {
        "稳定推进".into()
    } else {
        "轻微增长".into()
    }
}

fn parse_plan_ai_response(payload: &str) -> Result<PlanAiApiResponse, String> {
    let parsed: PlanAiApiResponse = ai_response_service::parse_ai_json(payload, "plan api")?;
    if parsed.requires_clarification {
        if parsed.questions.is_empty() {
            return Err("plan api returned clarification without questions".into());
        }
    } else if parsed.proposal.is_none() {
        return Err("plan api returned no proposal".into());
    }
    Ok(parsed)
}

fn resolve_period_range(
    period_type: &str,
    anchor_date: Option<&str>,
) -> Result<PeriodRange, String> {
    let anchor = match anchor_date {
        Some(value) => NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|e| e.to_string())?,
        None => Local::now().date_naive(),
    };

    match period_type {
        "week" => {
            let offset = anchor.weekday().num_days_from_monday() as i64;
            let start_date = anchor - Duration::days(offset);
            let end_date = start_date + Duration::days(6);
            Ok(PeriodRange {
                start_date,
                end_date,
            })
        }
        "month" => {
            let start_date = NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), 1)
                .ok_or_else(|| "invalid month start".to_string())?;
            let next_month = if anchor.month() == 12 {
                NaiveDate::from_ymd_opt(anchor.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(anchor.year(), anchor.month() + 1, 1)
            }
            .ok_or_else(|| "invalid next month".to_string())?;
            let end_date = next_month
                .pred_opt()
                .ok_or_else(|| "invalid month end".to_string())?;
            Ok(PeriodRange {
                start_date,
                end_date,
            })
        }
        _ => Err(format!("unsupported period type: {period_type}")),
    }
}

fn fmt_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    #[test]
    fn resolve_week_period_uses_monday_start() {
        let range = resolve_period_range("week", Some("2026-06-13")).expect("range");
        assert_eq!(range.start_date.weekday(), Weekday::Mon);
        assert_eq!(fmt_date(range.start_date), "2026-06-08");
        assert_eq!(fmt_date(range.end_date), "2026-06-14");
    }

    #[test]
    fn resolve_month_period_handles_leap_year_february() {
        let range = resolve_period_range("month", Some("2028-02-10")).expect("range");
        assert_eq!(fmt_date(range.start_date), "2028-02-01");
        assert_eq!(fmt_date(range.end_date), "2028-02-29");
    }

    #[test]
    fn parse_plan_ai_response_requires_questions_when_clarifying() {
        let result = parse_plan_ai_response(
            r#"{"requires_clarification":true,"questions":[],"proposal":null}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn build_goal_progress_summary_counts_completed_as_full_progress() {
        let summary = build_goal_progress_summary(&[
            crate::models::plan::PlanItem {
                id: 1,
                cycle_id: 1,
                title: "Goal A".into(),
                description: "".into(),
                dimension_key: None,
                progress_percent: 40,
                ai_comment: "".into(),
                sort_order: 0,
                is_completed: false,
            },
            crate::models::plan::PlanItem {
                id: 2,
                cycle_id: 1,
                title: "Goal B".into(),
                description: "".into(),
                dimension_key: None,
                progress_percent: 65,
                ai_comment: "".into(),
                sort_order: 1,
                is_completed: true,
            },
        ]);

        assert_eq!(summary.total_items, 2);
        assert_eq!(summary.completed_items, 1);
        assert_eq!(summary.active_goal_count, 1);
        assert_eq!(summary.average_progress_percent, 70);
    }

    #[test]
    fn resolve_goal_progress_keeps_existing_ai_value_when_unchecking_complete() {
        let existing = crate::models::plan::PlanItem {
            id: 1,
            cycle_id: 1,
            title: "Goal A".into(),
            description: "".into(),
            dimension_key: None,
            progress_percent: 55,
            ai_comment: "".into(),
            sort_order: 0,
            is_completed: true,
        };

        assert_eq!(resolve_goal_progress(Some(&existing), None, false), 55);
        assert_eq!(resolve_goal_progress(Some(&existing), None, true), 100);
    }
}
