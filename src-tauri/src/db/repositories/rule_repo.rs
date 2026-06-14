use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct CategoryRule {
    pub id: i64,
    pub keywords: String,
    pub primary_dim: String,
    pub secondary_dim: Option<String>,
    pub priority: i32,
}

pub fn get_active_rules(conn: &Connection) -> Result<Vec<CategoryRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, keywords, primary_dim, secondary_dim, priority
         FROM category_rules WHERE is_active = 1 ORDER BY priority DESC",
    )?;
    let rules = stmt
        .query_map([], |row| {
            Ok(CategoryRule {
                id: row.get(0)?,
                keywords: row.get(1)?,
                primary_dim: row.get(2)?,
                secondary_dim: row.get(3)?,
                priority: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rules)
}
