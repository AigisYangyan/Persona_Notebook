import { format } from "date-fns";

export function getTodayStr(): string {
  return format(new Date(), "yyyy-MM-dd");
}

export function formatDate(dateStr: string): string {
  return format(new Date(dateStr), "yyyy年MM月dd日");
}
