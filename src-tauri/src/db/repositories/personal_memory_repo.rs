use crate::models::personal_memory::{
    PersonalContextPack, PersonalMemoryItem, PersonalMemoryOverview, PersonalMemoryPatch,
    PersonalMemoryPatchApplyResult, PersonalMemoryPatchOperation, PersonalMemorySource,
    PersonalMemoryViewItem, PersonalProfile,
};
use chrono::{Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use std::collections::{HashMap, HashSet};

const PROFILE_ROW_ID: i64 = 1;

pub fn get_personal_profile(conn: &Connection) -> Result<PersonalProfile, String> {
    let profile = conn
        .query_row(
            "SELECT birthday, personality, experiences, personal_notes, updated_at
             FROM personal_profile
             WHERE id = ?1",
            params![PROFILE_ROW_ID],
            |row| {
                Ok(PersonalProfile {
                    birthday: row.get(0)?,
                    personality: row.get(1)?,
                    experiences: row.get(2)?,
                    personal_notes: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(profile.unwrap_or_default())
}

pub fn save_personal_profile(
    conn: &Connection,
    profile: &PersonalProfile,
) -> Result<PersonalProfile, String> {
    conn.execute(
        "INSERT INTO personal_profile (
            id,
            birthday,
            personality,
            experiences,
            personal_notes
         ) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            birthday = excluded.birthday,
            personality = excluded.personality,
            experiences = excluded.experiences,
            personal_notes = excluded.personal_notes,
            updated_at = datetime('now', 'localtime')",
        params![
            PROFILE_ROW_ID,
            profile.birthday,
            profile.personality,
            profile.experiences,
            profile.personal_notes
        ],
    )
    .map_err(|e| e.to_string())?;

    get_personal_profile(conn)
}

pub fn list_active_memory_items(conn: &Connection) -> Result<Vec<PersonalMemoryViewItem>, String> {
    let items = list_memory_items(conn)?;
    let sources = list_memory_sources(conn)?;
    Ok(items
        .into_iter()
        .filter(|item| item.status == "active" || item.status == "pending")
        .map(|item| to_view_item(item, &sources))
        .collect())
}

pub fn get_personal_memory_overview(conn: &Connection) -> Result<PersonalMemoryOverview, String> {
    let items = list_memory_items(conn)?;
    let sources = list_memory_sources(conn)?;
    let total_items = items.len() as i32;
    let active_items = items
        .iter()
        .filter(|item| item.status == "active")
        .count() as i32;
    let pending_items = items
        .iter()
        .filter(|item| item.status == "pending")
        .count() as i32;
    let rejected_items = items
        .iter()
        .filter(|item| item.status == "rejected")
        .count() as i32;

    let mut top_items = items
        .into_iter()
        .filter(|item| item.status == "active" || item.status == "pending")
        .map(|item| to_view_item(item, &sources))
        .collect::<Vec<_>>();
    top_items.sort_by(|a, b| {
        b.importance
            .cmp(&a.importance)
            .then_with(|| b.last_seen_date.cmp(&a.last_seen_date))
            .then_with(|| a.id.cmp(&b.id))
    });
    top_items.truncate(10);

    Ok(PersonalMemoryOverview {
        total_items,
        active_items,
        pending_items,
        rejected_items,
        top_items,
    })
}

pub fn search_personal_memory(
    conn: &Connection,
    query: Option<&str>,
    tags: &[String],
    limit: usize,
) -> Result<Vec<PersonalMemoryViewItem>, String> {
    let normalized_query = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase);
    let tag_filters = tags
        .iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();

    let items = list_memory_items(conn)?;
    let sources = list_memory_sources(conn)?;

    let mut results = items
        .into_iter()
        .map(|item| to_view_item(item, &sources))
        .filter(|item| {
            let query_matches = normalized_query.as_ref().is_none_or(|needle| {
                item.title.to_lowercase().contains(needle)
                    || item.summary.to_lowercase().contains(needle)
                    || item.detail.to_lowercase().contains(needle)
                    || item.tags.iter().any(|tag| tag.to_lowercase().contains(needle))
            });
            let tags_match = tag_filters.is_empty()
                || tag_filters.iter().all(|needle| {
                    item.tags
                        .iter()
                        .any(|tag| tag.to_lowercase() == *needle)
                });
            query_matches && tags_match
        })
        .collect::<Vec<_>>();

    results.sort_by(|a, b| {
        b.importance
            .cmp(&a.importance)
            .then_with(|| b.last_seen_date.cmp(&a.last_seen_date))
            .then_with(|| a.id.cmp(&b.id))
    });
    results.truncate(limit.max(1));
    Ok(results)
}

pub fn build_personal_context_pack(
    conn: &Connection,
    date: &str,
    mode: &str,
) -> Result<PersonalContextPack, String> {
    let profile = get_personal_profile(conn)?;
    let overview = get_personal_memory_overview(conn)?;
    let all_items = list_active_memory_items(conn)?;

    let mut high_priority_memories = all_items.clone();
    high_priority_memories.sort_by(|a, b| b.importance.cmp(&a.importance).then_with(|| b.last_seen_date.cmp(&a.last_seen_date)));
    high_priority_memories.truncate(20);

    let recent_threshold = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap_or_else(|_| Local::now().date_naive())
        - Duration::days(30);
    let mut recent_memories = all_items
        .iter()
        .filter(|item| {
            item.last_seen_date
                .as_deref()
                .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
                .map(|value| value >= recent_threshold)
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    recent_memories.sort_by(|a, b| b.last_seen_date.cmp(&a.last_seen_date).then_with(|| b.importance.cmp(&a.importance)));
    recent_memories.truncate(20);

    let relevant_types = ["habit", "recurring_pattern", "relationship", "goal_context", "caution"];
    let mut relevant_memories = all_items
        .iter()
        .filter(|item| relevant_types.contains(&item.memory_type.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    relevant_memories.sort_by(|a, b| b.importance.cmp(&a.importance).then_with(|| b.last_seen_date.cmp(&a.last_seen_date)));
    relevant_memories.truncate(20);

    Ok(PersonalContextPack {
        schema_version: "1.0".into(),
        date: date.to_string(),
        mode: mode.to_string(),
        generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        profile,
        overview,
        high_priority_memories,
        recent_memories,
        relevant_memories,
    })
}

pub fn apply_memory_patch(
    conn: &mut Connection,
    patch_json: &str,
    source_context_id: &str,
) -> Result<PersonalMemoryPatchApplyResult, String> {
    let patch_run_id = create_patch_run(conn, source_context_id, patch_json)?;
    let patch: PersonalMemoryPatch = serde_json::from_str(patch_json)
        .map_err(|e| {
            let _ = finalize_patch_run(
                conn,
                patch_run_id,
                "rejected",
                "rejected",
                0,
                1,
                Some(&format!("invalid patch json: {e}")),
            );
            format!("invalid patch json: {e}")
        })?;

    if patch.schema_version.trim() != "1.0" {
        let message = format!("unsupported patch schema version: {}", patch.schema_version);
        finalize_patch_run(conn, patch_run_id, "rejected", "rejected", 0, 1, Some(&message))?;
        return Err(message);
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut applied_operations = 0;
    let mut rejected_operations = 0;
    let mut rejection_messages = Vec::new();

    if let Some(profile_updates) = patch.profile_updates.clone() {
        match apply_profile_updates(&tx, &profile_updates) {
            Ok(changed) => {
                if changed {
                    applied_operations += 1;
                }
            }
            Err(message) => {
                rejected_operations += 1;
                rejection_messages.push(message);
            }
        }
    }

    for operation in &patch.memory_operations {
        match apply_memory_operation(&tx, operation) {
            Ok(was_applied) => {
                if was_applied {
                    applied_operations += 1;
                } else {
                    rejected_operations += 1;
                }
            }
            Err(message) => {
                rejected_operations += 1;
                rejection_messages.push(message);
            }
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    let message = if rejection_messages.is_empty() {
        "patch applied".to_string()
    } else {
        rejection_messages.join("; ")
    };
    let validation_status = if rejected_operations == 0 {
        "applied"
    } else if applied_operations > 0 {
        "partial"
    } else {
        "rejected"
    };

    finalize_patch_run(
        conn,
        patch_run_id,
        validation_status,
        validation_status,
        applied_operations,
        rejected_operations,
        if rejection_messages.is_empty() {
            None
        } else {
            Some(&message)
        },
    )?;

    if rejected_operations > 0 && applied_operations == 0 {
        return Err(message);
    }

    Ok(PersonalMemoryPatchApplyResult {
        patch_run_id,
        validation_status: validation_status.to_string(),
        apply_status: validation_status.to_string(),
        applied_operations,
        rejected_operations,
        message,
    })
}

fn apply_profile_updates(
    conn: &Connection,
    updates: &crate::models::personal_memory::PersonalProfilePatch,
) -> Result<bool, String> {
    if updates.birthday.as_deref().is_some_and(|value| !value.trim().is_empty()) {
        return Err("AI patch cannot overwrite birthday".into());
    }

    let mut profile = get_personal_profile(conn)?;
    let before = serde_json::to_string(&profile).map_err(|e| e.to_string())?;

    profile.personality = merge_profile_text(&profile.personality, updates.personality.as_deref());
    profile.experiences = merge_profile_text(&profile.experiences, updates.experiences.as_deref());
    profile.personal_notes = merge_profile_text(&profile.personal_notes, updates.personal_notes.as_deref());

    let after = serde_json::to_string(&profile).map_err(|e| e.to_string())?;
    if before == after {
        return Ok(false);
    }

    save_personal_profile(conn, &profile)?;
    log_memory_event(
        conn,
        None,
        "profile_update",
        &json!({
            "personality": updates.personality,
            "experiences": updates.experiences,
            "personal_notes": updates.personal_notes,
        })
        .to_string(),
    )?;
    Ok(true)
}

fn apply_memory_operation(
    conn: &Connection,
    operation: &PersonalMemoryPatchOperation,
) -> Result<bool, String> {
    validate_operation(operation)?;

    match operation.op.as_str() {
        "create" => apply_create_operation(conn, operation),
        "update" => apply_update_operation(conn, operation),
        "reinforce" => apply_reinforce_operation(conn, operation),
        "supersede" => apply_supersede_operation(conn, operation),
        "reject" => apply_reject_operation(conn, operation),
        other => Err(format!("unsupported memory operation: {other}")),
    }
}

fn apply_create_operation(
    conn: &Connection,
    operation: &PersonalMemoryPatchOperation,
) -> Result<bool, String> {
    let title = required_field(operation.title.as_deref(), "title")?;
    let memory_type = required_field(operation.memory_type.as_deref(), "memory_type")?;
    let summary = required_field(operation.summary.as_deref(), "summary")?;
    let normalized_title = title.trim().to_lowercase();

    if let Some(existing) = find_active_memory_by_title(conn, &normalized_title, memory_type)? {
        attach_sources(conn, existing.id, &operation.sources)?;
        let confidence = operation.confidence.unwrap_or(existing.confidence).max(existing.confidence);
        let importance = operation.importance.unwrap_or(existing.importance).max(existing.importance);
        let last_seen_date = operation
            .last_seen_date
            .as_deref()
            .or(operation.first_seen_date.as_deref())
            .map(str::to_string)
            .or(existing.last_seen_date.clone());

        update_memory_row(
            conn,
            existing.id,
            memory_type,
            title,
            summary,
            operation.detail.as_deref().unwrap_or(&existing.detail),
            &operation.tags,
            importance,
            confidence,
            existing.first_seen_date.as_deref(),
            last_seen_date.as_deref(),
            if confidence < 0.35 { "pending" } else { "active" },
            existing.supersedes_id,
            &existing.created_by,
        )?;
        log_memory_event(
            conn,
            Some(existing.id),
            "reinforce_auto_merge",
            &json!({
                "reason": operation.reason,
                "evidence_ids": operation.evidence_ids,
            })
            .to_string(),
        )?;
        return Ok(true);
    }

    let status = if operation.confidence.unwrap_or(0.0) < 0.35 {
        "pending"
    } else {
        "active"
    };
    let memory_id = insert_memory_row(
        conn,
        memory_type,
        title,
        summary,
        operation.detail.as_deref().unwrap_or(""),
        &operation.tags,
        operation.importance.unwrap_or(50).clamp(0, 100),
        operation.confidence.unwrap_or(0.0).clamp(0.0, 1.0),
        operation.first_seen_date.as_deref(),
        operation
            .last_seen_date
            .as_deref()
            .or(operation.first_seen_date.as_deref()),
        status,
        None,
        "ai",
    )?;
    attach_sources(conn, memory_id, &operation.sources)?;
    log_memory_event(
        conn,
        Some(memory_id),
        "create",
        &json!({
            "reason": operation.reason,
            "evidence_ids": operation.evidence_ids,
        })
        .to_string(),
    )?;
    Ok(true)
}

fn apply_update_operation(
    conn: &Connection,
    operation: &PersonalMemoryPatchOperation,
) -> Result<bool, String> {
    let target_id = required_target_id(operation)?;
    let existing = get_memory_item_by_id(conn, target_id)?
        .ok_or_else(|| format!("target memory item not found: {target_id}"))?;
    let existing_tags = parse_tags(&existing.tags_json);

    attach_sources(conn, target_id, &operation.sources)?;
    update_memory_row(
        conn,
        target_id,
        operation
            .memory_type
            .as_deref()
            .unwrap_or(&existing.memory_type),
        operation.title.as_deref().unwrap_or(&existing.title),
        operation.summary.as_deref().unwrap_or(&existing.summary),
        operation.detail.as_deref().unwrap_or(&existing.detail),
        if operation.tags.is_empty() { &existing_tags } else { &operation.tags },
        operation.importance.unwrap_or(existing.importance).clamp(0, 100),
        operation
            .confidence
            .unwrap_or(existing.confidence)
            .clamp(0.0, 1.0),
        operation
            .first_seen_date
            .as_deref()
            .or(existing.first_seen_date.as_deref()),
        operation
            .last_seen_date
            .as_deref()
            .or(existing.last_seen_date.as_deref()),
        if operation.confidence.unwrap_or(existing.confidence) < 0.35 {
            "pending"
        } else {
            &existing.status
        },
        existing.supersedes_id,
        &existing.created_by,
    )?;
    log_memory_event(
        conn,
        Some(target_id),
        "update",
        &json!({
            "reason": operation.reason,
            "evidence_ids": operation.evidence_ids,
        })
        .to_string(),
    )?;
    Ok(true)
}

fn apply_reinforce_operation(
    conn: &Connection,
    operation: &PersonalMemoryPatchOperation,
) -> Result<bool, String> {
    let target_id = required_target_id(operation)?;
    let existing = get_memory_item_by_id(conn, target_id)?
        .ok_or_else(|| format!("target memory item not found: {target_id}"))?;
    let existing_tags = parse_tags(&existing.tags_json);

    attach_sources(conn, target_id, &operation.sources)?;
    let importance = operation.importance.unwrap_or(existing.importance).max(existing.importance);
    let confidence = operation.confidence.unwrap_or(existing.confidence).max(existing.confidence);
    let status = if confidence < 0.35 { "pending" } else { "active" };

    update_memory_row(
        conn,
        target_id,
        &existing.memory_type,
        &existing.title,
        operation.summary.as_deref().unwrap_or(&existing.summary),
        operation.detail.as_deref().unwrap_or(&existing.detail),
        if operation.tags.is_empty() { &existing_tags } else { &operation.tags },
        importance,
        confidence,
        existing.first_seen_date.as_deref(),
        operation
            .last_seen_date
            .as_deref()
            .or(operation.first_seen_date.as_deref())
            .or(existing.last_seen_date.as_deref()),
        status,
        existing.supersedes_id,
        &existing.created_by,
    )?;
    log_memory_event(
        conn,
        Some(target_id),
        "reinforce",
        &json!({
            "reason": operation.reason,
            "evidence_ids": operation.evidence_ids,
        })
        .to_string(),
    )?;
    Ok(true)
}

fn apply_supersede_operation(
    conn: &Connection,
    operation: &PersonalMemoryPatchOperation,
) -> Result<bool, String> {
    let target_id = required_target_id(operation)?;
    let existing = get_memory_item_by_id(conn, target_id)?
        .ok_or_else(|| format!("target memory item not found: {target_id}"))?;
    let existing_tags = parse_tags(&existing.tags_json);
    let memory_type = operation
        .memory_type
        .as_deref()
        .unwrap_or(&existing.memory_type);
    let title = operation.title.as_deref().unwrap_or(&existing.title);
    let summary = operation.summary.as_deref().unwrap_or(&existing.summary);

    let new_memory_id = insert_memory_row(
        conn,
        memory_type,
        title,
        summary,
        operation.detail.as_deref().unwrap_or(&existing.detail),
        if operation.tags.is_empty() { &existing_tags } else { &operation.tags },
        operation.importance.unwrap_or(existing.importance).clamp(0, 100),
        operation
            .confidence
            .unwrap_or(existing.confidence)
            .clamp(0.0, 1.0),
        operation
            .first_seen_date
            .as_deref()
            .or(existing.first_seen_date.as_deref()),
        operation
            .last_seen_date
            .as_deref()
            .or(existing.last_seen_date.as_deref()),
        if operation.confidence.unwrap_or(existing.confidence) < 0.35 {
            "pending"
        } else {
            "active"
        },
        Some(target_id),
        "ai",
    )?;
    attach_sources(conn, new_memory_id, &operation.sources)?;
    set_memory_status(conn, target_id, "superseded")?;
    log_memory_event(
        conn,
        Some(target_id),
        "supersede_old",
        &json!({
            "new_memory_id": new_memory_id,
            "reason": operation.reason,
        })
        .to_string(),
    )?;
    log_memory_event(
        conn,
        Some(new_memory_id),
        "supersede_new",
        &json!({
            "supersedes_id": target_id,
            "reason": operation.reason,
            "evidence_ids": operation.evidence_ids,
        })
        .to_string(),
    )?;
    Ok(true)
}

fn apply_reject_operation(
    conn: &Connection,
    operation: &PersonalMemoryPatchOperation,
) -> Result<bool, String> {
    let target_id = required_target_id(operation)?;
    get_memory_item_by_id(conn, target_id)?
        .ok_or_else(|| format!("target memory item not found: {target_id}"))?;
    set_memory_status(conn, target_id, "rejected")?;
    log_memory_event(
        conn,
        Some(target_id),
        "reject",
        &json!({
            "reason": operation.reason,
            "evidence_ids": operation.evidence_ids,
        })
        .to_string(),
    )?;
    Ok(true)
}

fn validate_operation(operation: &PersonalMemoryPatchOperation) -> Result<(), String> {
    if operation.evidence_ids.is_empty() {
        return Err("memory operation requires at least one evidence id".into());
    }
    if operation.sources.is_empty() {
        return Err("memory operation requires at least one source".into());
    }
    let evidence_ids = operation.evidence_ids.iter().collect::<HashSet<_>>();
    let source_evidence_ids = operation
        .sources
        .iter()
        .map(|source| &source.evidence_id)
        .collect::<HashSet<_>>();
    if !evidence_ids.is_subset(&source_evidence_ids) {
        return Err("evidence ids must be backed by sources".into());
    }
    if let Some(confidence) = operation.confidence {
        if !(0.0..=1.0).contains(&confidence) {
            return Err("confidence must be between 0 and 1".into());
        }
    }
    Ok(())
}

fn required_field<'a>(value: Option<&'a str>, field_name: &str) -> Result<&'a str, String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("memory operation requires {field_name}"))
}

fn required_target_id(operation: &PersonalMemoryPatchOperation) -> Result<i64, String> {
    operation
        .target_id
        .ok_or_else(|| "memory operation requires target_id".to_string())
}

fn merge_profile_text(existing: &str, incoming: Option<&str>) -> String {
    let Some(incoming_text) = incoming.map(str::trim).filter(|value| !value.is_empty()) else {
        return existing.to_string();
    };
    if existing.trim().is_empty() {
        return incoming_text.to_string();
    }
    if existing.contains(incoming_text) {
        return existing.to_string();
    }
    format!("{existing}\n\nAI supplement: {incoming_text}")
}

fn list_memory_items(conn: &Connection) -> Result<Vec<PersonalMemoryItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                memory_type,
                title,
                summary,
                detail,
                tags_json,
                importance,
                confidence,
                first_seen_date,
                last_seen_date,
                status,
                supersedes_id,
                created_by
             FROM personal_memory_items
             ORDER BY updated_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([], |row| {
            Ok(PersonalMemoryItem {
                id: row.get(0)?,
                memory_type: row.get(1)?,
                title: row.get(2)?,
                summary: row.get(3)?,
                detail: row.get(4)?,
                tags_json: row.get(5)?,
                importance: row.get(6)?,
                confidence: row.get(7)?,
                first_seen_date: row.get(8)?,
                last_seen_date: row.get(9)?,
                status: row.get(10)?,
                supersedes_id: row.get(11)?,
                created_by: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

fn list_memory_sources(conn: &Connection) -> Result<Vec<PersonalMemorySource>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                memory_id,
                source_type,
                source_id,
                source_date,
                evidence_id,
                excerpt
             FROM personal_memory_sources
             ORDER BY id",
        )
        .map_err(|e| e.to_string())?;

    let sources = stmt
        .query_map([], |row| {
            Ok(PersonalMemorySource {
                id: row.get(0)?,
                memory_id: row.get(1)?,
                source_type: row.get(2)?,
                source_id: row.get(3)?,
                source_date: row.get(4)?,
                evidence_id: row.get(5)?,
                excerpt: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(sources)
}

pub fn list_patch_runs(
    conn: &Connection,
) -> Result<Vec<(i64, String, String, String, String, Option<String>, i32, i32, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                id,
                source_context_id,
                patch_json,
                validation_status,
                apply_status,
                rejected_reason,
                applied_operations,
                rejected_operations,
                created_at
             FROM personal_memory_patch_runs
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn get_memory_item_by_id(conn: &Connection, memory_id: i64) -> Result<Option<PersonalMemoryItem>, String> {
    conn.query_row(
        "SELECT
            id,
            memory_type,
            title,
            summary,
            detail,
            tags_json,
            importance,
            confidence,
            first_seen_date,
            last_seen_date,
            status,
            supersedes_id,
            created_by
         FROM personal_memory_items
         WHERE id = ?1",
        params![memory_id],
        |row| {
            Ok(PersonalMemoryItem {
                id: row.get(0)?,
                memory_type: row.get(1)?,
                title: row.get(2)?,
                summary: row.get(3)?,
                detail: row.get(4)?,
                tags_json: row.get(5)?,
                importance: row.get(6)?,
                confidence: row.get(7)?,
                first_seen_date: row.get(8)?,
                last_seen_date: row.get(9)?,
                status: row.get(10)?,
                supersedes_id: row.get(11)?,
                created_by: row.get(12)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn find_active_memory_by_title(
    conn: &Connection,
    normalized_title: &str,
    memory_type: &str,
) -> Result<Option<PersonalMemoryItem>, String> {
    let items = list_memory_items(conn)?;
    Ok(items.into_iter().find(|item| {
        (item.status == "active" || item.status == "pending")
            && item.memory_type == memory_type
            && item.title.trim().eq_ignore_ascii_case(normalized_title)
    }))
}

fn insert_memory_row(
    conn: &Connection,
    memory_type: &str,
    title: &str,
    summary: &str,
    detail: &str,
    tags: &[String],
    importance: i32,
    confidence: f64,
    first_seen_date: Option<&str>,
    last_seen_date: Option<&str>,
    status: &str,
    supersedes_id: Option<i64>,
    created_by: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO personal_memory_items (
            memory_type,
            title,
            summary,
            detail,
            tags_json,
            importance,
            confidence,
            first_seen_date,
            last_seen_date,
            status,
            supersedes_id,
            created_by
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            memory_type,
            title,
            summary,
            detail,
            serialize_tags(tags),
            importance.clamp(0, 100),
            confidence.clamp(0.0, 1.0),
            first_seen_date,
            last_seen_date,
            status,
            supersedes_id,
            created_by
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
fn update_memory_row(
    conn: &Connection,
    memory_id: i64,
    memory_type: &str,
    title: &str,
    summary: &str,
    detail: &str,
    tags: &[String],
    importance: i32,
    confidence: f64,
    first_seen_date: Option<&str>,
    last_seen_date: Option<&str>,
    status: &str,
    supersedes_id: Option<i64>,
    created_by: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE personal_memory_items
         SET memory_type = ?2,
             title = ?3,
             summary = ?4,
             detail = ?5,
             tags_json = ?6,
             importance = ?7,
             confidence = ?8,
             first_seen_date = ?9,
             last_seen_date = ?10,
             status = ?11,
             supersedes_id = ?12,
             created_by = ?13,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![
            memory_id,
            memory_type,
            title,
            summary,
            detail,
            serialize_tags(tags),
            importance.clamp(0, 100),
            confidence.clamp(0.0, 1.0),
            first_seen_date,
            last_seen_date,
            status,
            supersedes_id,
            created_by,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn attach_sources(
    conn: &Connection,
    memory_id: i64,
    sources: &[crate::models::personal_memory::PersonalMemoryPatchSource],
) -> Result<(), String> {
    for source in sources {
        conn.execute(
            "INSERT INTO personal_memory_sources (
                memory_id,
                source_type,
                source_id,
                source_date,
                evidence_id,
                excerpt
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(memory_id, evidence_id) DO UPDATE SET
                source_type = excluded.source_type,
                source_id = excluded.source_id,
                source_date = excluded.source_date,
                excerpt = excluded.excerpt",
            params![
                memory_id,
                source.source_type,
                source.source_id,
                source.source_date,
                source.evidence_id,
                source.excerpt.as_deref().unwrap_or(""),
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn set_memory_status(conn: &Connection, memory_id: i64, status: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE personal_memory_items
         SET status = ?2,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![memory_id, status],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn create_patch_run(conn: &Connection, source_context_id: &str, patch_json: &str) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO personal_memory_patch_runs (
            source_context_id,
            patch_json
         ) VALUES (?1, ?2)",
        params![source_context_id, patch_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

fn finalize_patch_run(
    conn: &Connection,
    patch_run_id: i64,
    validation_status: &str,
    apply_status: &str,
    applied_operations: i32,
    rejected_operations: i32,
    rejected_reason: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE personal_memory_patch_runs
         SET validation_status = ?2,
             apply_status = ?3,
             applied_operations = ?4,
             rejected_operations = ?5,
             rejected_reason = ?6,
             updated_at = datetime('now', 'localtime')
         WHERE id = ?1",
        params![
            patch_run_id,
            validation_status,
            apply_status,
            applied_operations,
            rejected_operations,
            rejected_reason,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn log_memory_event(
    conn: &Connection,
    memory_id: Option<i64>,
    event_type: &str,
    payload_json: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO personal_memory_events (memory_id, event_type, payload_json)
         VALUES (?1, ?2, ?3)",
        params![memory_id, event_type, payload_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn serialize_tags(tags: &[String]) -> String {
    serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
}

fn parse_tags(tags_json: &str) -> Vec<String> {
    serde_json::from_str(tags_json).unwrap_or_default()
}

fn to_view_item(
    item: PersonalMemoryItem,
    sources: &[PersonalMemorySource],
) -> PersonalMemoryViewItem {
    let item_sources = sources
        .iter()
        .filter(|source| source.memory_id == item.id)
        .collect::<Vec<_>>();
    let mut evidence_ids = item_sources
        .iter()
        .map(|source| source.evidence_id.clone())
        .collect::<Vec<_>>();
    evidence_ids.sort();
    evidence_ids.dedup();

    PersonalMemoryViewItem {
        id: item.id,
        memory_type: item.memory_type,
        title: item.title,
        summary: item.summary,
        detail: item.detail,
        tags: parse_tags(&item.tags_json),
        importance: item.importance,
        confidence: item.confidence,
        first_seen_date: item.first_seen_date,
        last_seen_date: item.last_seen_date,
        status: item.status,
        supersedes_id: item.supersedes_id,
        created_by: item.created_by,
        source_count: item_sources.len() as i32,
        evidence_ids,
    }
}

pub fn get_memory_sources_grouped(
    conn: &Connection,
) -> Result<HashMap<i64, Vec<PersonalMemorySource>>, String> {
    let mut grouped: HashMap<i64, Vec<PersonalMemorySource>> = HashMap::new();
    for source in list_memory_sources(conn)? {
        grouped.entry(source.memory_id).or_default().push(source);
    }
    Ok(grouped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    #[test]
    fn saves_and_reads_personal_profile() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        let saved = save_personal_profile(
            &conn,
            &PersonalProfile {
                birthday: "1998-01-02".into(),
                personality: "calm".into(),
                experiences: "studied design".into(),
                personal_notes: "likes routine".into(),
                updated_at: None,
            },
        )
        .expect("save profile");
        let loaded = get_personal_profile(&conn).expect("load profile");

        assert_eq!(saved.birthday, "1998-01-02");
        assert_eq!(loaded.personality, "calm");
        assert_eq!(loaded.personal_notes, "likes routine");
    }

    #[test]
    fn rejects_patch_operation_without_evidence_ids() {
        let mut conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");

        let patch_json = r#"{
          "schema_version": "1.0",
          "profile_updates": null,
          "memory_operations": [
            {
              "op": "create",
              "memory_type": "habit",
              "title": "Morning study",
              "summary": "Usually studies in the morning",
              "detail": "",
              "tags": ["study"],
              "importance": 75,
              "confidence": 0.8,
              "reason": "Repeated pattern",
              "evidence_ids": [],
              "sources": []
            }
          ]
        }"#;

        let error = apply_memory_patch(&mut conn, patch_json, "test-context").expect_err("reject patch");
        assert!(error.contains("evidence"));
    }
}
