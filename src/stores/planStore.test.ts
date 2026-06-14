import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { usePlanStore } from "@/stores/planStore";

vi.mock("@/api/client/tauriCommands", () => ({
  getWeekPlan: vi.fn(async (anchorDate?: string) => ({
    cycle: {
      id: 1,
      period_type: "week",
      start_date: anchorDate ?? "2026-06-08",
      end_date: "2026-06-14",
      title: "Week",
      summary: "",
      ai_summary: "",
      last_ai_run_at: null,
    },
    items: [],
    goal_progress: {
      total_items: 0,
      completed_items: 0,
      active_goal_count: 0,
      average_progress_percent: 0,
    },
    growth: {
      start_date: "2026-06-08",
      end_date: "2026-06-14",
      total_days: 7,
      active_days: 0,
      analyzed_days: 0,
      record_count: 0,
      total_change: 0,
      dimensions: [],
    },
    related_weeks: [],
  })),
  getMonthPlan: vi.fn(async (anchorDate?: string) => ({
    cycle: {
      id: 2,
      period_type: "month",
      start_date: anchorDate ?? "2026-06-01",
      end_date: "2026-06-30",
      title: "Month",
      summary: "",
      ai_summary: "",
      last_ai_run_at: null,
    },
    items: [],
    goal_progress: {
      total_items: 0,
      completed_items: 0,
      active_goal_count: 0,
      average_progress_percent: 0,
    },
    growth: {
      start_date: "2026-06-01",
      end_date: "2026-06-30",
      total_days: 30,
      active_days: 0,
      analyzed_days: 0,
      record_count: 0,
      total_change: 0,
      dimensions: [],
    },
    related_weeks: [],
  })),
  savePlanCycle: vi.fn(),
  savePlanItem: vi.fn(),
  deletePlanItem: vi.fn(),
  refreshPlanProgress: vi.fn(),
  submitPlanAiAnswers: vi.fn(),
  applyPlanAiUpdate: vi.fn(),
}));

describe("planStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("keeps week and month snapshots separate", async () => {
    const store = usePlanStore();

    await store.loadWeekPlan("2026-06-08");
    await store.loadMonthPlan("2026-06-01");

    expect(store.weekPlan?.cycle.period_type).toBe("week");
    expect(store.monthPlan?.cycle.period_type).toBe("month");
    expect(store.weekPlan?.cycle.start_date).toBe("2026-06-08");
    expect(store.monthPlan?.cycle.start_date).toBe("2026-06-01");
  });
});
