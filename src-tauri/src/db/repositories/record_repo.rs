use crate::models::record::Record;
use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
pub struct RecordDateCount {
    pub date: String,
    pub count: i32,
}

pub fn create_record(
    conn: &Connection,
    date: &str,
    title: &str,
    minutes: i32,
    difficulty_star: i32,
) -> Result<i64> {
    create_record_with_options(
        conn,
        date,
        title,
        minutes,
        difficulty_star,
        None,
        "stopwatch",
        None,
    )
}

pub fn create_record_with_options(
    conn: &Connection,
    date: &str,
    title: &str,
    minutes: i32,
    difficulty_star: i32,
    parent_id: Option<i64>,
    timer_mode: &str,
    countdown_target_seconds: Option<i32>,
) -> Result<i64> {
    let safe_minutes = minutes.max(0);
    let elapsed_seconds = i64::from(safe_minutes) * 60;
    conn.execute(
        "INSERT INTO records (
            date,
            title,
            minutes,
            difficulty_star,
            parent_id,
            is_completed,
            elapsed_seconds,
            timer_mode,
            countdown_target_seconds
        ) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8)",
        params![
            date,
            title,
            safe_minutes,
            difficulty_star,
            parent_id,
            elapsed_seconds,
            timer_mode,
            countdown_target_seconds
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_sub_record(
    conn: &Connection,
    parent_id: i64,
    title: &str,
    difficulty_star: i32,
    timer_mode: &str,
    countdown_target_seconds: Option<i32>,
) -> Result<i64> {
    let parent = get_record_by_id(conn, parent_id)?;
    create_record_with_options(
        conn,
        &parent.date,
        title,
        0,
        difficulty_star,
        Some(parent_id),
        timer_mode,
        countdown_target_seconds,
    )
}

pub fn create_sub_record_with_minutes(
    conn: &Connection,
    parent_id: i64,
    title: &str,
    minutes: i32,
    difficulty_star: i32,
) -> Result<i64> {
    let parent = get_record_by_id(conn, parent_id)?;
    create_record_with_options(
        conn,
        &parent.date,
        title,
        minutes,
        difficulty_star,
        Some(parent_id),
        "stopwatch",
        None,
    )
}

pub fn get_records_by_date(conn: &Connection, date: &str) -> Result<Vec<Record>> {
    let mut stmt = conn.prepare(
        "SELECT
            id,
            date,
            title,
            minutes,
            difficulty_star,
            parent_id,
            is_completed,
            completed_at,
            elapsed_seconds,
            timer_mode,
            countdown_target_seconds,
            timer_started_at
         FROM records
         WHERE date = ?1
         ORDER BY created_at",
    )?;
    let records = stmt
        .query_map(params![date], map_record_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(records)
}

pub fn get_record_by_id(conn: &Connection, id: i64) -> Result<Record> {
    let mut stmt = conn.prepare(
        "SELECT
            id,
            date,
            title,
            minutes,
            difficulty_star,
            parent_id,
            is_completed,
            completed_at,
            elapsed_seconds,
            timer_mode,
            countdown_target_seconds,
            timer_started_at
         FROM records
         WHERE id = ?1",
    )?;
    stmt.query_row(params![id], map_record_row)
}

pub fn delete_record(conn: &Connection, id: i64) -> Result<()> {
    for child_id in get_child_ids(conn, id)? {
        delete_record(conn, child_id)?;
    }
    conn.execute(
        "DELETE FROM record_timer_sessions WHERE record_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM records WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn start_record_timer(conn: &Connection, id: i64, started_at: &str) -> Result<()> {
    pause_all_running_except(conn, id, started_at)?;
    let record = get_record_by_id(conn, id)?;
    if record.timer_started_at.is_some() {
        return Ok(());
    }

    conn.execute(
        "UPDATE records
         SET timer_started_at = ?2,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![id, started_at],
    )?;
    conn.execute(
        "INSERT INTO record_timer_sessions (record_id, started_at) VALUES (?1, ?2)",
        params![id, started_at],
    )?;
    Ok(())
}

pub fn pause_record_timer(conn: &Connection, id: i64, ended_at: &str) -> Result<()> {
    let record = get_record_by_id(conn, id)?;
    let Some(started_at) = record.timer_started_at.as_deref() else {
        return Ok(());
    };

    let delta_seconds = seconds_between(started_at, ended_at).max(0);
    let elapsed_seconds = record.elapsed_seconds + delta_seconds;
    let minutes = minutes_from_seconds(elapsed_seconds);

    conn.execute(
        "UPDATE records
         SET elapsed_seconds = ?2,
             minutes = ?3,
             timer_started_at = NULL,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![id, elapsed_seconds, minutes],
    )?;
    conn.execute(
        "UPDATE record_timer_sessions
         SET ended_at = ?2,
             duration_seconds = ?3
         WHERE id = (
            SELECT id
            FROM record_timer_sessions
            WHERE record_id = ?1 AND ended_at IS NULL
            ORDER BY id DESC
            LIMIT 1
         )",
        params![id, ended_at, delta_seconds],
    )?;
    Ok(())
}

pub fn reset_record_timer(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE records
         SET elapsed_seconds = 0,
             minutes = 0,
             timer_started_at = NULL,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM record_timer_sessions WHERE record_id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn toggle_record_completed(
    conn: &Connection,
    id: i64,
    is_completed: bool,
    completed_at: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE records
         SET is_completed = ?2,
             completed_at = ?3,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![id, is_completed, completed_at],
    )?;
    Ok(())
}

pub fn update_record_timer_mode(
    conn: &Connection,
    id: i64,
    timer_mode: &str,
    countdown_target_seconds: Option<i32>,
) -> Result<()> {
    conn.execute(
        "UPDATE records
         SET timer_mode = ?2,
             countdown_target_seconds = ?3,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![id, timer_mode, countdown_target_seconds],
    )?;
    Ok(())
}

pub fn get_record_counts_in_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<RecordDateCount>> {
    let mut stmt = conn.prepare(
        "SELECT date, COUNT(*) as count
         FROM records
         WHERE date BETWEEN ?1 AND ?2
         GROUP BY date
         ORDER BY date",
    )?;
    let counts = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok(RecordDateCount {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(counts)
}

fn get_child_ids(conn: &Connection, parent_id: i64) -> Result<Vec<i64>> {
    let mut stmt =
        conn.prepare("SELECT id FROM records WHERE parent_id = ?1 ORDER BY created_at")?;
    let ids = stmt
        .query_map(params![parent_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

fn pause_all_running_except(conn: &Connection, except_id: i64, ended_at: &str) -> Result<()> {
    let mut stmt =
        conn.prepare("SELECT id FROM records WHERE timer_started_at IS NOT NULL AND id != ?1")?;
    let running_ids = stmt
        .query_map(params![except_id], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for running_id in running_ids {
        pause_record_timer(conn, running_id, ended_at)?;
    }
    Ok(())
}

fn map_record_row(row: &rusqlite::Row<'_>) -> Result<Record> {
    Ok(Record {
        id: row.get(0)?,
        date: row.get(1)?,
        title: row.get(2)?,
        minutes: row.get(3)?,
        difficulty_star: row.get(4)?,
        parent_id: row.get(5)?,
        is_completed: row.get(6)?,
        completed_at: row.get(7)?,
        elapsed_seconds: row.get(8)?,
        timer_mode: row.get(9)?,
        countdown_target_seconds: row.get(10)?,
        timer_started_at: row.get(11)?,
    })
}

fn minutes_from_seconds(elapsed_seconds: i64) -> i32 {
    if elapsed_seconds <= 0 {
        0
    } else {
        ((elapsed_seconds + 59) / 60) as i32
    }
}

fn seconds_between(started_at: &str, ended_at: &str) -> i64 {
    let parsed_start = NaiveDateTime::parse_from_str(started_at, "%Y-%m-%d %H:%M:%S");
    let parsed_end = NaiveDateTime::parse_from_str(ended_at, "%Y-%m-%d %H:%M:%S");
    match (parsed_start, parsed_end) {
        (Ok(start), Ok(end)) => (end - start).num_seconds(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn starts_new_timer_and_pauses_the_previous_running_task() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        let first_id = create_record(&conn, "2026-06-10", "任务A", 0, 0).expect("create first");
        let second_id = create_record(&conn, "2026-06-10", "任务B", 0, 0).expect("create second");

        start_record_timer(&conn, first_id, "2026-06-10 10:00:00").expect("start first");
        start_record_timer(&conn, second_id, "2026-06-10 10:05:00").expect("start second");

        let records = get_records_by_date(&conn, "2026-06-10").expect("load records");
        let first = records
            .iter()
            .find(|record| record.id == first_id)
            .expect("first task");
        let second = records
            .iter()
            .find(|record| record.id == second_id)
            .expect("second task");

        assert_eq!(first.elapsed_seconds, 300);
        assert_eq!(first.minutes, 5);
        assert!(first.timer_started_at.is_none());
        assert_eq!(
            second.timer_started_at.as_deref(),
            Some("2026-06-10 10:05:00")
        );
    }

    #[test]
    fn pause_accumulates_elapsed_seconds_and_rounds_minutes_up() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        let record_id = create_record(&conn, "2026-06-10", "任务A", 0, 0).expect("create task");

        start_record_timer(&conn, record_id, "2026-06-10 10:00:00").expect("start timer");
        pause_record_timer(&conn, record_id, "2026-06-10 10:01:01").expect("pause timer");

        let records = get_records_by_date(&conn, "2026-06-10").expect("load records");
        let record = records
            .into_iter()
            .find(|item| item.id == record_id)
            .expect("record");

        assert_eq!(record.elapsed_seconds, 61);
        assert_eq!(record.minutes, 2);
        assert!(record.timer_started_at.is_none());
    }
}
