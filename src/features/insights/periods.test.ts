import { describe, expect, it } from "vitest";
import { normalizeInsightPeriod, reportKindLabel } from "@/features/insights/periods";

describe("insight periods", () => {
  it("normalizes unknown values to day reports", () => {
    expect(normalizeInsightPeriod("week")).toBe("week");
    expect(normalizeInsightPeriod("month")).toBe("month");
    expect(normalizeInsightPeriod("stray")).toBe("day");
    expect(normalizeInsightPeriod()).toBe("day");
  });

  it("keeps report labels distinct for the cabinet", () => {
    expect(reportKindLabel("tarot", "day")).toBe("塔罗");
    expect(reportKindLabel("report", "day")).toBe("日报");
    expect(reportKindLabel("report", "week")).toBe("周报");
    expect(reportKindLabel("report", "month")).toBe("月报");
  });
});
