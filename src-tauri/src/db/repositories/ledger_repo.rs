use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub id: i64,
    pub date: String,
    pub record_id: Option<i64>,
    pub dimension_key: String,
    pub change_value: i32,
    pub source_title: String,
    pub reason: String,
    pub confidence: Option<f64>,
    pub engine: String,
}

pub fn insert_ledger(
    conn: &Connection,
    date: &str,
    record_id: Option<i64>,
    dimension_key: &str,
    change_value: i32,
    source_title: &str,
    reason: &str,
    confidence: Option<f64>,
    engine: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO stat_ledger
         (date, record_id, dimension_key, change_value, source_title, reason, confidence, engine)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![date, record_id, dimension_key, change_value, source_title, reason, confidence, engine],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_ledger_by_date(conn: &Connection, date: &str) -> Result<Vec<LedgerEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, date, record_id, dimension_key, change_value, source_title, reason, confidence, engine
         FROM stat_ledger WHERE date = ?1 AND is_rollback = 0 ORDER BY created_at"
    )?;
    let entries = stmt.query_map(params![date], |row| {
        Ok(LedgerEntry {
            id: row.get(0)?,
            date: row.get(1)?,
            record_id: row.get(2)?,
            dimension_key: row.get(3)?,
            change_value: row.get(4)?,
            source_title: row.get(5)?,
            reason: row.get(6)?,
            confidence: row.get(7)?,
            engine: row.get(8)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn rollback_ledger(conn: &Connection, ledger_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE stat_ledger SET is_rollback = 1 WHERE id = ?1",
        params![ledger_id],
    )?;
    Ok(())
}

pub fn get_dimension_totals(conn: &Connection) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT dimension_key, SUM(change_value) FROM stat_ledger WHERE is_rollback = 0 GROUP BY dimension_key"
    )?;
    let totals = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(totals)
}

pub fn get_totals_by_date(conn: &Connection, date: &str) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT dimension_key, SUM(change_value)
         FROM stat_ledger
         WHERE date = ?1 AND is_rollback = 0
         GROUP BY dimension_key"
    )?;
    let totals = stmt.query_map(params![date], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(totals)
}

pub fn get_totals_in_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT dimension_key, SUM(change_value)
         FROM stat_ledger
         WHERE date BETWEEN ?1 AND ?2
           AND is_rollback = 0
         GROUP BY dimension_key"
    )?;
    let totals = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(totals)
}

pub fn has_active_entries_for_date(conn: &Connection, date: &str) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT EXISTS(
            SELECT 1 FROM stat_ledger
            WHERE date = ?1 AND is_rollback = 0
         )"
    )?;
    stmt.query_row(params![date], |row| row.get(0))
}

pub fn get_ledger_date(conn: &Connection, ledger_id: i64) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT date FROM stat_ledger WHERE id = ?1")?;
    let mut rows = stmt.query(params![ledger_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn has_ledger_for_date(conn: &Connection, date: &str) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT 1 FROM stat_ledger WHERE date = ?1 AND is_rollback = 0 LIMIT 1"
    )?;
    let exists = stmt.exists(params![date])?;
    Ok(exists)
}
