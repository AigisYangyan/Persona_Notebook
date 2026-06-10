import { describe, expect, it } from "vitest";
import type { RecordItem, ScorePreviewItem } from "@/api/client/tauriCommands";
import { buildDailyAnalysisRequest } from "@/features/scoring/requestBuilder";

describe("buildDailyAnalysisRequest", () => {
  it("embeds deterministic rule hints into the API request", () => {
    const records: RecordItem[] = [
      {
        id: 1,
        date: "2026-06-10",
        title: "高等数学",
        minutes: 90,
        difficulty_star: 2,
      },
      {
        id: 2,
        date: "2026-06-10",
        title: "健身",
        minutes: 45,
        difficulty_star: 1,
      },
    ];
    const rulePreview: ScorePreviewItem[] = [
      {
        record_id: 1,
        title: "高等数学",
        category: "本地规则: knowledge",
        changes: [
          { dimension_key: "knowledge", change_value: 4 },
          { dimension_key: "willpower", change_value: 1 },
        ],
        difficulty_star: 2,
        confidence: 0.82,
        reason: "命中学习规则。",
        engine: "rules",
      },
      {
        record_id: 2,
        title: "健身",
        category: "本地规则: physique",
        changes: [
          { dimension_key: "physique", change_value: 2 },
          { dimension_key: "willpower", change_value: 1 },
        ],
        difficulty_star: 1,
        confidence: 0.8,
        reason: "命中运动规则。",
        engine: "rules",
      },
    ];

    const request = buildDailyAnalysisRequest("2026-06-10", records, rulePreview);

    expect(request.feedback_mode).toBe("rules_api");
    expect(request.rule_hints.source).toBe("deterministic_rules_cache");
    expect(request.rule_hints.record_hints).toEqual([
      {
        record_index: 0,
        title: "高等数学",
        category: "本地规则: knowledge",
        suggested_dimensions: ["knowledge", "willpower"],
        suggested_changes: {
          knowledge: 4,
          willpower: 1,
        },
        confidence: 0.82,
        reason: "命中学习规则。",
      },
      {
        record_index: 1,
        title: "健身",
        category: "本地规则: physique",
        suggested_dimensions: ["physique", "willpower"],
        suggested_changes: {
          physique: 2,
          willpower: 1,
        },
        confidence: 0.8,
        reason: "命中运动规则。",
      },
    ]);
    expect(request.rule_hints.suggested_totals).toEqual({
      knowledge: 4,
      willpower: 2,
      physique: 2,
    });
  });
});
