use crate::models::plan::{PlanAiSession, PlanCycle, PlanItem};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub fn ensure_cycle(
    conn: &Connection,
    period_type: &str,
    start_date: &str,
    end_date: &str,
) -> Result<PlanCycle> {
    conn.execute(
        "INSERT OR IGNORE INTO plan_cycles (
            period_type,
            start_date,
            end_date
        ) VALUES (?1, ?2, ?3)",
        params![period_type, start_date, end_date],
    )?;

    get_cycle_by_period(conn, period_type, start_date, end_date)?
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

pub fn get_cycle_by_period(
    conn: &Connection,
    period_type: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Option<PlanCycle>> {
    conn.query_row(
        "SELECT
            id,
            period_type,
            start_date,
            end_date,
            title,
            summary,
            ai_summary,
            last_ai_run_at
         FROM plan_cycles
         WHERE period_type = ?1 AND start_date = ?2 AND end_date = ?3",
        params![period_type, start_date, end_date],
        map_cycle_row,
    )
    .optional()
}

pub fn get_cycle_by_id(conn: &Connection, id: i64) -> Result<Option<PlanCycle>> {
    conn.query_row(
        "SELECT
            id,
            period_type,
            start_date,
            end_date,
            title,
            summary,
            ai_summary,
            last_ai_run_at
         FROM plan_cycles
         WHERE id = ?1",
        params![id],
        map_cycle_row,
    )
    .optional()
}

pub fn get_cycle_covering_date(
    conn: &Connection,
    period_type: &str,
    date: &str,
) -> Result<Option<PlanCycle>> {
    conn.query_row(
        "SELECT
            id,
            period_type,
            start_date,
            end_date,
            title,
            summary,
            ai_summary,
            last_ai_run_at
         FROM plan_cycles
         WHERE period_type = ?1
           AND start_date <= ?2
           AND end_date >= ?2
         ORDER BY start_date DESC
         LIMIT 1",
        params![period_type, date],
        map_cycle_row,
    )
    .optional()
}

pub fn update_cycle_details(
    conn: &Connection,
    cycle_id: i64,
    title: &str,
    summary: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE plan_cycles
         SET title = ?2,
             summary = ?3,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![cycle_id, title, summary],
    )?;
    Ok(())
}

pub fn update_cycle_ai_summary(
    conn: &Connection,
    cycle_id: i64,
    ai_summary: &str,
    title: &str,
    summary: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE plan_cycles
         SET title = ?2,
             summary = ?3,
             ai_summary = ?4,
             last_ai_run_at = datetime('now', 'localtime'),
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![cycle_id, title, summary, ai_summary],
    )?;
    Ok(())
}

pub fn list_items_by_cycle(conn: &Connection, cycle_id: i64) -> Result<Vec<PlanItem>> {
    let mut stmt = conn.prepare(
        "SELECT
            id,
            cycle_id,
            title,
            description,
            dimension_key,
            progress_percent,
            ai_comment,
            sort_order,
            is_completed
         FROM plan_items
         WHERE cycle_id = ?1
         ORDER BY sort_order, id",
    )?;

    let items = stmt
        .query_map(params![cycle_id], map_item_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

pub fn save_item(
    conn: &Connection,
    item_id: Option<i64>,
    cycle_id: i64,
    title: &str,
    description: &str,
    dimension_key: Option<&str>,
    progress_percent: i32,
    ai_comment: &str,
    sort_order: i32,
    is_completed: bool,
) -> Result<PlanItem> {
    let safe_progress = progress_percent.clamp(0, 100);
    match item_id {
        Some(id) => {
            conn.execute(
                "UPDATE plan_items
                 SET title = ?2,
                     description = ?3,
                     dimension_key = ?4,
                     progress_percent = ?5,
                     ai_comment = ?6,
                     sort_order = ?7,
                     is_completed = ?8,
                     updated_at = datetime('now', 'localtime')
                 WHERE id = ?1",
                params![
                    id,
                    title,
                    description,
                    dimension_key,
                    safe_progress,
                    ai_comment,
                    sort_order,
                    is_completed
                ],
            )?;
            get_item_by_id(conn, id)?.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
        }
        None => {
            conn.execute(
                "INSERT INTO plan_items (
                    cycle_id,
                    title,
                    description,
                    dimension_key,
                    progress_percent,
                    ai_comment,
                    sort_order,
                    is_completed
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    cycle_id,
                    title,
                    description,
                    dimension_key,
                    safe_progress,
                    ai_comment,
                    sort_order,
                    is_completed
                ],
            )?;
            get_item_by_id(conn, conn.last_insert_rowid())?
                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
        }
    }
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM plan_items WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn replace_items_for_cycle(
    conn: &Connection,
    cycle_id: i64,
    items: &[PlanItemInput],
) -> Result<Vec<PlanItem>> {
    conn.execute(
        "DELETE FROM plan_items WHERE cycle_id = ?1",
        params![cycle_id],
    )?;
    for item in items {
        conn.execute(
            "INSERT INTO plan_items (
                cycle_id,
                title,
                description,
                dimension_key,
                progress_percent,
                ai_comment,
                sort_order,
                is_completed
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                cycle_id,
                item.title,
                item.description,
                item.dimension_key,
                item.progress_percent.clamp(0, 100),
                item.ai_comment,
                item.sort_order,
                item.is_completed
            ],
        )?;
    }
    list_items_by_cycle(conn, cycle_id)
}

pub fn create_ai_session(
    conn: &Connection,
    cycle_id: i64,
    request_payload: &str,
    response_payload: Option<&str>,
    questions_json: &str,
    answers_json: &str,
    proposal_json: Option<&str>,
    status: &str,
) -> Result<PlanAiSession> {
    conn.execute(
        "INSERT INTO plan_ai_sessions (
            cycle_id,
            status,
            request_payload,
            response_payload,
            questions_json,
            answers_json,
            proposal_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            cycle_id,
            status,
            request_payload,
            response_payload,
            questions_json,
            answers_json,
            proposal_json
        ],
    )?;
    get_ai_session_by_id(conn, conn.last_insert_rowid())?
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

pub fn update_ai_session_answers(
    conn: &Connection,
    session_id: i64,
    answers_json: &str,
    request_payload: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE plan_ai_sessions
         SET answers_json = ?2,
             request_payload = ?3,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![session_id, answers_json, request_payload],
    )?;
    Ok(())
}

pub fn update_ai_session_resolution(
    conn: &Connection,
    session_id: i64,
    response_payload: &str,
    questions_json: &str,
    proposal_json: Option<&str>,
    status: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE plan_ai_sessions
         SET response_payload = ?2,
             questions_json = ?3,
             proposal_json = ?4,
             status = ?5,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![
            session_id,
            response_payload,
            questions_json,
            proposal_json,
            status
        ],
    )?;
    Ok(())
}

pub fn get_ai_session_by_id(conn: &Connection, id: i64) -> Result<Option<PlanAiSession>> {
    conn.query_row(
        "SELECT
            id,
            cycle_id,
            status,
            request_payload,
            response_payload,
            questions_json,
            answers_json,
            proposal_json
         FROM plan_ai_sessions
         WHERE id = ?1",
        params![id],
        |row| {
            Ok(PlanAiSession {
                id: row.get(0)?,
                cycle_id: row.get(1)?,
                status: row.get(2)?,
                request_payload: row.get(3)?,
                response_payload: row.get(4)?,
                questions_json: row.get(5)?,
                answers_json: row.get(6)?,
                proposal_json: row.get(7)?,
            })
        },
    )
    .optional()
}

pub fn get_latest_ai_session_for_cycle(
    conn: &Connection,
    cycle_id: i64,
    status: Option<&str>,
) -> Result<Option<PlanAiSession>> {
    match status {
        Some(value) => conn
            .query_row(
                "SELECT
                    id,
                    cycle_id,
                    status,
                    request_payload,
                    response_payload,
                    questions_json,
                    answers_json,
                    proposal_json
                 FROM plan_ai_sessions
                 WHERE cycle_id = ?1 AND status = ?2
                 ORDER BY id DESC
                 LIMIT 1",
                params![cycle_id, value],
                |row| {
                    Ok(PlanAiSession {
                        id: row.get(0)?,
                        cycle_id: row.get(1)?,
                        status: row.get(2)?,
                        request_payload: row.get(3)?,
                        response_payload: row.get(4)?,
                        questions_json: row.get(5)?,
                        answers_json: row.get(6)?,
                        proposal_json: row.get(7)?,
                    })
                },
            )
            .optional(),
        None => conn
            .query_row(
                "SELECT
                    id,
                    cycle_id,
                    status,
                    request_payload,
                    response_payload,
                    questions_json,
                    answers_json,
                    proposal_json
                 FROM plan_ai_sessions
                 WHERE cycle_id = ?1
                 ORDER BY id DESC
                 LIMIT 1",
                params![cycle_id],
                |row| {
                    Ok(PlanAiSession {
                        id: row.get(0)?,
                        cycle_id: row.get(1)?,
                        status: row.get(2)?,
                        request_payload: row.get(3)?,
                        response_payload: row.get(4)?,
                        questions_json: row.get(5)?,
                        answers_json: row.get(6)?,
                        proposal_json: row.get(7)?,
                    })
                },
            )
            .optional(),
    }
}

pub fn list_week_cycles_in_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<PlanCycle>> {
    let mut stmt = conn.prepare(
        "SELECT
            id,
            period_type,
            start_date,
            end_date,
            title,
            summary,
            ai_summary,
            last_ai_run_at
         FROM plan_cycles
         WHERE period_type = 'week'
           AND start_date >= ?1
           AND end_date <= ?2
         ORDER BY start_date",
    )?;

    let cycles = stmt
        .query_map(params![start_date, end_date], map_cycle_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(cycles)
}

pub fn next_sort_order(conn: &Connection, cycle_id: i64) -> Result<i32> {
    conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM plan_items WHERE cycle_id = ?1",
        params![cycle_id],
        |row| row.get(0),
    )
}

#[derive(Debug, Clone)]
pub struct PlanItemInput {
    pub title: String,
    pub description: String,
    pub dimension_key: Option<String>,
    pub progress_percent: i32,
    pub ai_comment: String,
    pub sort_order: i32,
    pub is_completed: bool,
}

pub fn get_item_by_id(conn: &Connection, id: i64) -> Result<Option<PlanItem>> {
    conn.query_row(
        "SELECT
            id,
            cycle_id,
            title,
            description,
            dimension_key,
            progress_percent,
            ai_comment,
            sort_order,
            is_completed
         FROM plan_items
         WHERE id = ?1",
        params![id],
        map_item_row,
    )
    .optional()
}

fn map_cycle_row(row: &rusqlite::Row<'_>) -> Result<PlanCycle> {
    Ok(PlanCycle {
        id: row.get(0)?,
        period_type: row.get(1)?,
        start_date: row.get(2)?,
        end_date: row.get(3)?,
        title: row.get(4)?,
        summary: row.get(5)?,
        ai_summary: row.get(6)?,
        last_ai_run_at: row.get(7)?,
    })
}

fn map_item_row(row: &rusqlite::Row<'_>) -> Result<PlanItem> {
    Ok(PlanItem {
        id: row.get(0)?,
        cycle_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        dimension_key: row.get(4)?,
        progress_percent: row.get(5)?,
        ai_comment: row.get(6)?,
        sort_order: row.get(7)?,
        is_completed: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn ensure_cycle_creates_and_reads_back_cycle() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        let cycle = ensure_cycle(&conn, "week", "2026-06-08", "2026-06-14").expect("cycle");

        assert_eq!(cycle.period_type, "week");
        assert_eq!(cycle.start_date, "2026-06-08");
        assert_eq!(cycle.end_date, "2026-06-14");
    }

    #[test]
    fn replace_items_for_cycle_replaces_existing_rows() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");
        let cycle = ensure_cycle(&conn, "week", "2026-06-08", "2026-06-14").expect("cycle");
        save_item(
            &conn,
            None,
            cycle.id,
            "Old",
            "",
            Some("knowledge"),
            10,
            "",
            0,
            false,
        )
        .expect("seed item");

        let items = replace_items_for_cycle(
            &conn,
            cycle.id,
            &[PlanItemInput {
                title: "New".into(),
                description: "Desc".into(),
                dimension_key: Some("willpower".into()),
                progress_percent: 40,
                ai_comment: "AI".into(),
                sort_order: 0,
                is_completed: false,
            }],
        )
        .expect("replace items");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "New");
        assert_eq!(items[0].dimension_key.as_deref(), Some("willpower"));
    }

    #[test]
    fn get_latest_ai_session_for_cycle_filters_by_status() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");
        let cycle = ensure_cycle(&conn, "week", "2026-06-08", "2026-06-14").expect("cycle");

        create_ai_session(&conn, cycle.id, "{}", Some("{}"), "[]", "[]", None, "ready")
            .expect("ready session");
        create_ai_session(
            &conn,
            cycle.id,
            "{}",
            Some("{}"),
            "[\"question\"]",
            "[]",
            None,
            "clarifying",
        )
        .expect("clarifying session");

        let session = get_latest_ai_session_for_cycle(&conn, cycle.id, Some("clarifying"))
            .expect("query session")
            .expect("session");
        assert_eq!(session.status, "clarifying");
    }
}
