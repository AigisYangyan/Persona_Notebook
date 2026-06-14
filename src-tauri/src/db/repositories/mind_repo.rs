use crate::models::mind::{BondEntry, BondPerson, DailyJournal};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub fn list_bond_people(conn: &Connection) -> Result<Vec<BondPerson>> {
    let mut stmt = conn.prepare(
        "SELECT
            p.id,
            p.name,
            p.relation_label,
            p.score,
            p.note,
            MAX(e.entry_date) AS latest_entry_date,
            COUNT(e.id) AS entry_count
         FROM bond_people p
         LEFT JOIN bond_entries e ON e.person_id = p.id
         GROUP BY p.id, p.name, p.relation_label, p.score, p.note, p.updated_at
         ORDER BY latest_entry_date DESC, p.updated_at DESC, p.id DESC",
    )?;

    let people = stmt
        .query_map([], map_bond_person_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(people)
}

pub fn save_bond_person(
    conn: &Connection,
    person_id: Option<i64>,
    name: &str,
    relation_label: &str,
    score: i32,
    note: &str,
) -> Result<BondPerson> {
    let safe_score = score.clamp(0, 10);
    match person_id {
        Some(id) => {
            conn.execute(
                "UPDATE bond_people
                 SET name = ?2,
                     relation_label = ?3,
                     score = ?4,
                     note = ?5,
                     updated_at = datetime('now', 'localtime')
                 WHERE id = ?1",
                params![id, name, relation_label, safe_score, note],
            )?;
            get_bond_person_by_id(conn, id)?.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
        }
        None => {
            conn.execute(
                "INSERT INTO bond_people (name, relation_label, score, note)
                 VALUES (?1, ?2, ?3, ?4)",
                params![name, relation_label, safe_score, note],
            )?;
            get_bond_person_by_id(conn, conn.last_insert_rowid())?
                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
        }
    }
}

pub fn delete_bond_person(conn: &Connection, person_id: i64) -> Result<()> {
    conn.execute("DELETE FROM bond_people WHERE id = ?1", params![person_id])?;
    Ok(())
}

pub fn list_bond_entries_by_person(conn: &Connection, person_id: i64) -> Result<Vec<BondEntry>> {
    let mut stmt = conn.prepare(
        "SELECT
            id,
            person_id,
            entry_date,
            title,
            content
         FROM bond_entries
         WHERE person_id = ?1
         ORDER BY entry_date DESC, id DESC",
    )?;

    let entries = stmt
        .query_map(params![person_id], map_bond_entry_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn save_bond_entry(
    conn: &Connection,
    person_id: i64,
    entry_date: &str,
    title: &str,
    content: &str,
) -> Result<BondEntry> {
    conn.execute(
        "INSERT INTO bond_entries (person_id, entry_date, title, content)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(person_id, entry_date) DO UPDATE SET
            title = excluded.title,
            content = excluded.content,
            updated_at = datetime('now', 'localtime')",
        params![person_id, entry_date, title, content],
    )?;
    conn.execute(
        "UPDATE bond_people
         SET updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![person_id],
    )?;

    get_bond_entry_by_person_and_date(conn, person_id, entry_date)?
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

pub fn delete_bond_entry(conn: &Connection, entry_id: i64) -> Result<()> {
    conn.execute("DELETE FROM bond_entries WHERE id = ?1", params![entry_id])?;
    Ok(())
}

pub fn get_daily_journal_by_date(
    conn: &Connection,
    entry_date: &str,
) -> Result<Option<DailyJournal>> {
    conn.query_row(
        "SELECT
            id,
            entry_date,
            title,
            content,
            mood
         FROM daily_journals
         WHERE entry_date = ?1",
        params![entry_date],
        map_daily_journal_row,
    )
    .optional()
}

pub fn list_recent_daily_journals(conn: &Connection, limit: i64) -> Result<Vec<DailyJournal>> {
    let mut stmt = conn.prepare(
        "SELECT
            id,
            entry_date,
            title,
            content,
            mood
         FROM daily_journals
         ORDER BY entry_date DESC, updated_at DESC
         LIMIT ?1",
    )?;

    let journals = stmt
        .query_map(params![limit.max(1)], map_daily_journal_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(journals)
}

pub fn save_daily_journal(
    conn: &Connection,
    entry_date: &str,
    title: &str,
    content: &str,
    mood: &str,
) -> Result<DailyJournal> {
    conn.execute(
        "INSERT INTO daily_journals (entry_date, title, content, mood)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(entry_date) DO UPDATE SET
            title = excluded.title,
            content = excluded.content,
            mood = excluded.mood,
            updated_at = datetime('now', 'localtime')",
        params![entry_date, title, content, mood],
    )?;

    get_daily_journal_by_date(conn, entry_date)?.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

pub fn delete_daily_journal(conn: &Connection, journal_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM daily_journals WHERE id = ?1",
        params![journal_id],
    )?;
    Ok(())
}

fn get_bond_person_by_id(conn: &Connection, person_id: i64) -> Result<Option<BondPerson>> {
    conn.query_row(
        "SELECT
            p.id,
            p.name,
            p.relation_label,
            p.score,
            p.note,
            MAX(e.entry_date) AS latest_entry_date,
            COUNT(e.id) AS entry_count
         FROM bond_people p
         LEFT JOIN bond_entries e ON e.person_id = p.id
         WHERE p.id = ?1
         GROUP BY p.id, p.name, p.relation_label, p.score, p.note",
        params![person_id],
        map_bond_person_row,
    )
    .optional()
}

fn get_bond_entry_by_person_and_date(
    conn: &Connection,
    person_id: i64,
    entry_date: &str,
) -> Result<Option<BondEntry>> {
    conn.query_row(
        "SELECT
            id,
            person_id,
            entry_date,
            title,
            content
         FROM bond_entries
         WHERE person_id = ?1 AND entry_date = ?2",
        params![person_id, entry_date],
        map_bond_entry_row,
    )
    .optional()
}

fn map_bond_person_row(row: &rusqlite::Row<'_>) -> Result<BondPerson> {
    Ok(BondPerson {
        id: row.get(0)?,
        name: row.get(1)?,
        relation_label: row.get(2)?,
        score: row.get(3)?,
        note: row.get(4)?,
        latest_entry_date: row.get(5)?,
        entry_count: row.get(6)?,
    })
}

fn map_bond_entry_row(row: &rusqlite::Row<'_>) -> Result<BondEntry> {
    Ok(BondEntry {
        id: row.get(0)?,
        person_id: row.get(1)?,
        entry_date: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
    })
}

fn map_daily_journal_row(row: &rusqlite::Row<'_>) -> Result<DailyJournal> {
    Ok(DailyJournal {
        id: row.get(0)?,
        entry_date: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        mood: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn save_bond_entry_upserts_by_person_and_date() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        let person = save_bond_person(&conn, None, "Alice", "friend", 7, "").expect("person");
        save_bond_entry(&conn, person.id, "2026-06-13", "First", "A").expect("first entry");
        let second =
            save_bond_entry(&conn, person.id, "2026-06-13", "Updated", "B").expect("second entry");
        let entries = list_bond_entries_by_person(&conn, person.id).expect("entries");

        assert_eq!(entries.len(), 1);
        assert_eq!(second.title, "Updated");
        assert_eq!(entries[0].content, "B");
    }

    #[test]
    fn delete_bond_person_cascades_entries() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        let person = save_bond_person(&conn, None, "Alice", "friend", 7, "").expect("person");
        save_bond_entry(&conn, person.id, "2026-06-13", "First", "A").expect("entry");
        delete_bond_person(&conn, person.id).expect("delete person");

        let people = list_bond_people(&conn).expect("people");
        assert!(people.is_empty());
    }

    #[test]
    fn save_daily_journal_upserts_by_date() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        save_daily_journal(&conn, "2026-06-13", "One", "alpha", "calm").expect("first");
        let second =
            save_daily_journal(&conn, "2026-06-13", "Two", "beta", "focused").expect("second");
        let fetched = get_daily_journal_by_date(&conn, "2026-06-13")
            .expect("fetch")
            .expect("journal");

        assert_eq!(second.id, fetched.id);
        assert_eq!(fetched.title, "Two");
        assert_eq!(fetched.mood, "focused");
    }
}
