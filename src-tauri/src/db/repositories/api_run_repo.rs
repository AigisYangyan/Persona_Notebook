use rusqlite::{params, Connection, Result};

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
) -> Result<i64> {
    conn.execute(
        "INSERT INTO api_runs
         (date, request_json, response_json, status, error_message, latency_ms, engine_name, task_kind, model_tier, fallback_used)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
        ],
    )?;
    Ok(conn.last_insert_rowid())
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
        )
        .expect("create api run");

        let (status, task_kind, model_tier): (String, String, String) = conn
            .query_row(
                "SELECT status, task_kind, model_tier FROM api_runs WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("query api run status");
        assert_eq!(status, "success");
        assert_eq!(task_kind, "scoring");
        assert_eq!(model_tier, "flash");
    }
}
