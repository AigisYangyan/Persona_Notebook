import type { RecordItem, ScorePreviewItem } from "@/api/client/tauriCommands";
import type { DailyAnalysisRequest, DailyAnalysisResponse } from "@/schemas/score";

export type PreviewRecordRef = Pick<RecordItem, "id" | "title" | "difficulty_star">;
export type { ScorePreviewItem };

export interface NormalizedPreview {
  items: ScorePreviewItem[];
  summary: string;
}

export function applyDailyCapsToPreview(
  items: ScorePreviewItem[],
  dimensions: DailyAnalysisRequest["stat_dimensions"]
): ScorePreviewItem[] {
  const remainingCaps = new Map(dimensions.map((dimension) => [dimension.key, dimension.daily_cap]));

  return items.map((item) => {
    const changes = item.changes.flatMap((change) => {
      const remaining = remainingCaps.get(change.dimension_key) ?? 0;
      const allowed = Math.min(Math.max(change.change_value, 0), remaining);
      remainingCaps.set(change.dimension_key, Math.max(remaining - allowed, 0));
      return allowed > 0
        ? [{ dimension_key: change.dimension_key, change_value: allowed }]
        : [];
    });

    return {
      ...item,
      changes,
    };
  });
}

export function normalizeApiPreview(
  request: DailyAnalysisRequest,
  response: DailyAnalysisResponse,
  records: PreviewRecordRef[]
): NormalizedPreview {
  const items = response.record_results.map<ScorePreviewItem>((result, index) => ({
    record_id: records[index]?.id ?? null,
    title: result.title,
    category: result.category,
    changes: Object.entries(result.changes).map(([dimension_key, change_value]) => ({
      dimension_key,
      change_value,
    })),
    difficulty_star: result.difficulty_star,
    confidence: result.confidence,
    reason: result.reason,
    engine: "rules_api",
  }));

  return {
    items: applyDailyCapsToPreview(items, request.stat_dimensions),
    summary: response.summary,
  };
}

export function buildRuleHintSummary(items: ScorePreviewItem[]): string {
  const totalPoints = items.reduce(
    (sum, item) => sum + item.changes.reduce((changeSum, change) => changeSum + change.change_value, 0),
    0
  );
  return `规则缓存已为 ${items.length} 条记录生成 ${totalPoints} 点建议，API 将在这些约束上复核并给出最终反馈。`;
}
