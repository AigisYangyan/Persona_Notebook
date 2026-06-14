import { describe, expect, it } from "vitest";
import {
  buildTaskTimeChartItems,
  buildTaskTree,
  getCountdownRemainingSeconds,
} from "@/features/records/taskMetrics";
import type { RecordItem } from "@/api/client/tauriCommands";

const baseRecord = (overrides: Partial<RecordItem>): RecordItem => ({
  id: 1,
  date: "2026-06-10",
  title: "任务",
  minutes: 0,
  difficulty_star: 0,
  parent_id: null,
  is_completed: false,
  completed_at: null,
  elapsed_seconds: 0,
  timer_mode: "stopwatch",
  countdown_target_seconds: null,
  timer_started_at: null,
  ...overrides,
});

describe("taskMetrics", () => {
  it("aggregates child elapsed time into parent totals while preserving self elapsed time", () => {
    const tree = buildTaskTree([
      baseRecord({ id: 1, title: "毕业设计", elapsed_seconds: 600, minutes: 10 }),
      baseRecord({ id: 2, title: "查资料", parent_id: 1, elapsed_seconds: 1200, minutes: 20 }),
      baseRecord({ id: 3, title: "写提纲", parent_id: 1, elapsed_seconds: 1800, minutes: 30 }),
    ]);

    expect(tree).toHaveLength(1);
    expect(tree[0].selfElapsedSeconds).toBe(600);
    expect(tree[0].totalElapsedSeconds).toBe(3600);
    expect(tree[0].children.map((item) => item.totalElapsedSeconds)).toEqual([1200, 1800]);
  });

  it("builds pie chart items without double counting parent aggregate time", () => {
    const items = buildTaskTimeChartItems([
      baseRecord({ id: 1, title: "毕业设计", elapsed_seconds: 600, minutes: 10 }),
      baseRecord({ id: 2, title: "查资料", parent_id: 1, elapsed_seconds: 1200, minutes: 20 }),
      baseRecord({ id: 3, title: "写提纲", parent_id: 1, elapsed_seconds: 1800, minutes: 30 }),
      baseRecord({ id: 4, title: "健身", elapsed_seconds: 900, minutes: 15 }),
    ]);

    expect(items).toEqual([
      { id: 1, title: "毕业设计", value: 600 },
      { id: 2, title: "查资料", value: 1200 },
      { id: 3, title: "写提纲", value: 1800 },
      { id: 4, title: "健身", value: 900 },
    ]);
  });

  it("never returns a negative countdown remaining value", () => {
    expect(getCountdownRemainingSeconds(15, 10)).toBe(0);
    expect(getCountdownRemainingSeconds(8, 10)).toBe(2);
  });
});
