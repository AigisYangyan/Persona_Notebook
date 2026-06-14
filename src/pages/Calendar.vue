<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NButton, NModal, useMessage } from "naive-ui";
import {
  getCalendarInsightHistory,
  getCalendarOverview,
  getCalendarPlanHistory,
  getLedgerByDate,
  getRecordsByDate,
  rollbackLedger,
  type CalendarInsightHistory,
  type CalendarOverviewItem,
  type CalendarPlanHistory,
  type LedgerByDateEntry,
  type RecordItem,
} from "@/api/client/tauriCommands";
import { format } from "date-fns";
import { useStatStore } from "@/stores/statStore";

interface DisplayRecord extends RecordItem {
  level: number;
  hasChildren: boolean;
}

const currentYear = ref(new Date().getFullYear());
const currentMonth = ref(new Date().getMonth());
const selectedDate = ref("");
const showDetailModal = ref(false);
const loadingDetail = ref(false);
const rollingBackId = ref<number | null>(null);
const dayRecords = ref<RecordItem[]>([]);
const dayLedger = ref<LedgerByDateEntry[]>([]);
const dayPlanHistory = ref<CalendarPlanHistory | null>(null);
const dayInsightHistory = ref<CalendarInsightHistory | null>(null);
const monthOverview = ref<Record<string, CalendarOverviewItem>>({});

const message = useMessage();
const statStore = useStatStore();

const weekDays = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
const monthNames = [
  "JANUARY",
  "FEBRUARY",
  "MARCH",
  "APRIL",
  "MAY",
  "JUNE",
  "JULY",
  "AUGUST",
  "SEPTEMBER",
  "OCTOBER",
  "NOVEMBER",
  "DECEMBER",
];

const dimLabels: Record<string, string> = {
  knowledge: "知识",
  willpower: "觉悟",
  expression: "表达",
  physique: "体魄",
  bond: "羁绊",
};

const dimColors: Record<string, string> = {
  knowledge: "#00d4ff",
  willpower: "#ffcc00",
  expression: "#00ffaa",
  physique: "#ff3366",
  bond: "#9966ff",
};

const calendarDays = computed(() => {
  const firstDay = new Date(currentYear.value, currentMonth.value, 1);
  const lastDay = new Date(currentYear.value, currentMonth.value + 1, 0);
  const startWeekday = firstDay.getDay();
  const days: { date: number; dateStr: string; inMonth: boolean }[] = [];

  const prevLastDay = new Date(currentYear.value, currentMonth.value, 0).getDate();
  for (let index = startWeekday - 1; index >= 0; index -= 1) {
    days.push({ date: prevLastDay - index, dateStr: "", inMonth: false });
  }

  for (let date = 1; date <= lastDay.getDate(); date += 1) {
    const dateStr = format(new Date(currentYear.value, currentMonth.value, date), "yyyy-MM-dd");
    days.push({ date, dateStr, inMonth: true });
  }

  while (days.length < 42) {
    days.push({
      date: days.length - lastDay.getDate() - startWeekday + 1,
      dateStr: "",
      inMonth: false,
    });
  }

  return days;
});

const selectedOverview = computed(() =>
  selectedDate.value ? monthOverview.value[selectedDate.value] ?? null : null
);

const flatDayRecords = computed<DisplayRecord[]>(() => flattenRecords(dayRecords.value));
const ledgerTotalScore = computed(() =>
  dayLedger.value.reduce((sum, entry) => sum + entry.change_value, 0)
);
const visibleDayRecords = computed(() => flatDayRecords.value.slice(0, 4));
const hiddenRecordCount = computed(() => Math.max(0, flatDayRecords.value.length - visibleDayRecords.value.length));
const visibleDayLedger = computed(() => dayLedger.value.slice(0, 3));
const hiddenLedgerCount = computed(() => Math.max(0, dayLedger.value.length - visibleDayLedger.value.length));

const planHistoryCards = computed(() => {
  const history = dayPlanHistory.value;
  if (!history) {
    return [];
  }
  return [
    history.week_plan ? { key: "week", label: "Week Plan", ...history.week_plan } : null,
    history.month_plan ? { key: "month", label: "Month Plan", ...history.month_plan } : null,
  ].filter(Boolean) as Array<{
    key: string;
    label: string;
    cycle: NonNullable<CalendarPlanHistory["week_plan"]>["cycle"];
    goal_progress: NonNullable<CalendarPlanHistory["week_plan"]>["goal_progress"];
    items: NonNullable<CalendarPlanHistory["week_plan"]>["items"];
    is_historical: boolean;
  }>;
});

const reportHistoryCards = computed(() => {
  const history = dayInsightHistory.value;
  if (!history) {
    return [];
  }
  return [
    history.tarot ? { key: "tarot", label: "Tarot", report: history.tarot } : null,
    history.daily_report ? { key: "day", label: "Daily Report", report: history.daily_report } : null,
    history.week_report ? { key: "week", label: "Week Report", report: history.week_report } : null,
    history.month_report ? { key: "month", label: "Month Report", report: history.month_report } : null,
  ].filter(Boolean) as Array<{
    key: string;
    label: string;
    report: NonNullable<CalendarInsightHistory["daily_report"]>;
  }>;
});
const visibleReportHistoryCards = computed(() => reportHistoryCards.value.slice(0, 2));
const hiddenReportCount = computed(() => Math.max(0, reportHistoryCards.value.length - visibleReportHistoryCards.value.length));
const visiblePlanHistoryCards = computed(() => planHistoryCards.value.slice(0, 2));

watch(
  [currentYear, currentMonth],
  () => {
    void loadOverview();
  },
  { immediate: true }
);

async function loadOverview() {
  try {
    const overview = await getCalendarOverview(currentYear.value, currentMonth.value + 1);
    monthOverview.value = Object.fromEntries(overview.map((item) => [item.date, item]));
  } catch (error: any) {
    message.error(`加载日历失败: ${error.message}`);
  }
}

function prevMonth() {
  if (currentMonth.value === 0) {
    currentMonth.value = 11;
    currentYear.value -= 1;
    return;
  }
  currentMonth.value -= 1;
}

function nextMonth() {
  if (currentMonth.value === 11) {
    currentMonth.value = 0;
    currentYear.value += 1;
    return;
  }
  currentMonth.value += 1;
}

async function selectDay(dateStr: string) {
  if (!dateStr) {
    return;
  }
  selectedDate.value = dateStr;
  showDetailModal.value = true;
  await loadSelectedDateDetail();
}

async function loadSelectedDateDetail() {
  if (!selectedDate.value) {
    return;
  }

  loadingDetail.value = true;
  try {
    const [records, ledger, planHistory, insightHistory] = await Promise.all([
      getRecordsByDate(selectedDate.value),
      getLedgerByDate(selectedDate.value),
      getCalendarPlanHistory(selectedDate.value),
      getCalendarInsightHistory(selectedDate.value),
    ]);
    dayRecords.value = records;
    dayLedger.value = ledger;
    dayPlanHistory.value = planHistory;
    dayInsightHistory.value = insightHistory;
  } catch (error: any) {
    message.error(`加载日期详情失败: ${error.message}`);
  } finally {
    loadingDetail.value = false;
  }
}

async function handleRollback(ledgerId: number) {
  if (rollingBackId.value !== null) {
    return;
  }

  rollingBackId.value = ledgerId;
  try {
    await rollbackLedger(ledgerId);
    message.success("该条成长记录已撤销");
    await Promise.all([loadSelectedDateDetail(), loadOverview(), statStore.refreshStats()]);
  } catch (error: any) {
    message.error(`撤销失败: ${error.message}`);
  } finally {
    rollingBackId.value = null;
  }
}

function getDayOverview(dateStr: string) {
  return monthOverview.value[dateStr];
}

function isToday(dateStr: string): boolean {
  return dateStr === format(new Date(), "yyyy-MM-dd");
}

function flattenRecords(records: RecordItem[]): DisplayRecord[] {
  const byId = new Map(records.map((record) => [record.id, record]));
  const childMap = new Map<number | null, RecordItem[]>();

  for (const record of records) {
    const parentId = byId.has(record.parent_id ?? -1) ? (record.parent_id ?? null) : null;
    const siblings = childMap.get(parentId) ?? [];
    childMap.set(parentId, [...siblings, record]);
  }

  const flattened: DisplayRecord[] = [];
  const visited = new Set<number>();

  const visit = (record: RecordItem, level: number) => {
    if (visited.has(record.id)) {
      return;
    }
    visited.add(record.id);
    const children = childMap.get(record.id) ?? [];
    flattened.push({
      ...record,
      level,
      hasChildren: children.length > 0,
    });
    for (const child of children) {
      visit(child, level + 1);
    }
  };

  for (const root of childMap.get(null) ?? []) {
    visit(root, 0);
  }
  for (const record of records) {
    visit(record, 0);
  }

  return flattened;
}

function formatRecordDuration(record: RecordItem): string {
  const totalSeconds = record.elapsed_seconds ?? record.minutes * 60;
  if (totalSeconds < 60) {
    return `${totalSeconds} 秒`;
  }
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (hours > 0) {
    return minutes > 0 ? `${hours} 小时 ${minutes} 分钟` : `${hours} 小时`;
  }
  return `${Math.floor(totalSeconds / 60)} 分钟`;
}

function formatEngineLabel(engine: string): string {
  if (engine === "rules_api") {
    return "Rules + API";
  }
  if (engine === "api") {
    return "API";
  }
  if (engine === "local" || engine === "rules") {
    return "Rules";
  }
  return engine;
}

function formatChangeValue(value: number): string {
  return value > 0 ? `+${value}` : `${value}`;
}

function extractReportHighlight(report: Record<string, unknown>): string {
  const payload = typeof report.report === "object" && report.report !== null
    ? (report.report as Record<string, unknown>)
    : report;
  if (typeof payload.deeper_reading === "string" && payload.deeper_reading.trim()) {
    return payload.deeper_reading;
  }
  if (typeof payload.root_causes === "string" && payload.root_causes.trim()) {
    return payload.root_causes;
  }
  if (typeof payload.psychological_theme === "string" && payload.psychological_theme.trim()) {
    return payload.psychological_theme;
  }
  return "";
}
</script>

<template>
  <div class="cyber-page calendar-page">
    <h1 class="cyber-page-title">
      CALENDAR<span class="sub">日历归档</span>
    </h1>

    <div class="cal-header cyber-panel">
      <button class="cal-nav-btn" @click="prevMonth">◀</button>
      <div class="cal-date-display">
        <div class="cal-year">{{ currentYear }}</div>
        <div class="cal-month-row">
          <span class="cal-month-num">{{ String(currentMonth + 1).padStart(2, "0") }}</span>
          <span class="cal-month-name">{{ monthNames[currentMonth] }}</span>
        </div>
      </div>
      <div class="cal-summary">
        <div class="summary-label">ARCHIVE PANEL</div>
        <div class="summary-copy">点击日期即可查看当天任务、成长账本、报告历史，以及对应的周/月计划归档。</div>
      </div>
      <button class="cal-nav-btn" @click="nextMonth">▶</button>
    </div>

    <div class="cal-panel cyber-panel">
      <div class="cal-weekdays">
        <div
          v-for="wd in weekDays"
          :key="wd"
          class="weekday-header"
          :class="{ weekend: wd === 'SUN' || wd === 'SAT' }"
        >
          {{ wd }}
        </div>
      </div>

      <div class="cal-days">
        <div
          v-for="(day, idx) in calendarDays"
          :key="idx"
          class="cyber-cal-cell"
          :class="{
            'other-month': !day.inMonth,
            today: day.inMonth && isToday(day.dateStr),
            'has-record': day.inMonth && getDayOverview(day.dateStr)?.record_count > 0,
            analyzed: day.inMonth && getDayOverview(day.dateStr)?.is_analyzed,
            weekend: idx % 7 === 0 || idx % 7 === 6,
            interactive: day.inMonth,
          }"
          @click="day.inMonth && selectDay(day.dateStr)"
        >
          <span class="day-num">{{ day.date }}</span>
          <span v-if="day.inMonth && getDayOverview(day.dateStr)" class="day-status">
            {{ getDayOverview(day.dateStr)?.is_analyzed ? "已评" : "待评" }}
          </span>
          <div
            v-if="day.inMonth && (getDayOverview(day.dateStr)?.has_week_plan_update || getDayOverview(day.dateStr)?.has_month_plan_update)"
            class="day-markers"
          >
            <span v-if="getDayOverview(day.dateStr)?.has_week_plan_update" class="marker week">W</span>
            <span v-if="getDayOverview(day.dateStr)?.has_month_plan_update" class="marker month">M</span>
          </div>
          <span v-if="day.inMonth && isToday(day.dateStr)" class="today-label">TODAY</span>
        </div>
      </div>
    </div>

    <n-modal
      v-model:show="showDetailModal"
      preset="card"
      :title="`${selectedDate} 日期详情`"
      style="width: min(1040px, calc(100vw - 20px))"
      class="cyber-modal"
    >
      <div class="detail-header">
        <div>
          <div class="detail-date">{{ selectedDate }}</div>
          <div class="detail-subtitle">这里会同时展示当天任务、成长账本、报告归档，以及对应日期落入的周/月计划。</div>
        </div>
        <div class="detail-summary">
          <span class="detail-chip">{{ flatDayRecords.length }} 项任务</span>
          <span class="detail-chip">{{ dayLedger.length }} 条账本</span>
          <span class="detail-chip">{{ reportHistoryCards.length }} 条报告</span>
          <span class="detail-chip" :class="{ analyzed: selectedOverview?.is_analyzed }">
            {{ selectedOverview?.is_analyzed ? "已完成评分" : "尚未评分" }}
          </span>
          <span class="detail-chip score">总变化 {{ formatChangeValue(ledgerTotalScore) }}</span>
        </div>
      </div>

      <div class="detail-grid">
        <section class="detail-section">
          <div class="section-label">日记录</div>
          <div v-if="loadingDetail" class="empty-hint">加载中...</div>
          <div v-else-if="flatDayRecords.length === 0" class="empty-hint">当天没有任务记录。</div>
          <div v-else class="task-list">
            <article
              v-for="record in visibleDayRecords"
              :key="record.id"
              class="task-item"
              :class="{ completed: record.is_completed }"
            >
              <div class="task-main">
                <div class="task-title-wrap">
                  <div class="task-title-wrap-left">
                    <span class="task-check" :class="{ completed: record.is_completed }"></span>
                    <span class="task-title" :style="{ paddingLeft: `${record.level * 18}px` }">
                      {{ record.title }}
                    </span>
                  </div>
                  <span v-if="record.hasChildren" class="task-tag">父任务</span>
                  <span v-else-if="record.parent_id" class="task-tag">子任务</span>
                </div>
                <div class="task-meta">
                  <span class="task-meta-pill">{{ formatRecordDuration(record) }}</span>
                  <span class="task-meta-pill">难度 {{ record.difficulty_star }}</span>
                </div>
              </div>
            </article>
            <div v-if="hiddenRecordCount > 0" class="more-hint">还有 {{ hiddenRecordCount }} 项任务未展开</div>
          </div>
        </section>

        <section class="detail-section">
          <div class="section-label">成长账本</div>
          <div v-if="loadingDetail" class="empty-hint">加载中...</div>
          <div v-else-if="dayLedger.length === 0" class="empty-hint">当天还没有成长评分记录。</div>
          <div v-else class="ledger-list">
            <article
              v-for="entry in visibleDayLedger"
              :key="entry.id"
              class="ledger-card"
              :style="{ borderLeftColor: dimColors[entry.dimension_key] || 'var(--cyber-cyan)' }"
            >
              <div class="ledger-card-header">
                <div class="ledger-tags">
                  <span
                    class="dim-tag"
                    :style="{
                      color: dimColors[entry.dimension_key] || 'var(--cyber-cyan)',
                      borderColor: dimColors[entry.dimension_key] || 'var(--cyber-cyan)',
                      background: (dimColors[entry.dimension_key] || 'var(--cyber-cyan)') + '15',
                    }"
                  >
                    {{ entry.dimension_name || dimLabels[entry.dimension_key] || entry.dimension_key }}
                  </span>
                  <span class="value-tag">{{ formatChangeValue(entry.change_value) }}</span>
                </div>
                <div class="ledger-meta">
                  <span>{{ entry.date }}</span>
                  <span class="meta-engine">{{ formatEngineLabel(entry.engine) }}</span>
                </div>
              </div>
              <div class="ledger-title">{{ entry.source_title }}</div>
              <div class="ledger-reason">{{ entry.reason }}</div>
              <div class="ledger-actions">
                <NButton
                  secondary
                  type="error"
                  size="small"
                  :loading="rollingBackId === entry.id"
                  @click="handleRollback(entry.id)"
                >
                  撤销
                </NButton>
              </div>
            </article>
            <div v-if="hiddenLedgerCount > 0" class="more-hint">还有 {{ hiddenLedgerCount }} 条账本记录未展开</div>
          </div>
        </section>
      </div>

      <section class="detail-section archive-section">
        <div class="section-label">报告归档</div>
        <div v-if="loadingDetail" class="empty-hint">加载中...</div>
        <div v-else-if="reportHistoryCards.length === 0" class="empty-hint">这个日期还没有对应的塔罗或报告归档。</div>
        <div v-else class="report-history-grid">
          <article v-for="reportCard in visibleReportHistoryCards" :key="reportCard.key" class="report-card">
            <div class="report-card-head">
              <span class="plan-type">{{ reportCard.label }}</span>
              <span class="plan-state">{{ reportCard.report.start_date }} - {{ reportCard.report.end_date }}</span>
            </div>
            <div class="report-title">{{ reportCard.report.title }}</div>
            <div class="report-summary">{{ reportCard.report.summary }}</div>
            <div class="report-highlight">
              {{
                extractReportHighlight(reportCard.report.content_json as Record<string, unknown>) ||
                "这条归档以结构化 JSON 保存，可作为后续 AI 继续回看和递进的依据。"
              }}
            </div>
          </article>
          <div v-if="hiddenReportCount > 0" class="more-hint">还有 {{ hiddenReportCount }} 条报告归档未展开</div>
        </div>
      </section>

      <section class="detail-section archive-section">
        <div class="section-label">计划历史</div>
        <div v-if="loadingDetail" class="empty-hint">加载中...</div>
        <div v-else-if="planHistoryCards.length === 0" class="empty-hint">这个日期还没有对应的周计划或月计划落点。</div>
        <div v-else class="plan-history-grid">
          <article v-for="planCard in visiblePlanHistoryCards" :key="planCard.key" class="plan-card">
            <div class="plan-card-head">
              <div>
                <div class="plan-type">{{ planCard.label }}</div>
                <div class="plan-title">{{ planCard.cycle.title }}</div>
              </div>
              <span class="plan-state">{{ planCard.is_historical ? "历史" : "当前周期" }}</span>
            </div>
            <div class="plan-range">{{ planCard.cycle.start_date }} - {{ planCard.cycle.end_date }}</div>
            <div class="plan-summary">{{ planCard.cycle.ai_summary || planCard.cycle.summary || "暂无摘要。" }}</div>
            <div class="plan-metrics">
              <span class="detail-chip">总进度 {{ planCard.goal_progress.average_progress_percent }}%</span>
              <span class="detail-chip">
                已完成 {{ planCard.goal_progress.completed_items }}/{{ planCard.goal_progress.total_items }}
              </span>
            </div>
            <div class="plan-goals">
              <div v-if="planCard.items.length === 0" class="empty-hint small">这个周期还没有 Goal。</div>
              <div v-for="item in planCard.items.slice(0, 2)" :key="item.id" class="goal-row">
                <div class="goal-row-head">
                  <strong>{{ item.title }}</strong>
                  <span>{{ item.is_completed ? "100%" : `${item.progress_percent}%` }}</span>
                </div>
                <p>{{ item.description || item.ai_comment || "暂无说明。" }}</p>
              </div>
              <div v-if="planCard.items.length > 2" class="more-hint">还有 {{ planCard.items.length - 2 }} 条 Goal 未展开</div>
            </div>
          </article>
        </div>
      </section>
    </n-modal>
  </div>
</template>

<style scoped>
.cal-header,
.cal-panel {
  padding: 14px 16px;
}

.calendar-page {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 10px;
  height: 100%;
  min-height: 0;
  padding: 14px 18px 10px;
  overflow: hidden;
}

.calendar-page :deep(.cyber-page-title) {
  margin: 0;
  font-size: 30px;
}

.calendar-page :deep(.cyber-page-title .sub) {
  font-size: 16px;
  margin-left: 8px;
}

.cal-header {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) minmax(260px, 0.75fr) auto;
  gap: 12px;
  align-items: center;
}

.cal-nav-btn {
  width: 42px;
  height: 42px;
  border: 1px solid var(--cyber-border);
  background: rgba(0, 28, 60, 0.4);
  color: var(--cyber-cyan);
  cursor: pointer;
}

.cal-date-display,
.cal-summary {
  display: grid;
  gap: 4px;
}

.cal-year {
  color: var(--cyber-text-muted);
  letter-spacing: 2px;
  font-size: 12px;
}

.cal-month-row {
  display: flex;
  gap: 10px;
  align-items: baseline;
}

.cal-month-num {
  font-size: 44px;
  font-weight: 900;
  color: var(--cyber-text-primary);
  line-height: 1;
}

.cal-month-name,
.summary-label {
  color: var(--cyber-cyan);
  letter-spacing: 2px;
  font-weight: 700;
  font-size: 12px;
}

.summary-copy {
  color: var(--cyber-text-secondary);
  line-height: 1.35;
  font-size: 12px;
  max-width: 520px;
}

.cal-panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  height: min(68vh, calc(100vh - 170px));
  overflow: hidden;
}

.cal-weekdays,
.cal-days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}

.cal-weekdays {
  margin-bottom: 4px;
}

.cal-days {
  min-height: 0;
  grid-template-rows: repeat(6, minmax(0, 1fr));
  overflow: hidden;
}

.weekday-header {
  text-align: center;
  padding: 6px 4px;
  color: var(--cyber-text-muted);
  border-bottom: 1px solid var(--cyber-border);
  font-size: 11px;
  font-weight: 700;
}

.weekday-header.weekend {
  color: var(--cyber-cyan);
}

.cyber-cal-cell {
  aspect-ratio: auto;
  min-height: 0;
  height: 100%;
  padding: 4px 2px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(0, 12, 30, 0.6);
  border: 1px solid rgba(0, 60, 120, 0.15);
  position: relative;
  transition: all 0.2s;
}

.cyber-cal-cell.interactive {
  cursor: pointer;
}

.cyber-cal-cell:hover {
  border-color: var(--cyber-border);
  background: rgba(0, 40, 80, 0.3);
}

.cyber-cal-cell.today {
  border-color: var(--cyber-cyan);
  background: rgba(0, 180, 255, 0.08);
}

.cyber-cal-cell.has-record::before {
  content: "";
  position: absolute;
  top: 6px;
  right: 6px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--cyber-cyan);
}

.cyber-cal-cell.other-month {
  opacity: 0.25;
}

.day-num {
  font-size: 16px;
  font-weight: 700;
  color: var(--cyber-text-primary);
}

.day-status,
.today-label {
  margin-top: 3px;
  font-size: 9px;
}

.cyber-cal-cell.analyzed .day-status {
  color: var(--cyber-success);
}

.cyber-cal-cell:not(.analyzed) .day-status {
  color: var(--cyber-warning);
}

.day-markers {
  position: absolute;
  bottom: 4px;
  display: flex;
  gap: 4px;
}

.marker {
  width: 12px;
  height: 12px;
  display: grid;
  place-items: center;
  font-size: 8px;
  font-weight: 700;
  border-radius: 50%;
  background: rgba(0, 20, 48, 0.9);
  border: 1px solid rgba(0, 180, 255, 0.22);
}

.marker.week {
  color: var(--cyber-cyan);
}

.marker.month {
  color: var(--cyber-success);
}

.today-label {
  color: var(--cyber-cyan);
}

.detail-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}

.detail-date {
  font-size: 20px;
  font-weight: 800;
  color: var(--cyber-text-primary);
}

.detail-subtitle,
.empty-hint,
.plan-summary,
.goal-row p,
.ledger-reason,
.report-summary,
.report-highlight {
  color: var(--cyber-text-muted);
  line-height: 1.45;
  font-size: 12px;
}

.detail-summary,
.task-meta,
.ledger-tags,
.ledger-meta,
.plan-metrics {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.detail-chip,
.task-meta-pill,
.task-tag,
.meta-engine,
.plan-state {
  padding: 4px 10px;
  border: 1px solid rgba(0, 180, 255, 0.2);
  background: rgba(0, 30, 70, 0.32);
  color: var(--cyber-text-secondary);
  font-size: 12px;
}

.detail-chip.analyzed {
  color: var(--cyber-success);
}

.detail-chip.score {
  color: var(--cyber-cyan);
}

.detail-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px;
}

.detail-section {
  display: grid;
  gap: 8px;
}

.archive-section {
  margin-top: 12px;
}

.section-label {
  font-size: 13px;
  font-weight: 700;
  color: var(--cyber-cyan);
  border-left: 3px solid var(--cyber-cyan);
  padding-left: 10px;
}

.task-list,
.ledger-list,
.plan-history-grid,
.plan-goals,
.report-history-grid {
  display: grid;
  gap: 8px;
}

.task-item,
.ledger-card,
.plan-card,
.report-card {
  padding: 10px 12px;
  background: rgba(0, 14, 36, 0.56);
  border: 1px solid rgba(0, 180, 255, 0.12);
}

.task-item.completed {
  border-color: rgba(0, 255, 170, 0.2);
}

.task-title-wrap,
.plan-card-head,
.goal-row-head,
.report-card-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.task-title-wrap-left {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.task-check {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.28);
  flex-shrink: 0;
  margin-top: 6px;
}

.task-check.completed {
  border-color: var(--cyber-success);
  background: var(--cyber-success);
}

.task-main {
  display: grid;
  gap: 6px;
}

.task-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--cyber-text-primary);
  line-height: 1.35;
}

.task-item.completed .task-title {
  color: var(--cyber-text-secondary);
  text-decoration: line-through;
}

.ledger-card {
  border-left: 3px solid var(--cyber-cyan);
}

.value-tag {
  color: var(--cyber-success);
  font-weight: 700;
}

.ledger-title,
.plan-title,
.report-title {
  color: var(--cyber-text-primary);
  font-size: 15px;
  font-weight: 700;
}

.ledger-actions {
  display: flex;
  justify-content: flex-end;
}

.plan-history-grid,
.report-history-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.plan-type,
.plan-range {
  color: var(--cyber-cyan);
  font-size: 12px;
  letter-spacing: 1px;
}

.goal-row {
  padding: 8px 10px;
  border: 1px solid rgba(0, 180, 255, 0.14);
  background: rgba(0, 20, 48, 0.36);
}

.empty-hint.small {
  padding: 0;
}

.more-hint {
  color: var(--cyber-cyan);
  font-size: 12px;
  padding-top: 2px;
}

@media (max-width: 980px) {
  .cal-header,
  .detail-grid,
  .plan-history-grid,
  .report-history-grid {
    grid-template-columns: 1fr;
  }

  .cal-panel {
    height: min(64vh, calc(100vh - 210px));
  }
}

@media (max-width: 720px) {
  .cal-month-num {
    font-size: 30px;
  }

  .cyber-cal-cell {
    padding: 3px 1px;
  }

  .day-num {
    font-size: 14px;
  }
}
</style>
