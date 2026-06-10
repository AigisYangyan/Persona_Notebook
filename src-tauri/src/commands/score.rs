use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::connection::DbState;
use crate::db::repositories::{daily_review_repo, dimension_repo, ledger_repo, rule_repo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ScorePreviewChange {
    pub dimension_key: String,
    pub change_value: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ScorePreviewItem {
    pub record_id: Option<i64>,
    pub title: String,
    pub category: String,
    pub changes: Vec<ScorePreviewChange>,
    pub difficulty_star: i32,
    pub confidence: f64,
    pub reason: String,
    pub engine: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScoreRecordInput {
    pub id: i64,
    pub title: String,
    pub minutes: i32,
    pub difficulty_star: i32,
}

#[tauri::command]
pub fn preview_score_with_local_rules(
    state: State<DbState>,
    records: Vec<ScoreRecordInput>,
) -> Result<Vec<ScorePreviewItem>, String> {
    if records.is_empty() {
        return Err("没有可分析的记录".into());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let rules = rule_repo::get_active_rules(&conn).map_err(|e| e.to_string())?;

    let raw_items = records
        .into_iter()
        .map(|record| {
            let title_lower = record.title.to_lowercase();
            let matched_rule = rules.iter().find(|rule| {
                rule.keywords
                    .split(',')
                    .map(str::trim)
                    .filter(|keyword| !keyword.is_empty())
                    .any(|keyword| title_lower.contains(&keyword.to_lowercase()))
            });

            let base_score = get_time_base_score(record.minutes);
            let multiplier = get_difficulty_multiplier(record.difficulty_star);
            let final_score = (base_score as f64 * multiplier).round() as i32;

            let (category, changes, confidence, reason) = if let Some(rule) = matched_rule {
                let changes = if let Some(secondary_dim) = &rule.secondary_dim {
                    let primary_score = ((final_score as f64) * 0.7).round() as i32;
                    let secondary_score = final_score - primary_score;
                    let mut values = Vec::new();
                    if primary_score > 0 {
                        values.push(ScorePreviewChange {
                            dimension_key: rule.primary_dim.clone(),
                            change_value: primary_score,
                        });
                    }
                    if secondary_score > 0 {
                        values.push(ScorePreviewChange {
                            dimension_key: secondary_dim.clone(),
                            change_value: secondary_score,
                        });
                    }
                    values
                } else {
                    vec![ScorePreviewChange {
                        dimension_key: rule.primary_dim.clone(),
                        change_value: final_score.max(0),
                    }]
                };

                (
                    format!("规则缓存: {}", rule.primary_dim),
                    changes,
                    0.82,
                    "命中规则缓存，已按时长和难度生成约束建议，等待 API 复核。".to_string(),
                )
            } else {
                (
                    "规则缓存: fallback".to_string(),
                    vec![ScorePreviewChange {
                        dimension_key: "willpower".into(),
                        change_value: 1,
                    }],
                    0.35,
                    "未命中规则缓存，先给出保守建议，等待 API 结合语义复核。".to_string(),
                )
            };

            ScorePreviewItem {
                record_id: Some(record.id),
                title: record.title,
                category,
                changes,
                difficulty_star: record.difficulty_star,
                confidence,
                reason,
                engine: "rules".into(),
            }
        })
        .collect::<Vec<_>>();

    Ok(raw_items)
}

#[tauri::command]
pub fn confirm_score_preview(
    state: State<DbState>,
    date: String,
    items: Vec<ScorePreviewItem>,
    summary: Option<String>,
) -> Result<(), String> {
    if date.trim().is_empty() {
        return Err("日期不能为空".into());
    }
    if items.is_empty() {
        return Err("没有可确认的评分预览".into());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    if ledger_repo::has_active_entries_for_date(&conn, &date).map_err(|e| e.to_string())? {
        return Err("该日期已经确认评分，请先回滚后再重新分析".into());
    }

    let dimensions = dimension_repo::get_all_dimensions(&conn).map_err(|e| e.to_string())?;
    let normalized_items = apply_daily_caps(items, &dimensions)?;

    if !has_positive_changes(&normalized_items) {
        return Err("当前预览没有可写入的成长变化".into());
    }

    for item in &normalized_items {
        for change in &item.changes {
            if change.change_value <= 0 {
                continue;
            }
            ledger_repo::insert_ledger(
                &conn,
                &date,
                item.record_id,
                &change.dimension_key,
                change.change_value,
                &item.title,
                &item.reason,
                Some(item.confidence),
                &item.engine,
            )
            .map_err(|e| e.to_string())?;
        }
    }

    daily_review_repo::recalculate_review(&conn, &date, summary.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn has_positive_changes(items: &[ScorePreviewItem]) -> bool {
    items.iter().any(|item| item.changes.iter().any(|change| change.change_value > 0))
}

fn get_time_base_score(minutes: i32) -> i32 {
    if minutes <= 15 {
        1
    } else if minutes <= 30 {
        2
    } else if minutes <= 60 {
        3
    } else if minutes <= 90 {
        4
    } else if minutes <= 120 {
        5
    } else if minutes <= 180 {
        6
    } else {
        7
    }
}

fn get_difficulty_multiplier(difficulty_star: i32) -> f64 {
    match difficulty_star {
        0 => 1.0,
        1 => 0.9,
        2 => 1.0,
        3 => 1.2,
        _ => 1.0,
    }
}

fn apply_daily_caps(
    items: Vec<ScorePreviewItem>,
    dimensions: &[dimension_repo::Dimension],
) -> Result<Vec<ScorePreviewItem>, String> {
    let mut remaining_caps = std::collections::HashMap::new();
    for dimension in dimensions {
        remaining_caps.insert(dimension.key.clone(), dimension.daily_cap.max(0));
    }

    let mut normalized_items = Vec::with_capacity(items.len());
    for item in items {
        let mut changes = Vec::new();
        for change in item.changes {
            let Some(remaining) = remaining_caps.get_mut(&change.dimension_key) else {
                return Err(format!("未知维度: {}", change.dimension_key));
            };

            let allowed = change.change_value.max(0).min(*remaining);
            *remaining -= allowed;

            if allowed > 0 {
                changes.push(ScorePreviewChange {
                    dimension_key: change.dimension_key,
                    change_value: allowed,
                });
            }
        }

        normalized_items.push(ScorePreviewItem {
            changes,
            ..item
        });
    }

    Ok(normalized_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    fn test_dimensions() -> Vec<dimension_repo::Dimension> {
        vec![
            dimension_repo::Dimension {
                key: "knowledge".into(),
                name: "学识".into(),
                daily_cap: 5,
            },
            dimension_repo::Dimension {
                key: "willpower".into(),
                name: "觉悟".into(),
                daily_cap: 8,
            },
        ]
    }

    #[test]
    fn apply_daily_caps_preserves_record_order_and_zero_changes() {
        let normalized = apply_daily_caps(
            vec![
                ScorePreviewItem {
                    record_id: Some(1),
                    title: "任务 A".into(),
                    category: "学习".into(),
                    changes: vec![ScorePreviewChange {
                        dimension_key: "knowledge".into(),
                        change_value: 5,
                    }],
                    difficulty_star: 2,
                    confidence: 1.0,
                    reason: "A".into(),
                    engine: "local".into(),
                },
                ScorePreviewItem {
                    record_id: Some(2),
                    title: "任务 B".into(),
                    category: "学习".into(),
                    changes: vec![ScorePreviewChange {
                        dimension_key: "knowledge".into(),
                        change_value: 3,
                    }],
                    difficulty_star: 2,
                    confidence: 1.0,
                    reason: "B".into(),
                    engine: "local".into(),
                },
            ],
            &test_dimensions(),
        )
        .expect("normalize preview");

        assert_eq!(normalized[0].changes[0].change_value, 5);
        assert!(normalized[1].changes.is_empty());
    }

    #[test]
    fn confirm_score_preview_rejects_duplicate_confirmations() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in memory db");
        run_migrations(&conn).expect("migrations");
        ledger_repo::insert_ledger(
            &conn,
            "2026-06-10",
            None,
            "knowledge",
            3,
            "任务 A",
            "学习",
            Some(1.0),
            "local",
        )
        .expect("seed ledger");

        let duplicate = ledger_repo::has_active_entries_for_date(&conn, "2026-06-10")
            .expect("query active entries");
        assert!(duplicate);
    }
}
