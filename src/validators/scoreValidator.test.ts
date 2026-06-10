import { describe, expect, it } from "vitest";
import type { DailyAnalysisRequest, DailyAnalysisResponse } from "@/schemas/score";
import { validateDailyResponse } from "@/validators/scoreValidator";

const request: DailyAnalysisRequest = {
  version: "1.0",
  feedback_mode: "rules_api",
  date: "2026-06-10",
  records: [{ title: "完成线代作业", minutes: 90, difficulty_star: 2 }],
  stat_dimensions: [
    { key: "knowledge", name: "学识", daily_cap: 10 },
    { key: "willpower", name: "觉悟", daily_cap: 8 },
  ],
  score_rules: {
    time_base: { "0-15": 1 },
    difficulty_multiplier: { "0": 1 },
    max_dims_per_record: 3,
    allocation_ratio: { primary: 0.7, secondary: 0.3 },
  },
  rule_hints: {
    source: "deterministic_rules_cache",
    summary: "规则缓存已准备完成。",
    suggested_totals: { knowledge: 4, willpower: 1 },
    record_hints: [
      {
        record_index: 0,
        title: "完成线代作业",
        category: "学习",
        suggested_dimensions: ["knowledge", "willpower"],
        suggested_changes: { knowledge: 4, willpower: 1 },
        confidence: 0.9,
        reason: "命中学习规则。",
      },
    ],
  },
};

describe("validateDailyResponse", () => {
  it("accepts a consistent response", () => {
    const response: DailyAnalysisResponse = {
      version: "1.0",
      date: "2026-06-10",
      total_changes: { knowledge: 4, willpower: 1 },
      record_results: [
        {
          title: "完成线代作业",
          category: "学习",
          changes: { knowledge: 4, willpower: 1 },
          difficulty_star: 2,
          confidence: 0.92,
          reason: "符合学习类任务。",
        },
      ],
      summary: "今日以学习成长为主。",
    };

    expect(validateDailyResponse(request, response)).toEqual({
      valid: true,
      errors: [],
    });
  });

  it("reports capped total mismatches", () => {
    const response: DailyAnalysisResponse = {
      version: "1.0",
      date: "2026-06-10",
      total_changes: { knowledge: 11, willpower: 1 },
      record_results: [
        {
          title: "完成线代作业",
          category: "学习",
          changes: { knowledge: 11, willpower: 1 },
          difficulty_star: 2,
          confidence: 0.92,
          reason: "符合学习类任务。",
        },
      ],
      summary: "今日以学习成长为主。",
    };

    const result = validateDailyResponse(request, response);
    expect(result.valid).toBe(false);
    expect(result.errors.some((item) => item.rule === "RULE_08_CAP")).toBe(true);
  });
});
