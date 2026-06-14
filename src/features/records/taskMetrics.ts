import type { RecordItem } from "@/api/client/tauriCommands";

export interface TaskTreeItem {
  record: RecordItem;
  children: TaskTreeItem[];
  selfElapsedSeconds: number;
  totalElapsedSeconds: number;
  remainingSeconds: number | null;
  isRunning: boolean;
}

export interface TaskTimeChartItem {
  id: number;
  title: string;
  value: number;
}

export function buildTaskTree(records: RecordItem[], nowMs = Date.now()): TaskTreeItem[] {
  const nodeMap = new Map<number, TaskTreeItem>();
  const roots: TaskTreeItem[] = [];

  for (const record of records) {
    nodeMap.set(record.id, createTreeNode(record, nowMs));
  }

  for (const record of records) {
    const node = nodeMap.get(record.id);
    if (!node) {
      continue;
    }

    const parentId = record.parent_id ?? null;
    if (parentId && nodeMap.has(parentId)) {
      nodeMap.get(parentId)?.children.push(node);
    } else {
      roots.push(node);
    }
  }

  for (const root of roots) {
    hydrateTotals(root);
  }

  return roots;
}

export function buildTaskTimeChartItems(
  records: RecordItem[],
  nowMs = Date.now()
): TaskTimeChartItem[] {
  return records
    .map((record) => ({
      id: record.id,
      title: record.title,
      value: getDisplayElapsedSeconds(record, nowMs),
    }))
    .filter((item) => item.value > 0);
}

export function getDisplayElapsedSeconds(record: RecordItem, nowMs = Date.now()): number {
  const persisted = record.elapsed_seconds ?? record.minutes * 60;
  const startedAt = record.timer_started_at ? parseTimerTimestamp(record.timer_started_at) : null;
  if (!startedAt) {
    return Math.max(0, persisted);
  }
  const deltaSeconds = Math.max(0, Math.floor((nowMs - startedAt) / 1000));
  return Math.max(0, persisted + deltaSeconds);
}

export function getDisplayMinutes(record: RecordItem, nowMs = Date.now()): number {
  const elapsedSeconds = getDisplayElapsedSeconds(record, nowMs);
  if (elapsedSeconds <= 0) {
    return 0;
  }
  return Math.ceil(elapsedSeconds / 60);
}

export function getCountdownRemainingSeconds(
  elapsedSeconds: number,
  countdownTargetSeconds: number
): number {
  return Math.max(0, countdownTargetSeconds - elapsedSeconds);
}

export function isCountdownExpired(record: RecordItem, nowMs = Date.now()): boolean {
  if ((record.timer_mode ?? "stopwatch") !== "countdown") {
    return false;
  }
  if (!record.timer_started_at || !record.countdown_target_seconds) {
    return false;
  }
  return getCountdownRemainingSeconds(
    getDisplayElapsedSeconds(record, nowMs),
    record.countdown_target_seconds
  ) === 0;
}

export function formatDuration(totalSeconds: number): string {
  const safeSeconds = Math.max(0, totalSeconds);
  const hours = Math.floor(safeSeconds / 3600);
  const minutes = Math.floor((safeSeconds % 3600) / 60);
  const seconds = safeSeconds % 60;

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

function createTreeNode(record: RecordItem, nowMs: number): TaskTreeItem {
  const selfElapsedSeconds = getDisplayElapsedSeconds(record, nowMs);
  const timerMode = record.timer_mode ?? "stopwatch";
  return {
    record,
    children: [],
    selfElapsedSeconds,
    totalElapsedSeconds: selfElapsedSeconds,
    remainingSeconds:
      timerMode === "countdown" && record.countdown_target_seconds
        ? getCountdownRemainingSeconds(selfElapsedSeconds, record.countdown_target_seconds)
        : null,
    isRunning: Boolean(record.timer_started_at),
  };
}

function hydrateTotals(node: TaskTreeItem): number {
  const childTotal = node.children.reduce((sum, child) => sum + hydrateTotals(child), 0);
  node.totalElapsedSeconds = node.selfElapsedSeconds + childTotal;
  return node.totalElapsedSeconds;
}

function parseTimerTimestamp(timestamp: string): number | null {
  const parsed = Date.parse(timestamp.replace(" ", "T"));
  return Number.isNaN(parsed) ? null : parsed;
}
