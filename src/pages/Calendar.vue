<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NDataTable, NModal } from "naive-ui";
import {
  getCalendarOverview,
  getLedgerByDate,
  getRecordsByDate,
  type CalendarOverviewItem,
  type RecordItem,
} from "@/api/client/tauriCommands";
import { format } from "date-fns";

const currentYear = ref(new Date().getFullYear());
const currentMonth = ref(new Date().getMonth());
const selectedDate = ref("");
const showDetailModal = ref(false);
const dayRecords = ref<RecordItem[]>([]);
const dayLedger = ref<
  { id: number; dimension_key: string; change_value: number; source_title: string; reason: string }[]
>([]);
const monthOverview = ref<Record<string, CalendarOverviewItem>>({});

const calendarDays = computed(() => {
  const firstDay = new Date(currentYear.value, currentMonth.value, 1);
  const lastDay = new Date(currentYear.value, currentMonth.value + 1, 0);
  const startWeekday = firstDay.getDay();
  const days: { date: number; dateStr: string; inMonth: boolean }[] = [];

  const prevLastDay = new Date(currentYear.value, currentMonth.value, 0).getDate();
  for (let index = startWeekday - 1; index >= 0; index--) {
    days.push({ date: prevLastDay - index, dateStr: "", inMonth: false });
  }

  for (let date = 1; date <= lastDay.getDate(); date++) {
    const dateStr = format(new Date(currentYear.value, currentMonth.value, date), "yyyy-MM-dd");
    days.push({ date, dateStr, inMonth: true });
  }

  const remaining = 42 - days.length;
  for (let date = 1; date <= remaining; date++) {
    days.push({ date, dateStr: "", inMonth: false });
  }

  return days;
});

const recordColumns = [
  { title: "任务", key: "title" },
  { title: "时长", key: "minutes", width: 80 },
  { title: "难度", key: "difficulty_star", width: 80 },
];

const ledgerColumns = [
  { title: "维度", key: "dimension_key", width: 80 },
  { title: "变化", key: "change_value", width: 60 },
  { title: "来源", key: "source_title" },
];

watch(
  [currentYear, currentMonth],
  () => {
    void loadOverview();
  },
  { immediate: true }
);

async function loadOverview() {
  const overview = await getCalendarOverview(currentYear.value, currentMonth.value + 1);
  monthOverview.value = Object.fromEntries(overview.map((item) => [item.date, item]));
}

function prevMonth() {
  if (currentMonth.value === 0) {
    currentMonth.value = 11;
    currentYear.value -= 1;
  } else {
    currentMonth.value -= 1;
  }
}

function nextMonth() {
  if (currentMonth.value === 11) {
    currentMonth.value = 0;
    currentYear.value += 1;
  } else {
    currentMonth.value += 1;
  }
}

async function selectDay(dateStr: string) {
  if (!dateStr) {
    return;
  }

  selectedDate.value = dateStr;
  dayRecords.value = await getRecordsByDate(dateStr);
  dayLedger.value = await getLedgerByDate(dateStr);
  showDetailModal.value = true;
}

function getDayOverview(dateStr: string) {
  return monthOverview.value[dateStr];
}

function isToday(dateStr: string): boolean {
  return dateStr === format(new Date(), "yyyy-MM-dd");
}

const weekDays = ['SUN 日', 'MON 一', 'TUE 二', 'WED 三', 'THU 四', 'FRI 五', 'SAT 六'];
const monthNames = ['JANUARY', 'FEBRUARY', 'MARCH', 'APRIL', 'MAY', 'JUNE', 'JULY', 'AUGUST', 'SEPTEMBER', 'OCTOBER', 'NOVEMBER', 'DECEMBER'];
</script>

<template>
  <div class="cyber-page">
    <h1 class="cyber-page-title">
      CALENDAR<span class="sub">日历</span>
    </h1>

    <!-- Calendar Header -->
    <div class="cal-header cyber-panel">
      <button class="cal-nav-btn" @click="prevMonth">◀</button>
      <div class="cal-date-display">
        <div class="cal-year">{{ currentYear }}</div>
        <div class="cal-month-row">
          <span class="cal-month-num">{{ String(currentMonth + 1).padStart(2, '0') }}</span>
          <span class="cal-month-name">{{ monthNames[currentMonth] }}</span>
        </div>
      </div>
      <div class="cal-lunar">
        <div class="lunar-label">LUNAR PHASE 月相</div>
        <div class="lunar-phase">WAXING CRESCENT</div>
        <div class="lunar-cn">娥眉月</div>
        <div class="lunar-icon">🌙</div>
      </div>
      <button class="cal-nav-btn" @click="nextMonth">▶</button>
    </div>

    <!-- Calendar Grid -->
    <div class="cal-panel cyber-panel" style="margin-top: 20px;">
      <!-- Weekday Headers -->
      <div class="cal-weekdays">
        <div
          v-for="wd in weekDays"
          :key="wd"
          class="weekday-header"
          :class="{ weekend: wd.startsWith('SUN') || wd.startsWith('SAT') }"
        >
          {{ wd }}
        </div>
      </div>
      <!-- Days Grid -->
      <div class="cal-days">
        <div
          v-for="(day, idx) in calendarDays"
          :key="idx"
          class="cyber-cal-cell"
          :class="{
            'other-month': !day.inMonth,
            'today': day.inMonth && isToday(day.dateStr),
            'has-record': day.inMonth && getDayOverview(day.dateStr),
            'analyzed': day.inMonth && getDayOverview(day.dateStr)?.is_analyzed,
            'weekend': idx % 7 === 0 || idx % 7 === 6,
            interactive: day.inMonth,
          }"
          @click="day.inMonth && selectDay(day.dateStr)"
        >
          <span class="day-num">{{ day.date }}</span>
          <span v-if="day.inMonth && getDayOverview(day.dateStr)" class="day-status">
            {{ getDayOverview(day.dateStr)?.is_analyzed ? '已评' : '待评' }}
          </span>
          <span v-if="day.inMonth && isToday(day.dateStr)" class="today-label">TODAY</span>
        </div>
      </div>
    </div>

    <!-- Detail Modal -->
    <n-modal
      v-model:show="showDetailModal"
      :title="`${selectedDate} 详情`"
      preset="card"
      style="width: 520px"
      class="cyber-modal"
    >
      <div class="modal-form">
        <div class="modal-section">
          <div class="section-label">任务记录</div>
          <n-data-table :columns="recordColumns" :data="dayRecords" size="small" :single-line="false" />
          <div v-if="dayRecords.length === 0" class="empty-hint">当日无任务</div>
        </div>
        <div class="modal-section">
          <div class="section-label">成长账本</div>
          <n-data-table :columns="ledgerColumns" :data="dayLedger" size="small" :single-line="false" />
          <div v-if="dayLedger.length === 0" class="empty-hint">当日无评分记录</div>
        </div>
      </div>
    </n-modal>
  </div>
</template>

<style scoped>
.cal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 28px;
  gap: 24px;
}

.cal-nav-btn {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 60, 120, 0.2);
  border: 1px solid var(--cyber-border);
  border-radius: 4px;
  color: var(--cyber-cyan);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.25s;
}

.cal-nav-btn:hover {
  background: rgba(0, 180, 255, 0.15);
  border-color: var(--cyber-cyan);
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.2);
}

.cal-date-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.cal-year {
  font-size: 16px;
  font-weight: 700;
  color: var(--cyber-text-secondary);
  letter-spacing: 3px;
}

.cal-month-row {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.cal-month-num {
  font-size: 64px;
  font-weight: 900;
  font-style: italic;
  color: var(--cyber-text-primary);
  line-height: 1;
  text-shadow: 0 0 24px rgba(0, 212, 255, 0.25);
}

.cal-month-name {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 4px;
  color: var(--cyber-cyan);
}

.cal-lunar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 10px 24px;
  background: rgba(0, 20, 50, 0.4);
  border: 1px solid var(--cyber-border);
  border-radius: 4px;
  position: relative;
}

.lunar-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 2px;
  color: var(--cyber-text-muted);
}

.lunar-phase {
  font-size: 14px;
  font-weight: 700;
  color: var(--cyber-cyan);
  letter-spacing: 2px;
}

.lunar-cn {
  font-size: 12px;
  color: var(--cyber-text-secondary);
}

.lunar-icon {
  position: absolute;
  right: -12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 32px;
  filter: drop-shadow(0 0 8px rgba(0, 212, 255, 0.4));
}

/* Calendar Grid */
.cal-panel {
  padding: 16px;
}

.cal-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
  margin-bottom: 4px;
}

.weekday-header {
  text-align: center;
  padding: 10px 4px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--cyber-text-muted);
  border-bottom: 1px solid var(--cyber-border);
}

.weekday-header.weekend {
  color: var(--cyber-cyan);
}

.cal-days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}

.cyber-cal-cell {
  aspect-ratio: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(0, 12, 30, 0.6);
  border: 1px solid rgba(0, 60, 120, 0.15);
  transition: all 0.2s;
  position: relative;
  min-height: 60px;
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
  box-shadow: inset 0 0 16px rgba(0, 212, 255, 0.08);
}

.cyber-cal-cell.has-record {
  border-color: rgba(0, 180, 255, 0.2);
}

.cyber-cal-cell.has-record::before {
  content: '';
  position: absolute;
  top: 6px;
  right: 6px;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--cyber-cyan);
  box-shadow: 0 0 4px var(--cyber-cyan);
}

.cyber-cal-cell.other-month {
  opacity: 0.25;
  cursor: default;
}

.cyber-cal-cell.weekend .day-num {
  color: var(--cyber-cyan);
}

.day-num {
  font-size: 20px;
  font-weight: 700;
  font-style: italic;
  color: var(--cyber-text-primary);
  line-height: 1;
}

.day-status {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 2px;
  margin-top: 4px;
  letter-spacing: 1px;
}

.cyber-cal-cell.analyzed .day-status {
  color: var(--cyber-success);
  background: rgba(0, 255, 170, 0.1);
}

.cyber-cal-cell:not(.analyzed) .day-status {
  color: var(--cyber-warning);
  background: rgba(255, 204, 0, 0.1);
}

.today-label {
  font-size: 9px;
  color: var(--cyber-cyan);
  margin-top: 2px;
  letter-spacing: 1px;
}

/* Modal */
.modal-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.modal-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-label {
  font-size: 13px;
  font-weight: 700;
  color: var(--cyber-cyan);
  letter-spacing: 1px;
  border-left: 3px solid var(--cyber-cyan);
  padding-left: 10px;
}

.empty-hint {
  color: var(--cyber-text-muted);
  padding: 12px 0;
  font-size: 13px;
  text-align: center;
  background: rgba(0, 20, 50, 0.2);
  border-radius: 4px;
}
</style>
