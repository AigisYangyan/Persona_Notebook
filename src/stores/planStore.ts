import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  applyPlanAiUpdate,
  deletePlanItem,
  getLatestPlanAiOutcome,
  getPlanAiOutcome,
  getMonthPlan,
  getWeekPlan,
  refreshPlanProgress,
  savePlanCycle,
  savePlanItem,
  submitPlanAiAnswers,
  type PlanAiOutcome,
  type PlanPeriodType,
  type PlanSnapshot,
} from "@/api/client/tauriCommands";
import { normalizeAnchorDate } from "@/features/plans/periods";

export const usePlanStore = defineStore("plan", () => {
  const weekAnchorDate = ref(normalizeAnchorDate());
  const monthAnchorDate = ref(normalizeAnchorDate());
  const weekPlan = ref<PlanSnapshot | null>(null);
  const monthPlan = ref<PlanSnapshot | null>(null);
  const loading = ref(false);
  const saving = ref(false);
  const aiLoading = ref(false);
  const aiOutcome = ref<PlanAiOutcome | null>(null);
  const aiPeriod = ref<PlanPeriodType | null>(null);
  const aiAnswers = ref<string[]>([]);

  const currentWeekPlan = computed(() => weekPlan.value);
  const currentMonthPlan = computed(() => monthPlan.value);

  async function loadWeekPlan(anchorDate = normalizeAnchorDate()) {
    loading.value = true;
    try {
      weekAnchorDate.value = anchorDate;
      weekPlan.value = await getWeekPlan(anchorDate);
    } finally {
      loading.value = false;
    }
  }

  async function loadMonthPlan(anchorDate = normalizeAnchorDate()) {
    loading.value = true;
    try {
      monthAnchorDate.value = anchorDate;
      monthPlan.value = await getMonthPlan(anchorDate);
    } finally {
      loading.value = false;
    }
  }

  async function saveCycle(periodType: PlanPeriodType, title: string, summary: string) {
    saving.value = true;
    try {
      const anchorDate = getAnchorDate(periodType);
      const snapshot = await savePlanCycle(periodType, anchorDate, title, summary);
      setSnapshot(periodType, snapshot);
    } finally {
      saving.value = false;
    }
  }

  async function saveItem(periodType: PlanPeriodType, payload: {
    cycleId: number;
    itemId?: number | null;
    title: string;
    description: string;
    dimensionKey?: string | null;
    progressPercent?: number | null;
    aiComment?: string | null;
    sortOrder?: number | null;
    isCompleted: boolean;
  }) {
    saving.value = true;
    try {
      const snapshot = await savePlanItem(payload);
      setSnapshot(periodType, snapshot);
    } finally {
      saving.value = false;
    }
  }

  async function removeItem(periodType: PlanPeriodType, cycleId: number, itemId: number) {
    saving.value = true;
    try {
      const snapshot = await deletePlanItem(cycleId, itemId);
      setSnapshot(periodType, snapshot);
    } finally {
      saving.value = false;
    }
  }

  async function refreshWithAi(periodType: PlanPeriodType) {
    aiLoading.value = true;
    try {
      const outcome = await refreshPlanProgress(periodType, getAnchorDate(periodType));
      aiOutcome.value = outcome;
      aiPeriod.value = periodType;
      aiAnswers.value = outcome.questions.map(() => "");
      return outcome;
    } finally {
      aiLoading.value = false;
    }
  }

  async function submitAiAnswers() {
    if (!aiOutcome.value) {
      throw new Error("No active AI session");
    }
    aiLoading.value = true;
    try {
      const outcome = await submitPlanAiAnswers(aiOutcome.value.session_id, aiAnswers.value);
      aiOutcome.value = outcome;
      aiAnswers.value = outcome.questions.map(() => "");
    } finally {
      aiLoading.value = false;
    }
  }

  async function applyAiProposal() {
    if (!aiOutcome.value || !aiPeriod.value) {
      throw new Error("No AI proposal to apply");
    }
    aiLoading.value = true;
    try {
      const snapshot = await applyPlanAiUpdate(aiOutcome.value.session_id);
      setSnapshot(aiPeriod.value, snapshot);
      clearAiState();
      return snapshot;
    } finally {
      aiLoading.value = false;
    }
  }

  function clearAiState() {
    aiOutcome.value = null;
    aiPeriod.value = null;
    aiAnswers.value = [];
  }

  function setAiOutcome(periodType: PlanPeriodType, outcome: PlanAiOutcome) {
    aiOutcome.value = outcome;
    aiPeriod.value = periodType;
    aiAnswers.value = outcome.questions.map(() => "");
  }

  async function loadAiOutcome(sessionId: number, periodType: PlanPeriodType) {
    aiLoading.value = true;
    try {
      const outcome = await getPlanAiOutcome(sessionId);
      setAiOutcome(periodType, outcome);
      return outcome;
    } finally {
      aiLoading.value = false;
    }
  }

  async function restoreLatestAiOutcome(periodType: PlanPeriodType, anchorDate?: string) {
    aiLoading.value = true;
    try {
      const outcome = await getLatestPlanAiOutcome(periodType, anchorDate ?? getAnchorDate(periodType));
      if (!outcome) {
        clearAiState();
        return null;
      }
      setAiOutcome(periodType, outcome);
      return outcome;
    } finally {
      aiLoading.value = false;
    }
  }

  function setAiAnswer(index: number, value: string) {
    aiAnswers.value = aiAnswers.value.map((answer, currentIndex) =>
      currentIndex === index ? value : answer
    );
  }

  function getAnchorDate(periodType: PlanPeriodType): string {
    return periodType === "week" ? weekAnchorDate.value : monthAnchorDate.value;
  }

  function setSnapshot(periodType: PlanPeriodType, snapshot: PlanSnapshot) {
    if (periodType === "week") {
      weekPlan.value = snapshot;
      weekAnchorDate.value = snapshot.cycle.start_date;
      return;
    }
    monthPlan.value = snapshot;
    monthAnchorDate.value = snapshot.cycle.start_date;
  }

  return {
    weekAnchorDate,
    monthAnchorDate,
    weekPlan,
    monthPlan,
    currentWeekPlan,
    currentMonthPlan,
    loading,
    saving,
    aiLoading,
    aiOutcome,
    aiPeriod,
    aiAnswers,
    loadWeekPlan,
    loadMonthPlan,
    saveCycle,
    saveItem,
    removeItem,
    refreshWithAi,
    submitAiAnswers,
    applyAiProposal,
    clearAiState,
    setAiOutcome,
    setAiAnswer,
    loadAiOutcome,
    restoreLatestAiOutcome,
  };
});
