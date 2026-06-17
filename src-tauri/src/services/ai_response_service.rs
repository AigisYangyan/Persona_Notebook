use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsageStats {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub prompt_cache_hit_tokens: Option<i64>,
    pub prompt_cache_miss_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatResponseMeta {
    pub finish_reason: Option<String>,
    pub usage: UsageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatResponseContent {
    pub content: String,
    pub meta: ChatResponseMeta,
}

pub fn clean_json_payload(raw: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('\u{feff}').trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```JSON")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

pub fn parse_ai_json<T: DeserializeOwned>(raw: &str, label: &str) -> Result<T, String> {
    let value = parse_ai_json_value(raw, label)?;
    serde_json::from_value(value).map_err(|error| format!("{label} schema mismatch: {error}"))
}

pub fn parse_ai_json_value(raw: &str, label: &str) -> Result<Value, String> {
    let clean = clean_json_payload(raw);
    if clean.is_empty() {
        return Err(format!("{label} returned empty content"));
    }

    serde_json::from_str(&clean).map_err(|error| {
        if looks_truncated_json(&clean, &error.to_string()) {
            format!("{label} returned truncated JSON: {error}")
        } else {
            format!("{label} is not valid JSON: {error}")
        }
    })
}

pub fn normalize_ai_json_string(raw: &str, label: &str) -> Result<String, String> {
    let value = parse_ai_json_value(raw, label)?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("{label} could not be serialized: {error}"))
}

pub fn extract_chat_content(response_text: &str) -> Result<String, String> {
    extract_chat_response(response_text).map(|payload| payload.content)
}

pub fn extract_chat_response(response_text: &str) -> Result<ChatResponseContent, String> {
    let envelope: Value = serde_json::from_str(response_text)
        .map_err(|error| format!("Failed to parse API envelope: {error}"))?;

    if let Some(error_obj) = envelope.get("error") {
        return Err(format!("API returned error payload: {error_obj}"));
    }

    let choices = envelope
        .get("choices")
        .and_then(Value::as_array)
        .ok_or_else(|| "API envelope missing choices".to_string())?;
    let choice = choices
        .first()
        .ok_or_else(|| "API envelope returned no choices".to_string())?;

    let finish_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .map(str::to_string);

    if finish_reason.as_deref().is_some_and(|reason| reason == "length") {
        return Err("Model output was truncated (finish_reason=length)".to_string());
    }

    let message = choice
        .get("message")
        .ok_or_else(|| "API envelope missing choice.message".to_string())?;
    let content = message
        .get("content")
        .ok_or_else(|| "API envelope missing message.content".to_string())?;

    let content = extract_content_text(content).ok_or_else(|| "API returned empty content".to_string())?;
    let usage = extract_usage_stats(&envelope);

    Ok(ChatResponseContent {
        content,
        meta: ChatResponseMeta {
            finish_reason,
            usage,
        },
    })
}

fn extract_content_text(content: &Value) -> Option<String> {
    match content {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Value::Array(parts) => {
            let text = parts
                .iter()
                .filter_map(|part| match part {
                    Value::Object(map) => map
                        .get("text")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}

fn looks_truncated_json(clean: &str, error_message: &str) -> bool {
    error_message.contains("EOF while parsing")
        || brace_delta(clean, '{', '}') > 0
        || brace_delta(clean, '[', ']') > 0
}

fn extract_usage_stats(envelope: &Value) -> UsageStats {
    let usage = envelope.get("usage").unwrap_or(&Value::Null);
    UsageStats {
        prompt_tokens: usage.get("prompt_tokens").and_then(Value::as_i64),
        completion_tokens: usage.get("completion_tokens").and_then(Value::as_i64),
        prompt_cache_hit_tokens: usage
            .get("prompt_cache_hit_tokens")
            .and_then(Value::as_i64),
        prompt_cache_miss_tokens: usage
            .get("prompt_cache_miss_tokens")
            .and_then(Value::as_i64),
    }
}

fn brace_delta(input: &str, open: char, close: char) -> i32 {
    input.chars().fold(0, |count, ch| {
        if ch == open {
            count + 1
        } else if ch == close {
            count - 1
        } else {
            count
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct DemoPayload {
        title: String,
    }

    #[test]
    fn parses_fenced_json_payload() {
        let parsed: DemoPayload = parse_ai_json("```json\n{\"title\":\"ok\"}\n```", "demo payload")
            .expect("parse fenced payload");
        assert_eq!(parsed.title, "ok");
    }

    #[test]
    fn detects_empty_payload() {
        let result = parse_ai_json_value("   ", "demo payload");
        assert!(result.is_err());
    }

    #[test]
    fn detects_truncated_json() {
        let result = parse_ai_json_value("{\"title\":", "demo payload");
        assert!(result.expect_err("should fail").contains("truncated JSON"));
    }

    #[test]
    fn extracts_string_chat_content() {
        let content = extract_chat_content(
            r#"{"choices":[{"message":{"content":"{\"ok\":true}"},"finish_reason":"stop"}]}"#,
        )
        .expect("extract content");
        assert_eq!(content, "{\"ok\":true}");
    }

    #[test]
    fn extracts_chat_meta_and_usage() {
        let payload = extract_chat_response(
            r#"{
              "choices":[{"message":{"content":"{\"ok\":true}"},"finish_reason":"stop"}],
              "usage":{"prompt_tokens":128,"completion_tokens":64,"prompt_cache_hit_tokens":96,"prompt_cache_miss_tokens":32}
            }"#,
        )
        .expect("extract payload");
        assert_eq!(payload.content, "{\"ok\":true}");
        assert_eq!(payload.meta.finish_reason.as_deref(), Some("stop"));
        assert_eq!(payload.meta.usage.prompt_tokens, Some(128));
        assert_eq!(payload.meta.usage.completion_tokens, Some(64));
        assert_eq!(payload.meta.usage.prompt_cache_hit_tokens, Some(96));
        assert_eq!(payload.meta.usage.prompt_cache_miss_tokens, Some(32));
    }

    #[test]
    fn extracts_array_chat_content() {
        let content = extract_chat_content(
            r#"{"choices":[{"message":{"content":[{"type":"text","text":"{\"ok\":true}"}]},"finish_reason":"stop"}]}"#,
        )
        .expect("extract content");
        assert_eq!(content, "{\"ok\":true}");
    }

    #[test]
    fn rejects_length_finish_reason() {
        let error = extract_chat_content(
            r#"{"choices":[{"message":{"content":"{}"},"finish_reason":"length"}]}"#,
        )
        .expect_err("should reject");
        assert!(error.contains("truncated"));
    }
}
