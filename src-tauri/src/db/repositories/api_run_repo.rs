use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
pub struct ApiRunRecord {
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

pub fn create_run(
    conn: &Connection,
    date: &str,
    request_json: &str,
    response_json: Option<&str>,
    status: &str,
    error_message: Option<&str>,
    latency_ms: i64,
    engine_name: &str,
    task_kind: &str,
    model_tier: &str,
    fallback_used: bool,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    prompt_cache_hit_tokens: Option<i64>,
    prompt_cache_miss_tokens: Option<i64>,
    finish_reason: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO api_runs
         (date, request_json, response_json, status, error_message, latency_ms, engine_name, task_kind, model_tier, fallback_used,
          prompt_tokens, completion_tokens, prompt_cache_hit_tokens, prompt_cache_miss_tokens, finish_reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            date,
            request_json,
            response_json,
            status,
            error_message,
            latency_ms,
            engine_name,
            task_kind,
            model_tier,
            if fallback_used { 1 } else { 0 },
            prompt_tokens,
            completion_tokens,
            prompt_cache_hit_tokens,
            prompt_cache_miss_tokens,
            finish_reason,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_recent_runs(conn: &Connection, limit: i64) -> Result<Vec<ApiRunRecord>> {
    let mut stmt = conn.prepare(
        "SELECT
            id,
            date,
            status,
            error_message,
            latency_ms,
            engine_name,
            task_kind,
            model_tier,
            fallback_used,
            prompt_tokens,
            completion_tokens,
            prompt_cache_hit_tokens,
            prompt_cache_miss_tokens,
            finish_reason,
            created_at
         FROM api_runs
         ORDER BY id DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit.max(1)], |row| {
        Ok(ApiRunRecord {
            id: row.get(0)?,
            date: row.get(1)?,
            status: row.get(2)?,
            error_message: row.get(3)?,
            latency_ms: row.get(4)?,
            engine_name: row.get(5)?,
            task_kind: row.get(6)?,
            model_tier: row.get(7)?,
            fallback_used: row.get::<_, i32>(8)? == 1,
            prompt_tokens: row.get(9)?,
            completion_tokens: row.get(10)?,
            prompt_cache_hit_tokens: row.get(11)?,
            prompt_cache_miss_tokens: row.get(12)?,
            finish_reason: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;

    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn create_run_persists_api_log() {
        let conn = Connection::open_in_memory().expect("open in memory db");
        run_migrations(&conn).expect("migrations");

        let id = create_run(
            &conn,
            "2026-06-10",
            "{\"date\":\"2026-06-10\"}",
            Some("{\"ok\":true}"),
            "success",
            None,
            123,
            "deepseek-chat",
            "scoring",
            "flash",
            false,
            Some(256),
            Some(96),
            Some(200),
            Some(56),
            Some("stop"),
        )
        .expect("create api run");

        let (status, task_kind, model_tier, finish_reason): (String, String, String, Option<String>) = conn
            .query_row(
                "SELECT status, task_kind, model_tier, finish_reason FROM api_runs WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("query api run status");
        assert_eq!(status, "success");
        assert_eq!(task_kind, "scoring");
        assert_eq!(model_tier, "flash");
        assert_eq!(finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn list_recent_runs_returns_newest_first() {
        let conn = Connection::open_in_memory().expect("open in memory db");
        run_migrations(&conn).expect("migrations");

        for index in 0..2 {
            create_run(
                &conn,
                "2026-06-10",
                "{\"date\":\"2026-06-10\"}",
                Some("{\"ok\":true}"),
                "success",
                None,
                100 + index,
                "deepseek-chat",
                "scoring",
                "flash",
                false,
                None,
                None,
                None,
                None,
                Some("stop"),
            )
            .expect("create api run");
        }

        let runs = list_recent_runs(&conn, 2).expect("list recent runs");
        assert_eq!(runs.len(), 2);
        assert!(runs[0].id > runs[1].id);
    }
}
