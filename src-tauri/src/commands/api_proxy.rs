use crate::db::connection::DbState;
use crate::db::repositories::{api_run_repo, setting_repo};
use crate::services::ai_response_service::{self, ChatResponseMeta};
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
    response_format: OpenAIResponseFormat,
    user: String,
}

#[derive(Serialize)]
struct OpenAIResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Clone)]
struct ApiRequestOptions {
    max_tokens: i32,
    temperature: f32,
}

#[derive(Debug, Clone)]
struct ApiRequestResult {
    content: String,
    _meta: ChatResponseMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiRunDiagnosticDto {
    pub id: i64,
    pub date: String,
    pub status: String,
    pub error_message: Option<String>,
    pub latency_ms: Option<i64>,
    pub engine_name: String,
    pub task_kind: String,
    pub model_tier: String,
    pub fallback_used: bool,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub prompt_cache_hit_tokens: Option<i64>,
    pub prompt_cache_miss_tokens: Option<i64>,
    pub finish_reason: Option<String>,
    pub created_at: String,
}

const SCORING_SYSTEM_PROMPT: &str = r#"You are the scoring engine for Personal Growth RPG Notebook.
Return valid JSON only. Do not include markdown, code fences, commentary, or extra text.

Rules:
1. Use the request's rule_hints as the first-class constraint.
2. Each task may affect at most 3 dimensions.
3. Respect the provided daily caps and keep outputs conservative for vague tasks.
4. Lower confidence when the task title is ambiguous or weakly supported by the request.
5. Keep category short and keep every reason compact.

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
      "reason": "short reason, under 24 Chinese chars or 16 English words"
    }
  ],
  "summary": "short daily summary, under 60 Chinese chars or 40 English words"
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
6. You may provide psychological interpretation based on profile/memories; treat it as interpretation, not as a factual claim.

Tarot mode:
- Treat tarot as a psychological archetype card, not fortune telling.
- The writing may be longer and more atmospheric, but it must stay grounded in the context.
- The report must include:
  card_name, archetype, psychological_theme, body_signal, warm_quote,
  encouragement, action, risk_reminder, deeper_reading, evidence_ids
- encouragement should be 3-6 sentences (warm, concrete, not generic).
- deeper_reading should be 2-3 paragraphs (each paragraph should add new insight).
- action should contain 3-5 executable suggestions, each with a short reason.
- Whenever possible, connect the interpretation to personal_context.profile and memories (do not invent facts).

Report mode:
- The report may be daily, weekly, or monthly.
- Daily report should feel like a gentle psychological assistant, not a checklist recap.
- Start from emotion, pressure, and self-understanding, then end with a few light next steps.
- Weekly and monthly reports may keep more diagnosis, but still avoid dry bullet-style recaps.
- The report must include:
  emotional_reflection, comfort_message, pressure_sources, inner_pattern,
  self_compassion, gentle_questions, small_next_steps,
  optional completed, unfinished, plan_progress, not_enough_data, evidence_ids
- Style & length targets:
  - emotional_reflection: at least 3 sentences, grounded in evidence and profile.
  - inner_pattern: 1-2 paragraphs, name the pattern and the trigger.
  - comfort_message: 2-4 sentences, specific to the user's context.
  - pressure_sources: one paragraph, not a list of labels.
  - gentle_questions: 3-6 questions.
  - small_next_steps: 3-6 steps; each step includes an explanation why it helps.

Required top-level shape:
{
  "schema_version": "1.0",
  "title": "clear title (not necessarily short)",
  "summary": "2-4 sentences summary",
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
pub fn get_recent_api_runs(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<ApiRunDiagnosticDto>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let records = api_run_repo::list_recent_runs(&conn, limit.unwrap_or(12)).map_err(|e| e.to_string())?;
    Ok(records.into_iter().map(|record| ApiRunDiagnosticDto {
        id: record.id,
        date: record.date,
        status: record.status,
        error_message: record.error_message,
        latency_ms: record.latency_ms,
        engine_name: record.engine_name,
        task_kind: record.task_kind,
        model_tier: record.model_tier,
        fallback_used: record.fallback_used,
        prompt_tokens: record.prompt_tokens,
        completion_tokens: record.completion_tokens,
        prompt_cache_hit_tokens: record.prompt_cache_hit_tokens,
        prompt_cache_miss_tokens: record.prompt_cache_miss_tokens,
        finish_reason: record.finish_reason,
        created_at: record.created_at,
    }).collect())
}

#[tauri::command]
pub async fn call_scoring_api(
    state: State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    let attempts = [1536, 2560];
    let mut last_error = None;

    for (index, max_tokens) in attempts.into_iter().enumerate() {
        let content = match execute_api_request(
            &state,
            request_json.clone(),
            SCORING_SYSTEM_PROMPT,
            AiTaskKind::Scoring,
            ApiRequestOptions {
                max_tokens,
                temperature: 0.2,
            },
        )
        .await
        {
            Ok(result) => result.content,
            Err(error) => {
                last_error = Some(error.clone());
                if index + 1 < attempts.len() && should_retry_scoring_error(&error) {
                    continue;
                }
                return Err(error);
            }
        };

        match ai_response_service::normalize_ai_json_string(&content, "scoring api") {
            Ok(normalized) => return Ok(normalized),
            Err(error) => {
                last_error = Some(error.clone());
                if index + 1 < attempts.len() && should_retry_scoring_error(&error) {
                    continue;
                }
                return Err(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "scoring api failed without a detailed error".to_string()))
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
        AiTaskKind::PlanRefresh,
        ApiRequestOptions {
            max_tokens: 3072,
            temperature: 0.2,
        },
    )
    .await?;
    ai_response_service::normalize_ai_json_string(&content.content, "plan api")
}

pub async fn execute_plan_api_request(
    state: &State<'_, DbState>,
    request_json: String,
    task_kind: AiTaskKind,
) -> Result<String, String> {
    execute_api_request(
        state,
        request_json,
        PLAN_SYSTEM_PROMPT,
        task_kind,
        ApiRequestOptions {
            max_tokens: 3072,
            temperature: 0.2,
        },
    )
    .await
    .map(|result| result.content)
}

pub async fn execute_daily_insight_api_request(
    state: &State<'_, DbState>,
    request_json: String,
    task_kind: AiTaskKind,
) -> Result<String, String> {
    let attempts = [8192, 12288];
    let mut last_error = None;
    for (index, max_tokens) in attempts.into_iter().enumerate() {
        let result = execute_api_request(
            state,
            request_json.clone(),
            DAILY_INSIGHT_SYSTEM_PROMPT,
            task_kind,
            ApiRequestOptions {
                max_tokens,
                temperature: 0.6,
            },
        )
        .await;
        match result {
            Ok(payload) => return Ok(payload.content),
            Err(error) => {
                last_error = Some(error.clone());
                if index + 1 < attempts.len() && should_retry_insight_error(&error) {
                    continue;
                }
                return Err(error);
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "daily insight api failed without a detailed error".to_string()))
}

async fn execute_api_request(
    state: &State<'_, DbState>,
    request_json: String,
    system_prompt: &str,
    task_kind: AiTaskKind,
    options: ApiRequestOptions,
) -> Result<ApiRequestResult, String> {
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
            None,
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
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        response_format: OpenAIResponseFormat {
            format_type: "json_object".into(),
        },
        user: "pgrn-local-user".into(),
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
                None,
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
            None,
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
            None,
        );
        return Err(message);
    }

    let response_payload = ai_response_service::extract_chat_response(&text).map_err(|error| {
        persist_api_run(
            state,
            &request_date,
            &request_json,
            Some(&text),
            "error",
            Some(&error),
            started_at.elapsed().as_millis() as i64,
            &route,
            None,
        );
        error
    })?;

    persist_api_run(
        state,
        &request_date,
        &request_json,
        Some(&response_payload.content),
        "success",
        None,
        started_at.elapsed().as_millis() as i64,
        &route,
        Some(&response_payload.meta),
    );

    Ok(ApiRequestResult {
        content: response_payload.content,
        _meta: response_payload.meta,
    })
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
    meta: Option<&ChatResponseMeta>,
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
            meta.and_then(|value| value.usage.prompt_tokens),
            meta.and_then(|value| value.usage.completion_tokens),
            meta.and_then(|value| value.usage.prompt_cache_hit_tokens),
            meta.and_then(|value| value.usage.prompt_cache_miss_tokens),
            meta.and_then(|value| value.finish_reason.as_deref()),
        );
    }
}

fn should_retry_scoring_error(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("truncated")
        || lower.contains("empty content")
        || lower.contains("schema mismatch")
        || lower.contains("not valid json")
        || lower.contains("eof while parsing")
}

fn should_retry_insight_error(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("truncated")
        || lower.contains("finish_reason=length")
        || lower.contains("empty content")
        || lower.contains("schema mismatch")
        || lower.contains("not valid json")
        || lower.contains("eof while parsing")
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

    #[test]
    fn scoring_retry_detector_matches_truncation_and_eof() {
        assert!(should_retry_scoring_error(
            "Model output was truncated (finish_reason=length)"
        ));
        assert!(should_retry_scoring_error("EOF while parsing a value at line 1 column 0"));
        assert!(!should_retry_scoring_error("rate limit exceeded"));
    }

    #[test]
    fn openai_request_serializes_json_mode_and_user() {
        let request = OpenAIRequest {
            model: "deepseek-chat".into(),
            messages: vec![OpenAIMessage {
                role: "user".into(),
                content: "{}".into(),
            }],
            temperature: 0.2,
            max_tokens: 512,
            response_format: OpenAIResponseFormat {
                format_type: "json_object".into(),
            },
            user: "pgrn-local-user".into(),
        };

        let value = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(value["response_format"]["type"], "json_object");
        assert_eq!(value["user"], "pgrn-local-user");
    }
}
