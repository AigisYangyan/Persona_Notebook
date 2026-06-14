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

  it("omits container parent tasks with zero self minutes from analysis records", () => {
    const records: RecordItem[] = [
      {
        id: 1,
        date: "2026-06-10",
        title: "课程项目",
        minutes: 0,
        difficulty_star: 0,
        parent_id: null,
        is_completed: false,
        completed_at: null,
        elapsed_seconds: 0,
        timer_mode: "stopwatch",
        countdown_target_seconds: null,
        timer_started_at: null,
      },
      {
        id: 2,
        date: "2026-06-10",
        title: "接口联调",
        minutes: 25,
        difficulty_star: 2,
        parent_id: 1,
        is_completed: false,
        completed_at: null,
        elapsed_seconds: 1500,
        timer_mode: "stopwatch",
        countdown_target_seconds: null,
        timer_started_at: null,
      },
    ];
    const rulePreview: ScorePreviewItem[] = [
      {
        record_id: 2,
        title: "接口联调",
        category: "本地规则: knowledge",
        changes: [{ dimension_key: "knowledge", change_value: 3 }],
        difficulty_star: 2,
        confidence: 0.82,
        reason: "命中学习规则。",
        engine: "rules",
      },
    ];

    const request = buildDailyAnalysisRequest("2026-06-10", records, rulePreview);

    expect(request.records).toEqual([
      {
        title: "接口联调",
        minutes: 25,
        difficulty_star: 2,
      },
    ]);
  });

  it("keeps parent tasks that have their own self minutes", () => {
    const records: RecordItem[] = [
      {
        id: 1,
        date: "2026-06-10",
        title: "毕业设计",
        minutes: 10,
        difficulty_star: 3,
        parent_id: null,
        is_completed: false,
        completed_at: null,
        elapsed_seconds: 600,
        timer_mode: "stopwatch",
        countdown_target_seconds: null,
        timer_started_at: null,
      },
      {
        id: 2,
        date: "2026-06-10",
        title: "论文润色",
        minutes: 35,
        difficulty_star: 2,
        parent_id: 1,
        is_completed: false,
        completed_at: null,
        elapsed_seconds: 2100,
        timer_mode: "countdown",
        countdown_target_seconds: 2400,
        timer_started_at: null,
      },
    ];
    const rulePreview: ScorePreviewItem[] = [
      {
        record_id: 1,
        title: "毕业设计",
        category: "本地规则: knowledge",
        changes: [{ dimension_key: "knowledge", change_value: 1 }],
        difficulty_star: 3,
        confidence: 0.82,
        reason: "命中项目规则。",
        engine: "rules",
      },
      {
        record_id: 2,
        title: "论文润色",
        category: "本地规则: expression",
        changes: [{ dimension_key: "expression", change_value: 2 }],
        difficulty_star: 2,
        confidence: 0.8,
        reason: "命中文稿规则。",
        engine: "rules",
      },
    ];

    const request = buildDailyAnalysisRequest("2026-06-10", records, rulePreview);

    expect(request.records).toEqual([
      {
        title: "毕业设计",
        minutes: 10,
        difficulty_star: 3,
      },
      {
        title: "论文润色",
        minutes: 35,
        difficulty_star: 2,
      },
    ]);
  });
});
