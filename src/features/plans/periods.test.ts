import { describe, expect, it } from "vitest";
import { formatCycleRange, getPeriodLabel, shiftAnchorDate, summarizeGrowth } from "@/features/plans/periods";

describe("plan periods", () => {
  it("shifts week anchors by seven days", () => {
    expect(shiftAnchorDate("week", "2026-06-13", 1)).toBe("2026-06-20");
  });

  it("shifts month anchors across year boundaries", () => {
    expect(shiftAnchorDate("month", "2026-12-10", 1)).toBe("2027-01-10");
  });

  it("formats week ranges from cycle dates", () => {
    expect(
      formatCycleRange({
        id: 1,
        period_type: "week",
        start_date: "2026-06-08",
        end_date: "2026-06-14",
        title: "",
        summary: "",
        ai_summary: "",
        last_ai_run_at: null,
      })
    ).toBe("2026-06-08 - 2026-06-14");
  });

  it("builds compact summaries from active dimensions", () => {
    expect(
      summarizeGrowth([
        {
          key: "knowledge",
          name: "知识",
          total: 8,
          daily_cap: 10,
          max_total: 70,
          progress_percent: 50,
          headline: "稳定推进",
        },
        {
          key: "willpower",
          name: "觉悟",
          total: 2,
          daily_cap: 8,
          max_total: 56,
          progress_percent: 12,
          headline: "轻微增长",
        },
      ])
    ).toContain("知识稳定推进");
  });

  it("shows month labels in yyyy-MM", () => {
    expect(getPeriodLabel("month", "2026-06-13")).toBe("2026-06");
  });
});
