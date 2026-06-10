import { ref } from "vue";
import { defineStore } from "pinia";
import {
  confirmScorePreview,
  createRecord,
  deleteRecord,
  getRecordsByDate,
  previewScoreWithLocalRules,
  type RecordItem,
  type ScorePreviewItem,
} from "@/api/client/tauriCommands";
import { OpenAIAdapter } from "@/api/adapter/openaiAdapter";
import { buildDailyAnalysisRequest } from "@/features/scoring/requestBuilder";
import { normalizeApiPreview } from "@/features/scoring/preview";
import type { ScoringEngine } from "@/stores/settingStore";
import { getTodayStr } from "@/utils/date";

export const useRecordStore = defineStore("record", () => {
  const records = ref<RecordItem[]>([]);
  const currentDate = ref(getTodayStr());
  const loading = ref(false);
  const pendingPreview = ref<ScorePreviewItem[]>([]);
  const pendingSummary = ref("");
  const pendingEngine = ref<ScoringEngine | null>(null);

  async function fetchRecords() {
    loading.value = true;
    try {
      records.value = await getRecordsByDate(currentDate.value);
    } finally {
      loading.value = false;
    }
  }

  async function addRecord(title: string, minutes: number, difficultyStar: number) {
    const newRecord = await createRecord(currentDate.value, title, minutes, difficultyStar);
    records.value = [...records.value, newRecord];
  }

  async function removeRecord(id: number) {
    await deleteRecord(id);
    records.value = records.value.filter((record) => record.id !== id);
  }

  async function analyzeToday() {
    if (records.value.length === 0) {
      clearPendingPreview();
      return;
    }

    const rulePreview = await previewScoreWithLocalRules(
      records.value.map((record) => ({
        id: record.id,
        title: record.title,
        minutes: record.minutes,
        difficulty_star: record.difficulty_star,
      }))
    );
    const request = buildDailyAnalysisRequest(currentDate.value, records.value, rulePreview);
    const adapter = new OpenAIAdapter();
    const response = await adapter.score(request);
    const preview = normalizeApiPreview(request, response, records.value);
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

  return {
    records,
    currentDate,
    pendingPreview,
    pendingSummary,
    pendingEngine,
    loading,
    fetchRecords,
    addRecord,
    removeRecord,
    analyzeToday,
    confirmPendingPreview,
    clearPendingPreview,
  };
});
