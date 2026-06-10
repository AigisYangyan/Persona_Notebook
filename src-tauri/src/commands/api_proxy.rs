use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::connection::DbState;
use crate::db::repositories::{api_run_repo, setting_repo};
use std::time::Instant;

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

#[tauri::command]
pub async fn call_scoring_api(
    state: State<'_, DbState>,
    request_json: String,
) -> Result<String, String> {
    let started_at = Instant::now();
    let request_date = extract_request_date(&request_json);

    // Read settings before await to avoid holding MutexGuard across await
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
        let error_message = "API Key 未配置".to_string();
        persist_api_run(
            &state,
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

    let system_prompt = r#"你是 Personal Growth RPG Notebook 的评分引擎。你的任务是根据用户提供的当日任务记录、确定性 Rules 缓存和评分规则，输出最终的成长评分 JSON。

【五维系统】
- knowledge（学识）：学习、知识、课程、考试、阅读、技术理解
- willpower（觉悟）：自律、执行、抗压、计划完成、长期坚持
- expression（表达）：写作、输出、沟通、摄影、剪辑、自媒体、作品呈现
- physique（体魄）：健身、跑步、睡眠、饮食、健康维护
- bond（羁绊）：社交、合作、人际连接、团队协作、关系维护

【混合模式说明】
1. 本次请求固定为 rules_api 模式。
2. 输入中的 rule_hints 来自确定性规则引擎，是第一轮分类、分值建议和缓存，不是可以忽略的普通参考。
3. request.records 与 rule_hints.record_hints 按相同顺序一一对应。
4. 你必须优先采用 rule_hints 给出的 category、suggested_dimensions、suggested_changes 和 confidence。
5. 只有当任务标题语义与 rule_hints 明显冲突时，才允许小幅修正；修正后仍必须遵守 score_rules、单日上限和最多 3 个维度，并在 reason 中简短说明偏离原因。
6. 如果 rule_hints 已经合理，不要无故扩张维度、抬高分值或重写分类。

【评分原则】
1. 你只根据任务标题（title）、耗时（minutes）、难度星级（difficulty_star）和 rule_hints 评分。
2. 不要过度脑补用户的意图或产出。描述模糊则降低分数和置信度。
3. 单条任务最多影响 3 个维度，不得给所有维度加分。
4. 主维度获得主要成长值，次维度获得少量成长值。
5. 每个维度单日有上限（见 score_rules），返回的 total_changes 不得超过上限。
6. 描述过于模糊的任务（如"做了点事"），confidence 必须 ≤ 0.5，加分应 ≤ 2。
7. 生活杂务类任务（如"吃饭""睡觉"）通常不给分或只给 willpower +1。
8. 输出的是最终评分结果，不是再次回传 rule_hints。

【输出约束】（极其重要）
1. 你必须只输出合法 JSON，不要输出 Markdown 代码块标记（如 ```json）。
2. 不要输出任何自然语言解释、道歉、寒暄或总结性文字。
3. 不要输出除 JSON 以外的任何字符。
4. 确保 JSON 可被标准解析器解析，键名使用英文，值使用正确的数据类型。

【响应格式】
{
  "version": "1.0",
  "date": "YYYY-MM-DD",
  "total_changes": { "knowledge": N, "willpower": N, ... },
  "record_results": [
    {
      "title": "原任务标题",
      "category": "任务分类名称",
      "changes": { "knowledge": N, "willpower": N },
      "difficulty_star": 0-3,
      "confidence": 0.0-1.0,
      "reason": "简短评分理由（15-50字）"
    }
  ],
  "summary": "当日成长总结（30-80字）"
}"#;

    let client = reqwest::Client::new();
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
        max_tokens: 2048,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let message = format!("HTTP 请求失败: {}", e);
            persist_api_run(
                &state,
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
    let text = response
        .text()
        .await
        .map_err(|e| {
            let message = format!("读取响应失败: {}", e);
            persist_api_run(
                &state,
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
        let message = format!("API 返回错误 ({}): {}", status, text);
        persist_api_run(
            &state,
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

    let api_resp: OpenAIResponse =
        serde_json::from_str(&text).map_err(|e| {
            let message = format!("解析 API 响应失败: {}", e);
            persist_api_run(
                &state,
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

    let content = api_resp
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    persist_api_run(
        &state,
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
        .and_then(|value| value.get("date").and_then(|date| date.as_str()).map(str::to_owned))
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
