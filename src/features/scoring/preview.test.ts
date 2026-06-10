import { describe, expect, it } from "vitest";
import {
  applyDailyCapsToPreview,
  buildRuleHintSummary,
  normalizeApiPreview,
  type ScorePreviewItem,
} from "@/features/scoring/preview";
import type { DailyAnalysisRequest, DailyAnalysisResponse } from "@/schemas/score";

const request: DailyAnalysisRequest = {
  version: "1.0",
  feedback_mode: "rules_api",
  date: "2026-06-10",
  records: [
    { title: "完成线代作业", minutes: 90, difficulty_star: 2 },
    { title: "健身", minutes: 60, difficulty_star: 2 },
  ],
  stat_dimensions: [
    { key: "knowledge", name: "学识", daily_cap: 5 },
    { key: "willpower", name: "觉悟", daily_cap: 8 },
    { key: "expression", name: "表达", daily_cap: 8 },
    { key: "physique", name: "体魄", daily_cap: 3 },
    { key: "bond", name: "羁绊", daily_cap: 6 },
  ],
  score_rules: {
    time_base: {
      "0-15": 1,
      "16-30": 2,
      "31-60": 3,
      "61-90": 4,
      "91-120": 5,
      "121-180": 6,
      "181+": 7,
    },
    difficulty_multiplier: {
      "0": 1,
      "1": 0.9,
      "2": 1,
      "3": 1.2,
    },
    max_dims_per_record: 3,
    allocation_ratio: {
      primary: 0.7,
      secondary: 0.3,
    },
  },
  rule_hints: {
    source: "deterministic_rules_cache",
    summary: "规则缓存已准备完成。",
    suggested_totals: {
      knowledge: 5,
      willpower: 2,
      expression: 0,
      physique: 3,
      bond: 0,
    },
    record_hints: [
      {
        record_index: 0,
        title: "完成线代作业",
        category: "学习",
        suggested_dimensions: ["knowledge", "willpower"],
        suggested_changes: {
          knowledge: 4,
          willpower: 2,
        },
        confidence: 0.82,
        reason: "命中学习规则。",
      },
      {
        record_index: 1,
        title: "健身",
        category: "运动",
        suggested_dimensions: ["physique", "willpower"],
        suggested_changes: {
          physique: 3,
          willpower: 1,
        },
        confidence: 0.8,
        reason: "命中运动规则。",
      },
    ],
  },
};

describe("normalizeApiPreview", () => {
  it("maps API results onto record ids and clips daily caps", () => {
    const response: DailyAnalysisResponse = {
      version: "1.0",
      date: "2026-06-10",
      total_changes: {
        knowledge: 5,
        willpower: 2,
        expression: 0,
        physique: 3,
        bond: 0,
      },
      record_results: [
        {
          title: "完成线代作业",
          category: "学习",
          changes: { knowledge: 4, willpower: 2 },
          difficulty_star: 2,
          confidence: 0.91,
          reason: "学习任务以学识为主，也体现执行力。",
        },
        {
          title: "健身",
          category: "运动",
          changes: { knowledge: 3, physique: 4 },
          difficulty_star: 2,
          confidence: 0.76,
          reason: "运动主要提升体魄。",
        },
      ],
      summary: "今日成长以学习和运动为主。",
    };

    const preview = normalizeApiPreview(request, response, [
      { id: 11, title: "完成线代作业", difficulty_star: 2 },
      { id: 12, title: "健身", difficulty_star: 2 },
    ]);

    expect(preview.items).toEqual<ScorePreviewItem[]>([
      {
        record_id: 11,
        title: "完成线代作业",
        category: "学习",
        changes: [
          { dimension_key: "knowledge", change_value: 4 },
          { dimension_key: "willpower", change_value: 2 },
        ],
        difficulty_star: 2,
        confidence: 0.91,
        reason: "学习任务以学识为主，也体现执行力。",
        engine: "rules_api",
      },
      {
        record_id: 12,
        title: "健身",
        category: "运动",
        changes: [
          { dimension_key: "knowledge", change_value: 1 },
          { dimension_key: "physique", change_value: 3 },
        ],
        difficulty_star: 2,
        confidence: 0.76,
        reason: "运动主要提升体魄。",
        engine: "rules_api",
      },
    ]);
    expect(preview.summary).toBe("今日成长以学习和运动为主。");
  });
});

describe("applyDailyCapsToPreview", () => {
  it("keeps zero-change records when caps are exhausted", () => {
    const preview = applyDailyCapsToPreview(
      [
        {
          record_id: 1,
          title: "任务 A",
          category: "学习",
          changes: [{ dimension_key: "knowledge", change_value: 5 }],
          difficulty_star: 2,
          confidence: 1,
          reason: "A",
          engine: "rules_api",
        },
        {
          record_id: 2,
          title: "任务 B",
          category: "学习",
          changes: [{ dimension_key: "knowledge", change_value: 3 }],
          difficulty_star: 2,
          confidence: 1,
          reason: "B",
          engine: "rules_api",
        },
      ],
      request.stat_dimensions
    );

    expect(preview[1]?.changes).toEqual([]);
  });
});

describe("buildRuleHintSummary", () => {
  it("describes rules cache and total suggested points", () => {
    const summary = buildRuleHintSummary([
      {
        record_id: 1,
        title: "任务 A",
        category: "学习",
        changes: [{ dimension_key: "knowledge", change_value: 3 }],
        difficulty_star: 2,
        confidence: 1,
        reason: "A",
        engine: "rules_api",
      },
      {
        record_id: 2,
        title: "任务 B",
        category: "运动",
        changes: [{ dimension_key: "physique", change_value: 2 }],
        difficulty_star: 2,
        confidence: 1,
        reason: "B",
        engine: "rules_api",
      },
    ]);

    expect(summary).toContain("规则缓存");
    expect(summary).toContain("2 条");
    expect(summary).toContain("5 点");
  });
});
