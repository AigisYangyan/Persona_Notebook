use crate::db::connection::DbState;
use crate::db::repositories::{api_run_repo, setting_repo};
use crate::services::ai_response_service;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiModelTier {
    Flash,
    Pro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTaskKind {
    Scoring,
    PlanRefresh,
    PlanClarification,
    Tarot,
    PeriodReportDay,
    PeriodReportWeek,
    PeriodReportMonth,
}

#[derive(Debug, Clone)]
struct DeepSeekConfig {
    base_url: String,
    flash_model: String,
    pro_model: String,
    api_key: String,
}

#[derive(Debug, Clone)]
struct ResolvedAiRoute {
    task_kind: AiTaskKind,
    resolved_tier: AiModelTier,
    fallback_used: bool,
    base_url: String,
    model: String,
    api_key: String,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: i32,
}

const SCORING_SYSTEM_PROMPT: &str = r#"You are the scoring engine for Personal Growth RPG Notebook.
Return valid JSON only. Do not include markdown, code fences, commentary, or extra text.

Rules:
1. Use the request's rule_hints as the first-class constraint.
2. Each task may affect at most 3 dimensions.
3. Respect the provided daily caps and keep outputs conservative for vague tasks.
4. Lower confidence when the task title is ambiguous or weakly supported by the request.

Required JSON shape:
{
  "version": "1.0",
  "date": "YYYY-MM-DD",
  "total_changes": {
    "knowledge": 0,
    "willpower": 0,
    "expression": 0,
    "physique": 0,
    "bond": 0
  },
  "record_results": [
    {
      "title": "original task title",
      "category": "short category",
      "changes": { "knowledge": 0 },
      "difficulty_star": 0,
      "confidence": 0.0,
      "reason": "15-50 word reason"
    }
  ],
  "summary": "0-80 word daily summary"
}"#;

const PLAN_SYSTEM_PROMPT: &str = r#"You are the planning adjustment engine for Personal Growth RPG Notebook.
Return valid JSON only. Do not include markdown, code fences, commentary, or extra text.

Your job:
1. Read the cycle goals, recent records, growth signals, and optional user answers.
2. If the goals are too vague, missing the object, action, cadence, or success criteria, ask clarification questions first.
3. If the information is sufficient, return an apply-ready proposal.
4. Keep progress_percent between 0 and 100.
5. Keep ai_summary short and concrete.

Clarification shape:
{
  "requires_clarification": true,
  "questions": ["question 1", "question 2"],
  "proposal": null
}

Proposal shape:
{
  "requires_clarification": false,
  "questions": [],
  "proposal": {
    "title": "plan title",
    "summary": "short user-facing summary",
    "ai_summary": "one-sentence status",
    "items": [
      {
        "title": "goal title",
        "description": "specific goal description",
        "dimension_key": "knowledge",
        "progress_percent": 35,
        "ai_comment": "why this progress was assigned",
        "sort_order": 0,
        "is_completed": false
      }
    ]
  }
}"#;

const DAILY_INSIGHT_SYSTEM_PROMPT: &str = r#"You are the personal insight engine for Personal Growth RPG Notebook.
You may only use the provided JSON context. Do not invent facts, events, emotions, relationships, or progress not supported by evidence.
Return valid JSON only. Do not include markdown, code fences, commentary, or extra text.

Global rules:
1. Cite evidence_ids whenever possible. If evidence is insufficient, say so explicitly.
2. Output must contain both report and memory_delta.
3. memory_delta must follow PersonalMemoryPatch v1.
4. memory_delta must never overwrite birthday.
5. Warmth and imagination are welcome, but only for expression. Do not fabricate facts.

Tarot mode:
- Treat tarot as a psychological archetype card, not fortune telling.
- The writing may be longer and more atmospheric, but it must stay grounded in the context.
- The report must include:
  card_name, archetype, psychological_theme, body_signal, warm_quote,
  encouragement, action, risk_reminder, deeper_reading, evidence_ids
- encouragement should be 2-4 sentences.
- deeper_reading should be one longer paragraph.
- action should contain 2-4 executable suggestions.

Report mode:
- The report may be daily, weekly, or monthly.
- It must go beyond recap and include diagnosis, leverage points, and concrete remedies.
- The report must include:
  completed, unfinished, time_focus, growth_changes, plan_progress,
  journal_and_bond_observations, root_causes, leverage_points,
  concrete_remedies, not_enough_data, next_actions, evidence_ids

Required top-level shape:
{
  "schema_version": "1.0",
  "title": "short title",
  "summary": "short summary",
  "report": {},
  "memory_delta": {
    "schema_version": "1.0",
    "profile_updates": null,
    "memory_operations": []
  }
}"#;

impl AiModelTier {
    fn as_str(self) -> &'static str {
        match self {
            Self::Flash => "flash",
            Self::Pro => "pro",
        }
    }
}

impl AiTaskKind {
    fn model_tier(self) -> AiModelTier {
        match self {
            Self::Scoring | Self::PlanRefresh | Self::PlanClarification => AiModelTier::Flash,
            Self::Tarot
            | Self::PeriodReportDay
            | Self::PeriodReportWeek
            | Self::PeriodReportMonth => AiModelTier::Pro,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Scoring => "scoring",
            Self::PlanRefresh => "plan_refresh",
            Self::PlanClarification => "plan_clarification",
            Self::Tarot => "tarot",
            Self::PeriodReportDay => "period_report_day",
            Self::PeriodReportWeek => "period_report_week",
            Self::PeriodReportMonth => "period_report_month",
        }
    }
}

#[tauri::command]
pub async fn call_scoring_api(
    state: State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    let content = execute_api_request(
        &state,
        request_json,
        SCORING_SYSTEM_PROMPT,
        2048,
        AiTaskKind::Scoring,
    )
    .await?;
    ai_response_service::normalize_ai_json_string(&content, "scoring api")
}

#[tauri::command]
pub async fn call_plan_api(
    state: State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    let content = execute_api_request(
        &state,
        request_json,
        PLAN_SYSTEM_PROMPT,
        3072,
        AiTaskKind::PlanRefresh,
    )
    .await?;
    ai_response_service::normalize_ai_json_string(&content, "plan api")
}

pub async fn execute_plan_api_request(
    state: &State<'_, DbState>,
    request_json: String,
    task_kind: AiTaskKind,
) -> Result<String, String> {
    execute_api_request(state, request_json, PLAN_SYSTEM_PROMPT, 3072, task_kind).await
}

pub async fn execute_daily_insight_api_request(
    state: &State<'_, DbState>,
    request_json: String,
    task_kind: AiTaskKind,
) -> Result<String, String> {
    execute_api_request(
        state,
        request_json,
        DAILY_INSIGHT_SYSTEM_PROMPT,
        4096,
        task_kind,
    )
    .await
}

async fn execute_api_request(
    state: &State<'_, DbState>,
    request_json: String,
    system_prompt: &str,
    max_tokens: i32,
    task_kind: AiTaskKind,
) -> Result<String, String> {
    let started_at = Instant::now();
    let request_date = extract_request_date(&request_json);

    let route = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        resolve_ai_route(&conn, task_kind)?
    };

    if route.api_key.trim().is_empty() {
        let error_message = "API Key is not configured".to_string();
        persist_api_run(
            state,
            &request_date,
            &request_json,
            None,
            "error",
            Some(&error_message),
            started_at.elapsed().as_millis() as i64,
            &route,
        );
        return Err(error_message);
    }

    let url = format!("{}/chat/completions", route.base_url.trim_end_matches('/'));
    let body = OpenAIRequest {
        model: route.model.clone(),
        messages: vec![
            OpenAIMessage {
                role: "system".into(),
                content: system_prompt.into(),
            },
            OpenAIMessage {
                role: "user".into(),
                content: request_json.clone(),
            },
        ],
        temperature: 0.2,
        max_tokens,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", route.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let message = format!("HTTP request failed: {e}");
            persist_api_run(
                state,
                &request_date,
                &request_json,
                None,
                "error",
                Some(&message),
                started_at.elapsed().as_millis() as i64,
                &route,
            );
            message
        })?;

    let status = response.status();
    let text = response.text().await.map_err(|e| {
        let message = format!("Failed to read API response: {e}");
        persist_api_run(
            state,
            &request_date,
            &request_json,
            None,
            "error",
            Some(&message),
            started_at.elapsed().as_millis() as i64,
            &route,
        );
        message
    })?;

    if !status.is_success() {
        let message = format!("API error ({status}): {text}");
        persist_api_run(
            state,
            &request_date,
            &request_json,
            Some(&text),
            "error",
            Some(&message),
            started_at.elapsed().as_millis() as i64,
            &route,
        );
        return Err(message);
    }

    let content = ai_response_service::extract_chat_content(&text).map_err(|error| {
        persist_api_run(
            state,
            &request_date,
            &request_json,
            Some(&text),
            "error",
            Some(&error),
            started_at.elapsed().as_millis() as i64,
            &route,
        );
        error
    })?;

    persist_api_run(
        state,
        &request_date,
        &request_json,
        Some(&content),
        "success",
        None,
        started_at.elapsed().as_millis() as i64,
        &route,
    );

    Ok(content)
}

fn resolve_ai_route(
    conn: &rusqlite::Connection,
    task_kind: AiTaskKind,
) -> Result<ResolvedAiRoute, String> {
    let config = load_deepseek_config(conn)?;
    let requested_tier = task_kind.model_tier();
    let requested_model = match requested_tier {
        AiModelTier::Flash => config.flash_model.trim(),
        AiModelTier::Pro => config.pro_model.trim(),
    };

    let (resolved_tier, model) = if !requested_model.is_empty() {
        (requested_tier, requested_model.to_string())
    } else {
        let fallback_tier = match requested_tier {
            AiModelTier::Flash => AiModelTier::Pro,
            AiModelTier::Pro => AiModelTier::Flash,
        };
        let fallback_model = match fallback_tier {
            AiModelTier::Flash => config.flash_model.trim(),
            AiModelTier::Pro => config.pro_model.trim(),
        };
        if fallback_model.is_empty() {
            return Err(
                "DeepSeek model is not configured. Please set flash or pro model in Settings."
                    .to_string(),
            );
        }
        (fallback_tier, fallback_model.to_string())
    };

    Ok(ResolvedAiRoute {
        task_kind,
        resolved_tier,
        fallback_used: requested_tier != resolved_tier,
        base_url: config.base_url,
        model,
        api_key: config.api_key,
    })
}

fn load_deepseek_config(conn: &rusqlite::Connection) -> Result<DeepSeekConfig, String> {
    let deepseek_base_url = setting_repo::get_setting(conn, "deepseek_base_url")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let legacy_base_url = setting_repo::get_setting(conn, "api_base_url")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let base_url = if !deepseek_base_url.trim().is_empty() {
        deepseek_base_url
    } else if is_deepseek_like(&legacy_base_url) {
        legacy_base_url
    } else {
        "https://api.deepseek.com/v1".to_string()
    };

    let deepseek_flash_model = setting_repo::get_setting(conn, "deepseek_flash_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let deepseek_pro_model = setting_repo::get_setting(conn, "deepseek_pro_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let legacy_model = setting_repo::get_setting(conn, "api_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    Ok(DeepSeekConfig {
        base_url,
        flash_model: if !deepseek_flash_model.trim().is_empty() {
            deepseek_flash_model
        } else if is_deepseek_like(&legacy_model) {
            legacy_model.clone()
        } else {
            String::new()
        },
        pro_model: if !deepseek_pro_model.trim().is_empty() {
            deepseek_pro_model
        } else if is_deepseek_like(&legacy_model) {
            legacy_model
        } else {
            String::new()
        },
        api_key: setting_repo::get_setting(conn, "api_key")
            .map_err(|e| e.to_string())?
            .unwrap_or_default(),
    })
}

fn is_deepseek_like(value: &str) -> bool {
    value.to_ascii_lowercase().contains("deepseek")
}

fn extract_request_date(request_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(request_json)
        .ok()
        .and_then(|value| {
            value
                .get("date")
                .and_then(|date| date.as_str())
                .map(str::to_owned)
                .or_else(|| {
                    value
                        .get("cycle")
                        .and_then(|cycle| cycle.get("start_date"))
                        .and_then(|date| date.as_str())
                        .map(str::to_owned)
                })
        })
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string())
}

fn persist_api_run(
    state: &State<'_, DbState>,
    date: &str,
    request_json: &str,
    response_json: Option<&str>,
    status: &str,
    error_message: Option<&str>,
    latency_ms: i64,
    route: &ResolvedAiRoute,
) {
    if let Ok(conn) = state.0.lock() {
        let _ = api_run_repo::create_run(
            &conn,
            date,
            request_json,
            response_json,
            status,
            error_message,
            latency_ms,
            &route.model,
            route.task_kind.as_str(),
            route.resolved_tier.as_str(),
            route.fallback_used,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::setting_repo;
    use rusqlite::Connection;

    #[test]
    fn task_kind_uses_expected_model_tier() {
        assert_eq!(AiTaskKind::Scoring.model_tier(), AiModelTier::Flash);
        assert_eq!(AiTaskKind::PlanRefresh.model_tier(), AiModelTier::Flash);
        assert_eq!(
            AiTaskKind::PlanClarification.model_tier(),
            AiModelTier::Flash
        );
        assert_eq!(AiTaskKind::Tarot.model_tier(), AiModelTier::Pro);
        assert_eq!(AiTaskKind::PeriodReportDay.model_tier(), AiModelTier::Pro);
        assert_eq!(AiTaskKind::PeriodReportWeek.model_tier(), AiModelTier::Pro);
        assert_eq!(AiTaskKind::PeriodReportMonth.model_tier(), AiModelTier::Pro);
    }

    #[test]
    fn route_falls_back_to_pro_when_flash_missing() {
        let conn = Connection::open_in_memory().expect("db");
        run_migrations(&conn).expect("migrate");
        setting_repo::set_setting(&conn, "deepseek_pro_model", "deepseek-reasoner")
            .expect("pro model");

        let route = resolve_ai_route(&conn, AiTaskKind::Scoring).expect("route");

        assert_eq!(route.resolved_tier, AiModelTier::Pro);
        assert!(route.fallback_used);
        assert_eq!(route.model, "deepseek-reasoner");
    }

    #[test]
    fn route_falls_back_to_flash_when_pro_missing() {
        let conn = Connection::open_in_memory().expect("db");
        run_migrations(&conn).expect("migrate");
        setting_repo::set_setting(&conn, "deepseek_flash_model", "deepseek-chat")
            .expect("flash model");

        let route = resolve_ai_route(&conn, AiTaskKind::Tarot).expect("route");

        assert_eq!(route.resolved_tier, AiModelTier::Flash);
        assert!(route.fallback_used);
        assert_eq!(route.model, "deepseek-chat");
    }

    #[test]
    fn route_rejects_when_both_models_missing() {
        let conn = Connection::open_in_memory().expect("db");
        run_migrations(&conn).expect("migrate");

        let error = resolve_ai_route(&conn, AiTaskKind::Tarot).expect_err("missing model");

        assert!(error.contains("DeepSeek model is not configured"));
    }
}
