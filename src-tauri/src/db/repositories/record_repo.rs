use rusqlite::{Connection, Result, params};
use crate::models::record::Record;

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
    conn.execute(
        "INSERT INTO records (date, title, minutes, difficulty_star) VALUES (?1, ?2, ?3, ?4)",
        params![date, title, minutes, difficulty_star],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_records_by_date(conn: &Connection, date: &str) -> Result<Vec<Record>> {
    let mut stmt = conn.prepare(
        "SELECT id, date, title, minutes, difficulty_star FROM records WHERE date = ?1 ORDER BY created_at"
    )?;
    let records = stmt.query_map(params![date], |row| {
        Ok(Record {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            minutes: row.get(3)?,
            difficulty_star: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(records)
}

pub fn delete_record(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM records WHERE id = ?1", params![id])?;
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
         ORDER BY date"
    )?;
    let counts = stmt.query_map(params![start_date, end_date], |row| {
        Ok(RecordDateCount {
            date: row.get(0)?,
            count: row.get(1)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(counts)
}
