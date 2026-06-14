use crate::db::connection::DbState;
use crate::db::repositories::mind_repo;
use crate::models::mind::{BondEntry, BondPerson, DailyJournal};
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct BondPersonItem {
    pub id: i64,
    pub name: String,
    pub relation_label: String,
    pub score: i32,
    pub note: String,
    pub latest_entry_date: Option<String>,
    pub entry_count: i32,
}

#[derive(Serialize, Clone)]
pub struct BondEntryItem {
    pub id: i64,
    pub person_id: i64,
    pub entry_date: String,
    pub title: String,
    pub content: String,
}

#[derive(Serialize, Clone)]
pub struct DailyJournalItem {
    pub id: i64,
    pub entry_date: String,
    pub title: String,
    pub content: String,
    pub mood: String,
}

#[tauri::command]
pub fn get_bond_people(state: State<DbState>) -> Result<Vec<BondPersonItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let people = mind_repo::list_bond_people(&conn).map_err(|e| e.to_string())?;
    Ok(people.into_iter().map(map_bond_person).collect())
}

#[tauri::command]
pub fn save_bond_person(
    state: State<DbState>,
    person_id: Option<i64>,
    name: String,
    relation_label: Option<String>,
    score: i32,
    note: Option<String>,
) -> Result<BondPersonItem, String> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err("person name cannot be empty".into());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let person = mind_repo::save_bond_person(
        &conn,
        person_id,
        trimmed_name,
        relation_label.as_deref().unwrap_or(""),
        score,
        note.as_deref().unwrap_or(""),
    )
    .map_err(|e| e.to_string())?;
    Ok(map_bond_person(person))
}

#[tauri::command]
pub fn delete_bond_person(state: State<DbState>, person_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    mind_repo::delete_bond_person(&conn, person_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_bond_entries(state: State<DbState>, person_id: i64) -> Result<Vec<BondEntryItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let entries = mind_repo::list_bond_entries_by_person(&conn, person_id).map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(map_bond_entry).collect())
}

#[tauri::command]
pub fn save_bond_entry(
    state: State<DbState>,
    person_id: i64,
    entry_date: String,
    title: Option<String>,
    content: String,
) -> Result<BondEntryItem, String> {
    let trimmed_date = entry_date.trim();
    if trimmed_date.is_empty() {
        return Err("entry date cannot be empty".into());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let entry = mind_repo::save_bond_entry(
        &conn,
        person_id,
        trimmed_date,
        title.as_deref().unwrap_or(""),
        &content,
    )
    .map_err(|e| e.to_string())?;
    Ok(map_bond_entry(entry))
}

#[tauri::command]
pub fn delete_bond_entry(state: State<DbState>, entry_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    mind_repo::delete_bond_entry(&conn, entry_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_daily_journal_by_date(
    state: State<DbState>,
    entry_date: String,
) -> Result<Option<DailyJournalItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let journal = mind_repo::get_daily_journal_by_date(&conn, &entry_date).map_err(|e| e.to_string())?;
    Ok(journal.map(map_daily_journal))
}

#[tauri::command]
pub fn get_recent_daily_journals(
    state: State<DbState>,
    limit: Option<i64>,
) -> Result<Vec<DailyJournalItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let journals =
        mind_repo::list_recent_daily_journals(&conn, limit.unwrap_or(20)).map_err(|e| e.to_string())?;
    Ok(journals.into_iter().map(map_daily_journal).collect())
}

#[tauri::command]
pub fn save_daily_journal(
    state: State<DbState>,
    entry_date: String,
    title: Option<String>,
    content: String,
    mood: Option<String>,
) -> Result<DailyJournalItem, String> {
    let trimmed_date = entry_date.trim();
    if trimmed_date.is_empty() {
        return Err("entry date cannot be empty".into());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let journal = mind_repo::save_daily_journal(
        &conn,
        trimmed_date,
        title.as_deref().unwrap_or(""),
        &content,
        mood.as_deref().unwrap_or(""),
    )
    .map_err(|e| e.to_string())?;
    Ok(map_daily_journal(journal))
}

#[tauri::command]
pub fn delete_daily_journal(state: State<DbState>, journal_id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    mind_repo::delete_daily_journal(&conn, journal_id).map_err(|e| e.to_string())
}

fn map_bond_person(person: BondPerson) -> BondPersonItem {
    BondPersonItem {
        id: person.id,
        name: person.name,
        relation_label: person.relation_label,
        score: person.score,
        note: person.note,
        latest_entry_date: person.latest_entry_date,
        entry_count: person.entry_count,
    }
}

fn map_bond_entry(entry: BondEntry) -> BondEntryItem {
    BondEntryItem {
        id: entry.id,
        person_id: entry.person_id,
        entry_date: entry.entry_date,
        title: entry.title,
        content: entry.content,
    }
}

fn map_daily_journal(journal: DailyJournal) -> DailyJournalItem {
    DailyJournalItem {
        id: journal.id,
        entry_date: journal.entry_date,
        title: journal.title,
        content: journal.content,
        mood: journal.mood,
    }
}
