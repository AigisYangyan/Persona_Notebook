use rusqlite::{Connection, Result};

#[derive(Debug, Clone)]
pub struct Dimension {
    pub key: String,
    pub name: String,
    pub daily_cap: i32,
}

pub fn get_all_dimensions(conn: &Connection) -> Result<Vec<Dimension>> {
    let mut stmt = conn.prepare(
        "SELECT key, name, daily_cap FROM stat_dimensions ORDER BY sort_order"
    )?;
    let dims = stmt.query_map([], |row| {
        Ok(Dimension {
            key: row.get(0)?,
            name: row.get(1)?,
            daily_cap: row.get(2)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(dims)
}
