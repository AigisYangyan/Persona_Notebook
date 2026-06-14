import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useInsightStore } from "@/stores/insightStore";

const tarotReport = {
  id: 1,
  report_kind: "tarot",
  period_type: "day",
  start_date: "2026-06-13",
  end_date: "2026-06-13",
  title: "力量",
  summary: "稳住身体节奏",
  content_json: { report: { card_name: "力量" } },
  context_snapshot_id: 10,
  status: "success",
  error_message: null,
  memory_patch_apply_status: "applied",
  memory_patch_apply_message: "ok",
  created_at: "2026-06-13 08:00:00",
};

const deletedReports = new Set<number>();

vi.mock("@/api/client/tauriCommands", () => ({
  generateTarotInsight: vi.fn(async () => tarotReport),
  generatePeriodReport: vi.fn(async (periodType: string) => ({
    ...tarotReport,
    id: periodType === "week" ? 2 : 3,
    report_kind: "report",
    period_type: periodType,
    title: `${periodType} report`,
  })),
  listInsightReports: vi.fn(async (reportKind?: string | null, periodType?: string | null) =>
    deletedReports.has(tarotReport.id)
      ? []
      : [
          {
            ...tarotReport,
            report_kind: reportKind ?? "tarot",
            period_type: periodType ?? "day",
          },
        ]
  ),
  getInsightContextSnapshot: vi.fn(async () => ({
    id: 10,
    report_kind: "tarot",
    period_type: "day",
    start_date: "2026-06-13",
    end_date: "2026-06-13",
    context_json: "{}",
    created_at: "2026-06-13 08:00:00",
  })),
  deleteInsightReport: vi.fn(async (reportId: number) => {
    deletedReports.add(reportId);
  }),
}));

describe("insightStore", () => {
  beforeEach(() => {
    deletedReports.clear();
    setActivePinia(createPinia());
  });

  it("refreshes the tarot cabinet after generation", async () => {
    const store = useInsightStore();

    await store.generateTarot("2026-06-13");

    expect(store.currentTarot?.title).toBe("力量");
    expect(store.tarotCabinet).toHaveLength(1);
  });

  it("keeps report periods separate", async () => {
    const store = useInsightStore();

    await store.generateReport("week", "2026-06-13");
    await store.loadReportCabinet("month");

    expect(store.currentReport?.period_type).toBe("week");
    expect(store.reportCabinet[0].period_type).toBe("month");
  });

  it("removes a selected tarot report from the cabinet", async () => {
    const store = useInsightStore();

    await store.generateTarot("2026-06-13");
    await store.removeReport(tarotReport.id, "tarot", "day");

    expect(store.currentTarot).toBeNull();
  });
});
