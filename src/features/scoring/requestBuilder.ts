import type { RecordItem, ScorePreviewItem } from "@/api/client/tauriCommands";
import { buildRuleHintSummary } from "@/features/scoring/preview";
import type { DailyAnalysisRequest } from "@/schemas/score";

const DEFAULT_STAT_DIMENSIONS: DailyAnalysisRequest["stat_dimensions"] = [
  { key: "knowledge", name: "学识", daily_cap: 10 },
  { key: "willpower", name: "觉悟", daily_cap: 8 },
  { key: "expression", name: "表达", daily_cap: 8 },
  { key: "physique", name: "体魄", daily_cap: 8 },
  { key: "bond", name: "羁绊", daily_cap: 6 },
];

const DEFAULT_SCORE_RULES: DailyAnalysisRequest["score_rules"] = {
  time_base: {
    "0-15": 1,
    "16-30": 2,
    "31-60": 3,
    "61-90": 4,
    "91-120": 5,
    "121-180": 6,
    "181+": 7,
  },
  difficulty_multiplier: { "0": 1.0, "1": 0.9, "2": 1.0, "3": 1.2 },
  max_dims_per_record: 3,
  allocation_ratio: { primary: 0.7, secondary: 0.3 },
};

function toSuggestedChanges(changes: ScorePreviewItem["changes"]): Record<string, number> {
  return Object.fromEntries(
    changes.map((change) => [change.dimension_key, change.change_value])
  );
}

function buildSuggestedTotals(items: ScorePreviewItem[]): Record<string, number> {
  return items.reduce<Record<string, number>>((totals, item) => {
    for (const change of item.changes) {
      totals[change.dimension_key] = (totals[change.dimension_key] || 0) + change.change_value;
    }
    return totals;
  }, {});
}

export function buildDailyAnalysisRequest(
  date: string,
  records: RecordItem[],
  rulePreview: ScorePreviewItem[]
): DailyAnalysisRequest {
  const analyzableRecords = records.filter((record) => record.minutes > 0);
  const analyzableIds = new Set(analyzableRecords.map((record) => record.id));
  const filteredRulePreview = rulePreview.filter((item) =>
    item.record_id == null ? true : analyzableIds.has(item.record_id)
  );

  return {
    version: "1.0",
    feedback_mode: "rules_api",
    date,
    records: analyzableRecords.map((record) => ({
      title: record.title,
      minutes: record.minutes,
      difficulty_star: record.difficulty_star,
    })),
    stat_dimensions: DEFAULT_STAT_DIMENSIONS,
    score_rules: DEFAULT_SCORE_RULES,
    rule_hints: {
      source: "deterministic_rules_cache",
      summary: buildRuleHintSummary(filteredRulePreview),
      suggested_totals: buildSuggestedTotals(filteredRulePreview),
      record_hints: filteredRulePreview.map((item, index) => ({
        record_index: index,
        title: item.title,
        category: item.category,
        suggested_dimensions: item.changes.map((change) => change.dimension_key),
        suggested_changes: toSuggestedChanges(item.changes),
        confidence: item.confidence,
        reason: item.reason,
      })),
    },
  };
}
