export type InsightReportKind = "tarot" | "report";
export type InsightPeriod = "day" | "week" | "month";

export function normalizeInsightPeriod(value?: string | null): InsightPeriod {
  if (value === "week" || value === "month") {
    return value;
  }
  return "day";
}

export function reportKindLabel(kind: InsightReportKind, period: InsightPeriod): string {
  if (kind === "tarot") {
    return "塔罗";
  }
  const labels: Record<InsightPeriod, string> = {
    day: "日报",
    week: "周报",
    month: "月报",
  };
  return labels[period];
}
