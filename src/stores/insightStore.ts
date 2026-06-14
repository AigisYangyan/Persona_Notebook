import { ref } from "vue";
import { defineStore } from "pinia";
import {
  deleteInsightReport,
  generatePeriodReport,
  generateTarotInsight,
  getInsightContextSnapshot,
  listInsightReports,
  type InsightContextSnapshot,
  type InsightPeriodType,
  type InsightReport,
} from "@/api/client/tauriCommands";

export const useInsightStore = defineStore("insight", () => {
  const currentTarot = ref<InsightReport | null>(null);
  const currentReport = ref<InsightReport | null>(null);
  const tarotCabinet = ref<InsightReport[]>([]);
  const reportCabinet = ref<InsightReport[]>([]);
  const contextSnapshot = ref<InsightContextSnapshot | null>(null);
  const loading = ref(false);
  const generating = ref(false);

  async function loadTarotCabinet(limit = 12) {
    loading.value = true;
    try {
      tarotCabinet.value = await listInsightReports("tarot", "day", limit);
    } finally {
      loading.value = false;
    }
  }

  async function loadReportCabinet(periodType?: InsightPeriodType | null, limit = 12) {
    loading.value = true;
    try {
      reportCabinet.value = await listInsightReports("report", periodType ?? null, limit);
    } finally {
      loading.value = false;
    }
  }

  async function generateTarot(date: string) {
    generating.value = true;
    try {
      currentTarot.value = await generateTarotInsight(date);
      await loadTarotCabinet();
    } finally {
      generating.value = false;
    }
  }

  async function generateReport(periodType: InsightPeriodType, anchorDate: string) {
    generating.value = true;
    try {
      currentReport.value = await generatePeriodReport(periodType, anchorDate);
      await loadReportCabinet(periodType);
    } finally {
      generating.value = false;
    }
  }

  async function loadContextSnapshot(snapshotId: number) {
    contextSnapshot.value = await getInsightContextSnapshot(snapshotId);
  }

  async function removeReport(
    reportId: number,
    reportKind: "tarot" | "report",
    periodType?: InsightPeriodType | null
  ) {
    await deleteInsightReport(reportId);
    if (reportKind === "tarot") {
      tarotCabinet.value = tarotCabinet.value.filter((report) => report.id !== reportId);
      currentTarot.value =
        currentTarot.value?.id === reportId ? tarotCabinet.value[0] ?? null : currentTarot.value;
      await loadTarotCabinet();
      currentTarot.value =
        currentTarot.value?.id === reportId ? tarotCabinet.value[0] ?? null : currentTarot.value;
      return;
    }

    reportCabinet.value = reportCabinet.value.filter((report) => report.id !== reportId);
    currentReport.value =
      currentReport.value?.id === reportId ? reportCabinet.value[0] ?? null : currentReport.value;
    await loadReportCabinet(periodType ?? null);
    currentReport.value =
      currentReport.value?.id === reportId ? reportCabinet.value[0] ?? null : currentReport.value;
  }

  function selectTarot(report: InsightReport) {
    currentTarot.value = report;
  }

  function selectReport(report: InsightReport) {
    currentReport.value = report;
  }

  function clearContextSnapshot() {
    contextSnapshot.value = null;
  }

  return {
    currentTarot,
    currentReport,
    tarotCabinet,
    reportCabinet,
    contextSnapshot,
    loading,
    generating,
    loadTarotCabinet,
    loadReportCabinet,
    generateTarot,
    generateReport,
    loadContextSnapshot,
    removeReport,
    selectTarot,
    selectReport,
    clearContextSnapshot,
  };
});
