use crate::db::connection::DbState;
use crate::db::repositories::{api_run_repo, setting_repo};
use crate::services::ai_response_service;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

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

#[tauri::command]
pub async fn call_scoring_api(
    state: State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    let content = execute_api_request(&state, request_json, SCORING_SYSTEM_PROMPT, 2048).await?;
    ai_response_service::normalize_ai_json_string(&content, "scoring api")
}

#[tauri::command]
pub async fn call_plan_api(
    state: State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    let content = execute_api_request(&state, request_json, PLAN_SYSTEM_PROMPT, 3072).await?;
    ai_response_service::normalize_ai_json_string(&content, "plan api")
}

pub async fn execute_plan_api_request(
    state: &State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    execute_api_request(state, request_json, PLAN_SYSTEM_PROMPT, 3072).await
}

pub async fn execute_daily_insight_api_request(
    state: &State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    execute_api_request(state, request_json, DAILY_INSIGHT_SYSTEM_PROMPT, 4096).await
}

async fn execute_api_request(
    state: &State<'_, DbState>,
    request_json: String,
    system_prompt: &str,
    max_tokens: i32,
) -> Result<String, String> {
    let started_at = Instant::now();
    let request_date = extract_request_date(&request_json);

    let (base_url, model, api_key) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let base_url = setting_repo::get_setting(&conn, "api_base_url")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "https://api.openai.com/v1".into());
        let model = setting_repo::get_setting(&conn, "api_model")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "gpt-4o-mini".into());
        let api_key = setting_repo::get_setting(&conn, "api_key")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        (base_url, model, api_key)
    };

    if api_key.is_empty() {
        let error_message = "API Key is not configured".to_string();
        persist_api_run(
            state,
            &request_date,
            &request_json,
            None,
            "error",
            Some(&error_message),
            started_at.elapsed().as_millis() as i64,
            &model,
        );
        return Err(error_message);
    }

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = OpenAIRequest {
        model: model.clone(),
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
        .header("Authorization", format!("Bearer {}", api_key))
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
                &model,
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
            &model,
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
            &model,
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
            &model,
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
        &model,
    );

    Ok(content)
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
    engine_name: &str,
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
            engine_name,
        );
    }
}
