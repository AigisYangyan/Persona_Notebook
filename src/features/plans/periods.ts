import {
  addMonths,
  addWeeks,
  endOfMonth,
  endOfWeek,
  format,
  parseISO,
  startOfMonth,
  startOfWeek,
} from "date-fns";
import type { PlanCycle, PlanGrowthDimension, PlanPeriodType } from "@/api/client/tauriCommands";

export function normalizeAnchorDate(dateStr?: string): string {
  return dateStr || format(new Date(), "yyyy-MM-dd");
}

export function shiftAnchorDate(
  periodType: PlanPeriodType,
  anchorDate: string,
  delta: number
): string {
  const parsed = parseISO(anchorDate);
  const nextDate = periodType === "week" ? addWeeks(parsed, delta) : addMonths(parsed, delta);
  return format(nextDate, "yyyy-MM-dd");
}

export function formatCycleRange(cycle: PlanCycle): string {
  if (cycle.period_type === "week") {
    return `${cycle.start_date} - ${cycle.end_date}`;
  }
  const monthStart = startOfMonth(parseISO(cycle.start_date));
  const monthEnd = endOfMonth(monthStart);
  return `${format(monthStart, "yyyy-MM")} (${format(monthStart, "MM/dd")} - ${format(monthEnd, "MM/dd")})`;
}

export function getPeriodLabel(periodType: PlanPeriodType, anchorDate: string): string {
  const parsed = parseISO(anchorDate);
  if (periodType === "week") {
    const start = startOfWeek(parsed, { weekStartsOn: 1 });
    const end = endOfWeek(parsed, { weekStartsOn: 1 });
    return `${format(start, "MM/dd")} - ${format(end, "MM/dd")}`;
  }
  return format(startOfMonth(parsed), "yyyy-MM");
}

export function summarizeGrowth(dimensions: PlanGrowthDimension[]): string {
  const active = dimensions.filter((item) => item.total > 0);
  if (active.length === 0) {
    return "本期仍在蓄力，成长账本还没有明显抬头。";
  }
  const sorted = [...active].sort((left, right) => right.progress_percent - left.progress_percent);
  return sorted
    .slice(0, 2)
    .map((item) => `${item.name}${item.headline}`)
    .join(" / ");
}
