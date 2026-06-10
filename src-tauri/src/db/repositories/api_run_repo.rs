use rusqlite::{Connection, Result, params};

pub fn create_run(
    conn: &Connection,
    date: &str,
    request_json: &str,
    response_json: Option<&str>,
    status: &str,
    error_message: Option<&str>,
    latency_ms: i64,
    engine_name: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO api_runs
         (date, request_json, response_json, status, error_message, latency_ms, engine_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            date,
            request_json,
            response_json,
            status,
            error_message,
            latency_ms,
            engine_name,
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
            "gpt-4o-mini",
        )
        .expect("create api run");

        let status: String = conn
            .query_row(
                "SELECT status FROM api_runs WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("query api run status");
        assert_eq!(status, "success");
    }
}
