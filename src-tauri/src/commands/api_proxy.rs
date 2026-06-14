use crate::db::connection::DbState;
use crate::db::repositories::{api_run_repo, setting_repo};
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

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

const SCORING_SYSTEM_PROMPT: &str = r#"你是 Personal Growth RPG Notebook 的评分引擎。
你的任务是根据用户提供的当日任务记录、规则缓存和评分规则，输出最终成长评分 JSON。

规则：
1. 只输出合法 JSON，不要 Markdown、解释、寒暄或额外文字。
2. 评分必须优先参考输入中的 rule_hints。
3. 每条任务最多影响 3 个维度，并遵守单日上限。
4. 描述模糊的任务要降低 confidence 和分值。

输出格式：
{
  "version": "1.0",
  "date": "YYYY-MM-DD",
  "total_changes": { "knowledge": 0, "willpower": 0, "expression": 0, "physique": 0, "bond": 0 },
  "record_results": [
    {
      "title": "原任务标题",
      "category": "任务分类",
      "changes": { "knowledge": 0 },
      "difficulty_star": 0,
      "confidence": 0.0,
      "reason": "15-50字简短理由"
    }
  ],
  "summary": "0-80字的当日成长总结"
}"#;

const PLAN_SYSTEM_PROMPT: &str = r#"你是 Personal Growth RPG Notebook 的计划调整引擎。
你会根据周计划或月计划、Goal 条目、成长账本汇总，以及用户补充回答，输出结构化 JSON。

要求：
1. 只输出合法 JSON，不要 Markdown、解释、寒暄或额外文字。
2. 如果 Goal 过于笼统、缺少对象、动作、节奏或成功标准，先要求澄清，不要直接瞎改。
3. 如果信息足够，输出一份可应用的 proposal。
4. progress_percent 必须在 0 到 100 之间。
5. ai_summary 用一句简短总结说明当前周期推进状态。
6. items 要保持 Goal 结构化，标题短、描述具体、维度合理。

输出格式一：需要澄清
{
  "requires_clarification": true,
  "questions": ["问题1", "问题2"],
  "proposal": null
}

输出格式二：可直接应用
{
  "requires_clarification": false,
  "questions": [],
  "proposal": {
    "title": "计划标题",
    "summary": "用户可见的计划摘要",
    "ai_summary": "一句话总结当前推进",
    "items": [
      {
        "title": "目标标题",
        "description": "更具体的目标描述",
        "dimension_key": "knowledge",
        "progress_percent": 35,
        "ai_comment": "为什么给出这个进度",
        "sort_order": 0,
        "is_completed": false
      }
    ]
  }
}"#;

const DAILY_INSIGHT_SYSTEM_PROMPT: &str = r#"你是 Personal Growth RPG Notebook 的个人洞察引擎。你只能基于用户提供的 JSON 上下文输出，不能编造没有证据的经历、情绪、关系或计划。

通用规则：
1. 只输出合法 JSON，不要 Markdown、寒暄或额外解释。
2. 每条判断都要尽量引用 context.evidence_index 中的 evidence_id；没有证据时写入 insufficient_evidence。
3. 输出必须包含 report 和 memory_delta 两部分。memory_delta 必须符合 PersonalMemoryPatch v1；没有可更新记忆时使用空 memory_operations。
4. memory_delta 不允许覆盖 birthday；长期记忆只能来自上下文证据。
5. 风格要温暖、诚实、细腻、可执行。可以有想象力，但想象力只能用于表达和鼓励，不能用于虚构事实。
6. 如果数据不足，要直接承认不足，并把重点放在可验证的观察与下一步建议上。

塔罗模式：
- 塔罗是“心理原型牌”，不是命运占卜，不做超自然断言。
- 文字可以更长、更有氛围、更有期待感，但必须 grounded，不能脱离证据上下文。
- 设计元素尽量贴近心理、体力、恢复、行动节奏、情绪张力和内在叙事。
- 可以引用一句适合当下情境的温暖名句，但不要冒充精确出处；如果不确定出处，只写 quote 本身，不写作者。
- report 必须包含：
  card_name,
  archetype,
  psychological_theme,
  body_signal,
  warm_quote,
  encouragement,
  action,
  risk_reminder,
  deeper_reading,
  evidence_ids
- encouragement 写成 2-4 句，带有情绪支撑感。
- deeper_reading 写成 1 段较长解释，允许开放联想，但必须回扣已有记录、计划、日记、羁绊或长期记忆。
- action 输出 2-4 条可执行建议。

报告模式：
- report 必须按照 daily / weekly / monthly 之一组织。
- 报告不是简单复盘，要做更深层的剖析：看见完成与未完成，看见背后的节奏、阻力、情绪模式和计划推进偏差。
- 不能乱说“你一定怎样”，只能基于证据提出“更可能的解释”。
- report 必须包含：
  completed,
  unfinished,
  time_focus,
  growth_changes,
  plan_progress,
  journal_and_bond_observations,
  root_causes,
  leverage_points,
  concrete_remedies,
  not_enough_data,
  next_actions,
  evidence_ids
- completed / unfinished / next_actions 建议输出为字符串数组。
- time_focus / growth_changes / plan_progress / journal_and_bond_observations / root_causes / leverage_points 可写成较完整段落。
- concrete_remedies 输出 2-5 条具体措施，优先具体、可执行、可验证。

输出格式：
{
  "schema_version": "1.0",
  "title": "短标题",
  "summary": "80字以内摘要",
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
    execute_api_request(&state, request_json, SCORING_SYSTEM_PROMPT, 2048).await
}

#[tauri::command]
pub async fn call_plan_api(
    state: State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    execute_api_request(&state, request_json, PLAN_SYSTEM_PROMPT, 3072).await
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

    let api_response: OpenAIResponse = serde_json::from_str(&text).map_err(|e| {
        let message = format!("Failed to parse API envelope: {e}");
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
        message
    })?;

    let content = api_response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .unwrap_or_default();

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
