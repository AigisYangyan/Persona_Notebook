use rusqlite::{Connection, Result, params};

use super::ledger_repo;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DailyDimensionTotals {
    pub knowledge: i32,
    pub willpower: i32,
    pub expression: i32,
    pub physique: i32,
    pub bond: i32,
}

impl DailyDimensionTotals {
    pub fn from_pairs(pairs: &[(String, i32)]) -> Self {
        let mut totals = Self::default();
        for (key, value) in pairs {
            match key.as_str() {
                "knowledge" => totals.knowledge = *value,
                "willpower" => totals.willpower = *value,
                "expression" => totals.expression = *value,
                "physique" => totals.physique = *value,
                "bond" => totals.bond = *value,
                _ => {}
            }
        }
        totals
    }

    pub fn total_points(&self) -> i32 {
        self.knowledge + self.willpower + self.expression + self.physique + self.bond
    }
}

#[derive(Debug, Clone)]
pub struct DailyReview {
    pub date: String,
    pub totals: DailyDimensionTotals,
    pub summary_text: Option<String>,
    pub is_analyzed: bool,
}

#[derive(Debug, Clone)]
pub struct DailyReviewFlag {
    pub date: String,
    pub is_analyzed: bool,
}

pub fn get_review(conn: &Connection, date: &str) -> Result<Option<DailyReview>> {
    let mut stmt = conn.prepare(
        "SELECT date, total_knowledge, total_willpower, total_expression, total_physique, total_bond, summary_text, is_analyzed
         FROM daily_reviews WHERE date = ?1"
    )?;
    let mut rows = stmt.query(params![date])?;
    if let Some(row) = rows.next()? {
        Ok(Some(DailyReview {
            date: row.get(0)?,
            totals: DailyDimensionTotals {
                knowledge: row.get(1)?,
                willpower: row.get(2)?,
                expression: row.get(3)?,
                physique: row.get(4)?,
                bond: row.get(5)?,
            },
            summary_text: row.get(6)?,
            is_analyzed: row.get::<_, i32>(7)? != 0,
        }))
    } else {
        Ok(None)
    }
}

pub fn upsert_review(
    conn: &Connection,
    date: &str,
    totals: &DailyDimensionTotals,
    summary_text: Option<&str>,
    is_analyzed: bool,
) -> Result<()> {
    conn.execute(
        "INSERT INTO daily_reviews
         (date, total_knowledge, total_willpower, total_expression, total_physique, total_bond, summary_text, is_analyzed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(date) DO UPDATE SET
           total_knowledge = excluded.total_knowledge,
           total_willpower = excluded.total_willpower,
           total_expression = excluded.total_expression,
           total_physique = excluded.total_physique,
           total_bond = excluded.total_bond,
           summary_text = excluded.summary_text,
           is_analyzed = excluded.is_analyzed,
           updated_at = datetime('now', 'localtime')",
        params![
            date,
            totals.knowledge,
            totals.willpower,
            totals.expression,
            totals.physique,
            totals.bond,
            summary_text,
            if is_analyzed { 1 } else { 0 },
        ],
    )?;
    Ok(())
}

pub fn recalculate_review(
    conn: &Connection,
    date: &str,
    summary_override: Option<&str>,
) -> Result<DailyReview> {
    let totals = DailyDimensionTotals::from_pairs(&ledger_repo::get_totals_by_date(conn, date)?);
    let is_analyzed = ledger_repo::has_active_entries_for_date(conn, date)?;
    let existing = get_review(conn, date)?;
    let summary_text = if is_analyzed {
        summary_override
            .map(str::to_owned)
            .or_else(|| existing.as_ref().and_then(|review| review.summary_text.clone()))
    } else {
        None
    };

    upsert_review(conn, date, &totals, summary_text.as_deref(), is_analyzed)?;

    Ok(DailyReview {
        date: date.to_owned(),
        totals,
        summary_text,
        is_analyzed,
    })
}

pub fn get_flags_in_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<DailyReviewFlag>> {
    let mut stmt = conn.prepare(
        "SELECT date, is_analyzed
         FROM daily_reviews
         WHERE date BETWEEN ?1 AND ?2"
    )?;
    let flags = stmt.query_map(params![start_date, end_date], |row| {
        Ok(DailyReviewFlag {
            date: row.get(0)?,
            is_analyzed: row.get::<_, i32>(1)? != 0,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(flags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn recalculate_review_updates_totals_from_ledger() {
        let conn = Connection::open_in_memory().expect("open in memory db");
        run_migrations(&conn).expect("migrations");

        ledger_repo::insert_ledger(
            &conn,
            "2026-06-10",
            None,
            "knowledge",
            4,
            "完成线代作业",
            "学习",
            Some(0.9),
            "api",
        )
        .expect("insert knowledge ledger");
        ledger_repo::insert_ledger(
            &conn,
            "2026-06-10",
            None,
            "willpower",
            1,
            "完成线代作业",
            "学习",
            Some(0.9),
            "api",
        )
        .expect("insert willpower ledger");

        let review = recalculate_review(&conn, "2026-06-10", Some("今日以学习成长为主。"))
            .expect("recalculate review");

        assert_eq!(
            review.totals,
            DailyDimensionTotals {
                knowledge: 4,
                willpower: 1,
                expression: 0,
                physique: 0,
                bond: 0,
            }
        );
        assert!(review.is_analyzed);
        assert_eq!(review.summary_text.as_deref(), Some("今日以学习成长为主。"));
    }
}
