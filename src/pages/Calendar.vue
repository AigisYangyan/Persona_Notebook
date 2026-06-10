<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NButton, NCard, NDataTable, NModal, NSpace } from "naive-ui";
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

const monthLabel = computed(() => {
  return format(new Date(currentYear.value, currentMonth.value), "yyyy年MM月");
});

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
</script>

<template>
  <div>
    <h1 style="margin-top: 0">日历</h1>
    <n-card>
      <n-space justify="space-between" align="center" style="margin-bottom: 16px">
        <n-button @click="prevMonth">&lt;</n-button>
        <strong>{{ monthLabel }}</strong>
        <n-button @click="nextMonth">&gt;</n-button>
      </n-space>

      <div class="calendar-grid">
        <div v-for="weekday in ['日', '一', '二', '三', '四', '五', '六']" :key="weekday" class="weekday">
          {{ weekday }}
        </div>
        <div
          v-for="(day, idx) in calendarDays"
          :key="idx"
          class="day-cell"
          :class="{ 'other-month': !day.inMonth, clickable: day.inMonth }"
          @click="day.inMonth && selectDay(day.dateStr)"
        >
          <span class="day-num">{{ day.date }}</span>
          <template v-if="day.inMonth && getDayOverview(day.dateStr)">
            <span class="record-dot" />
            <span
              class="analyzed-badge"
              :class="{ done: getDayOverview(day.dateStr)?.is_analyzed }"
            >
              {{ getDayOverview(day.dateStr)?.is_analyzed ? "已评" : "待评" }}
            </span>
          </template>
        </div>
      </div>
    </n-card>

    <n-modal v-model:show="showDetailModal" :title="`${selectedDate} 详情`" preset="card" style="width: 500px">
      <n-space vertical>
        <div>
          <strong>任务记录</strong>
          <n-data-table :columns="recordColumns" :data="dayRecords" size="small" :single-line="false" />
          <div v-if="dayRecords.length === 0" style="color: #999; padding: 8px 0">当日无任务</div>
        </div>
        <div>
          <strong>成长账本</strong>
          <n-data-table :columns="ledgerColumns" :data="dayLedger" size="small" :single-line="false" />
          <div v-if="dayLedger.length === 0" style="color: #999; padding: 8px 0">当日无评分记录</div>
        </div>
      </n-space>
    </n-modal>
  </div>
</template>

<style scoped>
.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}

.weekday {
  text-align: center;
  font-weight: bold;
  padding: 8px;
  color: #666;
}

.day-cell {
  aspect-ratio: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #eee;
  border-radius: 4px;
  position: relative;
}

.day-cell.clickable {
  cursor: pointer;
}

.day-cell.clickable:hover {
  background: #f0f0f0;
}

.day-cell.other-month {
  color: #ccc;
  border-color: transparent;
}

.day-num {
  font-size: 14px;
}

.record-dot {
  position: absolute;
  bottom: 8px;
  left: 8px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #2080f0;
}

.analyzed-badge {
  position: absolute;
  right: 6px;
  bottom: 6px;
  font-size: 11px;
  color: #d03050;
}

.analyzed-badge.done {
  color: #18a058;
}
</style>
