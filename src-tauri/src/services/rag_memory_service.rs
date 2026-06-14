use crate::db::repositories::personal_memory_repo;
use crate::models::personal_memory::PersonalProfile;
use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct RagMemoryManifest {
    pub schema_version: String,
    pub exported_at: String,
    pub memory_item_count: i32,
    pub active_memory_count: i32,
    pub patch_run_count: i32,
    pub profile_updated_at: Option<String>,
}

pub fn ensure_rag_memory_dir(rag_memory_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(rag_memory_dir.join("patch_runs")).map_err(|e| e.to_string())
}

pub fn rebuild_rag_memory_files(
    conn: &Connection,
    rag_memory_dir: &Path,
) -> Result<RagMemoryManifest, String> {
    ensure_rag_memory_dir(rag_memory_dir)?;

    let profile = personal_memory_repo::get_personal_profile(conn)?;
    write_profile_file(rag_memory_dir, &profile)?;

    let items = personal_memory_repo::search_personal_memory(conn, None, &[], 10_000)?;
    let memory_items_jsonl = items
        .iter()
        .map(|item| serde_json::to_string(item).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    fs::write(
        rag_memory_dir.join("memory_items.jsonl"),
        memory_items_jsonl,
    )
    .map_err(|e| e.to_string())?;

    let manifest = RagMemoryManifest {
        schema_version: "1.0".into(),
        exported_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        memory_item_count: items.len() as i32,
        active_memory_count: items
            .iter()
            .filter(|item| item.status == "active" || item.status == "pending")
            .count() as i32,
        patch_run_count: personal_memory_repo::list_patch_runs(conn)?.len() as i32,
        profile_updated_at: profile.updated_at.clone(),
    };

    fs::write(
        rag_memory_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(manifest)
}

pub fn export_rag_memory_snapshot(
    conn: &Connection,
    rag_memory_dir: &Path,
) -> Result<String, String> {
    let manifest = rebuild_rag_memory_files(conn, rag_memory_dir)?;
    let profile = personal_memory_repo::get_personal_profile(conn)?;
    let items = personal_memory_repo::search_personal_memory(conn, None, &[], 10_000)?;
    let patch_runs = personal_memory_repo::list_patch_runs(conn)?;

    #[derive(Serialize)]
    struct PatchRunExport {
        id: i64,
        source_context_id: String,
        patch_json: String,
        validation_status: String,
        apply_status: String,
        rejected_reason: Option<String>,
        applied_operations: i32,
        rejected_operations: i32,
        created_at: String,
    }

    #[derive(Serialize)]
    struct RagMemorySnapshot {
        profile: PersonalProfile,
        manifest: RagMemoryManifest,
        memory_items: Vec<crate::models::personal_memory::PersonalMemoryViewItem>,
        patch_runs: Vec<PatchRunExport>,
        directory: String,
    }

    let snapshot = RagMemorySnapshot {
        profile,
        manifest,
        memory_items: items,
        patch_runs: patch_runs
            .into_iter()
            .map(
                |(
                    id,
                    source_context_id,
                    patch_json,
                    validation_status,
                    apply_status,
                    rejected_reason,
                    applied_operations,
                    rejected_operations,
                    created_at,
                )| PatchRunExport {
                    id,
                    source_context_id,
                    patch_json,
                    validation_status,
                    apply_status,
                    rejected_reason,
                    applied_operations,
                    rejected_operations,
                    created_at,
                },
            )
            .collect(),
        directory: path_to_string(rag_memory_dir),
    };

    serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())
}

pub fn write_profile_file(rag_memory_dir: &Path, profile: &PersonalProfile) -> Result<(), String> {
    ensure_rag_memory_dir(rag_memory_dir)?;
    fs::write(
        rag_memory_dir.join("profile.json"),
        serde_json::to_string_pretty(profile).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

pub fn write_patch_run_file(
    rag_memory_dir: &Path,
    created_at: &str,
    patch_json: &str,
) -> Result<PathBuf, String> {
    ensure_rag_memory_dir(rag_memory_dir)?;
    let safe_name = created_at.replace([':', ' '], "-");
    let path = rag_memory_dir
        .join("patch_runs")
        .join(format!("{safe_name}.json"));
    fs::write(&path, patch_json).map_err(|e| e.to_string())?;
    Ok(path)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::personal_memory_repo::save_personal_profile;

    #[test]
    fn rebuild_writes_profile_and_memory_files() {
        let conn = Connection::open_in_memory().expect("open db");
        run_migrations(&conn).expect("migrate");
        save_personal_profile(
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

        let test_dir = std::env::temp_dir().join(format!(
            "pgrn-rag-memory-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));

        rebuild_rag_memory_files(&conn, &test_dir).expect("rebuild files");

        assert!(test_dir.join("profile.json").exists());
        assert!(test_dir.join("memory_items.jsonl").exists());
        assert!(test_dir.join("manifest.json").exists());

        let _ = fs::remove_dir_all(test_dir);
    }
}
