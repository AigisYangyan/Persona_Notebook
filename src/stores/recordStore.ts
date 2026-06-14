import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  confirmScorePreview,
  createRecord,
  createManualRecordEntry,
  createSubRecord,
  deleteRecord,
  getRecordsByDate,
  pauseRecordTimer,
  previewScoreWithLocalRules,
  resetRecordTimer,
  startRecordTimer,
  toggleRecordCompleted,
  type RecordItem,
  type ScorePreviewItem,
  type TimerMode,
  updateRecordTimerMode,
} from "@/api/client/tauriCommands";
import { OpenAIAdapter } from "@/api/adapter/openaiAdapter";
import { buildDailyAnalysisRequest } from "@/features/scoring/requestBuilder";
import { normalizeApiPreview } from "@/features/scoring/preview";
import {
  buildTaskTimeChartItems,
  buildTaskTree,
  getDisplayMinutes,
  isCountdownExpired,
} from "@/features/records/taskMetrics";
import type { ScoringEngine } from "@/stores/settingStore";
import { getTodayStr } from "@/utils/date";

const DEFAULT_COUNTDOWN_SECONDS = 25 * 60;

export const useRecordStore = defineStore("record", () => {
  const records = ref<RecordItem[]>([]);
  const currentDate = ref(getTodayStr());
  const loading = ref(false);
  const pendingPreview = ref<ScorePreviewItem[]>([]);
  const pendingSummary = ref("");
  const pendingEngine = ref<ScoringEngine | null>(null);
  const nowMs = ref(Date.now());
  const autoPausingIds = new Set<number>();
  let tickHandle: number | null = null;

  const taskTree = computed(() => buildTaskTree(records.value, nowMs.value));
  const chartItems = computed(() => buildTaskTimeChartItems(records.value, nowMs.value));
  const runningRecord = computed(
    () => records.value.find((record) => Boolean(record.timer_started_at)) ?? null
  );
  const totalTrackedSeconds = computed(() =>
    chartItems.value.reduce((sum, item) => sum + item.value, 0)
  );
  const completedCount = computed(
    () => records.value.filter((record) => record.is_completed).length
  );

  async function fetchRecords() {
    loading.value = true;
    try {
      records.value = await getRecordsByDate(currentDate.value);
      syncNow();
    } finally {
      loading.value = false;
    }
  }

  async function addRecord(
    title: string,
    difficultyStar: number,
    timerMode: TimerMode,
    countdownTargetSeconds: number | null
  ) {
    await createRecord(
      currentDate.value,
      title,
      difficultyStar,
      timerMode,
      timerMode === "countdown"
        ? countdownTargetSeconds ?? DEFAULT_COUNTDOWN_SECONDS
        : null
    );
    await fetchRecords();
  }

  async function addSubRecord(
    parentId: number,
    title: string,
    difficultyStar: number,
    timerMode: TimerMode,
    countdownTargetSeconds: number | null
  ) {
    await createSubRecord(
      parentId,
      title,
      difficultyStar,
      timerMode,
      timerMode === "countdown"
        ? countdownTargetSeconds ?? DEFAULT_COUNTDOWN_SECONDS
        : null
    );
    await fetchRecords();
  }

  async function addManualRecord(
    title: string,
    minutes: number,
    difficultyStar: number,
    parentId?: number | null
  ) {
    await createManualRecordEntry({
      date: currentDate.value,
      title,
      minutes,
      difficultyStar,
      parentId: parentId ?? null,
    });
    await fetchRecords();
  }

  async function removeRecord(id: number) {
    await deleteRecord(id);
    await fetchRecords();
  }

  async function startTimer(id: number) {
    await startRecordTimer(id);
    await fetchRecords();
  }

  async function pauseTimer(id: number) {
    await pauseRecordTimer(id);
    await fetchRecords();
  }

  async function resetTimer(id: number) {
    await resetRecordTimer(id);
    await fetchRecords();
  }

  async function setCompleted(id: number, isCompleted: boolean) {
    await toggleRecordCompleted(id, isCompleted);
    await fetchRecords();
  }

  async function saveTimerMode(
    id: number,
    timerMode: TimerMode,
    countdownTargetSeconds: number | null
  ) {
    await updateRecordTimerMode(
      id,
      timerMode,
      timerMode === "countdown"
        ? countdownTargetSeconds ?? DEFAULT_COUNTDOWN_SECONDS
        : null
    );
    await fetchRecords();
  }

  async function analyzeToday() {
    await pauseRunningTimers();
    if (records.value.length === 0) {
      clearPendingPreview();
      return;
    }

    const analyzableRecords = records.value
      .map((record) => ({
        ...record,
        minutes: getDisplayMinutes(record, nowMs.value),
      }))
      .filter((record) => record.minutes > 0);

    if (analyzableRecords.length === 0) {
      clearPendingPreview();
      throw new Error("当前没有可分析的计时记录");
    }

    const rulePreview = await previewScoreWithLocalRules(
      analyzableRecords.map((record) => ({
        id: record.id,
        title: record.title,
        minutes: record.minutes,
        difficulty_star: record.difficulty_star,
      }))
    );
    const request = buildDailyAnalysisRequest(currentDate.value, analyzableRecords, rulePreview);
    const adapter = new OpenAIAdapter();
    const response = await adapter.score(request);
    const preview = normalizeApiPreview(request, response, analyzableRecords);
    pendingPreview.value = preview.items;
    pendingSummary.value = preview.summary;
    pendingEngine.value = "rules_api";
  }

  async function confirmPendingPreview() {
    if (pendingPreview.value.length === 0) {
      throw new Error("当前没有待确认的成长预览");
    }

    await confirmScorePreview(
      currentDate.value,
      pendingPreview.value,
      pendingSummary.value || null
    );
    clearPendingPreview();
  }

  function clearPendingPreview() {
    pendingPreview.value = [];
    pendingSummary.value = "";
    pendingEngine.value = null;
  }

  function startClock() {
    if (tickHandle !== null) {
      return;
    }
    syncNow();
    tickHandle = window.setInterval(() => {
      syncNow();
      void pauseExpiredCountdowns();
    }, 1000);
  }

  function stopClock() {
    if (tickHandle !== null) {
      window.clearInterval(tickHandle);
      tickHandle = null;
    }
  }

  function syncNow() {
    nowMs.value = Date.now();
  }

  async function pauseRunningTimers() {
    const runningIds = records.value
      .filter((record) => Boolean(record.timer_started_at))
      .map((record) => record.id);

    for (const id of runningIds) {
      await pauseRecordTimer(id);
    }

    if (runningIds.length > 0) {
      await fetchRecords();
    }
  }

  async function pauseExpiredCountdowns() {
    const expired = records.value.filter((record) => {
      if (autoPausingIds.has(record.id)) {
        return false;
      }
      return isCountdownExpired(record, nowMs.value);
    });

    if (expired.length === 0) {
      return;
    }

    expired.forEach((record) => autoPausingIds.add(record.id));
    try {
      for (const record of expired) {
        await pauseRecordTimer(record.id);
      }
      await fetchRecords();
    } finally {
      expired.forEach((record) => autoPausingIds.delete(record.id));
    }
  }

  return {
    records,
    currentDate,
    pendingPreview,
    pendingSummary,
    pendingEngine,
    loading,
    nowMs,
    taskTree,
    chartItems,
    runningRecord,
    totalTrackedSeconds,
    completedCount,
    fetchRecords,
    addRecord,
    addSubRecord,
    addManualRecord,
    removeRecord,
    startTimer,
    pauseTimer,
    resetTimer,
    setCompleted,
    saveTimerMode,
    analyzeToday,
    confirmPendingPreview,
    clearPendingPreview,
    pauseRunningTimers,
    startClock,
    stopClock,
  };
});
