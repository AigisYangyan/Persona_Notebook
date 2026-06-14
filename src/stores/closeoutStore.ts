import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  runGlobalCloseout,
  type CloseoutStep,
  type CloseoutStepStatus,
  type GlobalCloseoutResult,
  type PlanAiOutcome,
} from "@/api/client/tauriCommands";
import { useInsightStore } from "@/stores/insightStore";
import { usePlanStore } from "@/stores/planStore";
import { useRecordStore } from "@/stores/recordStore";
import { useStatStore } from "@/stores/statStore";
import { getTodayStr } from "@/utils/date";

const INITIAL_STEP: CloseoutStep = {
  status: "pending",
  message: "等待执行",
  report_id: null,
  session_id: null,
  questions: [],
};

export interface DailyCloseoutResultView {
  score: CloseoutStep;
  dailyReport: CloseoutStep;
  weekPlan: CloseoutStep;
  monthPlan: CloseoutStep;
  closeoutRunId: number | null;
}

export type CloseoutScope = "day" | "week" | "month" | "all";

export const useCloseoutStore = defineStore("closeout", () => {
  const running = ref(false);
  const result = ref<DailyCloseoutResultView>(createInitialResult());

  const hasResult = computed(() =>
    [result.value.score, result.value.dailyReport, result.value.weekPlan, result.value.monthPlan].some(
      (step) => step.status !== "pending"
    )
  );

  async function run(date = getTodayStr(), scope: CloseoutScope = "all") {
    running.value = true;
    result.value = createInitialResult();

    try {
      const closeout = await runGlobalCloseout(date, scope);
      await syncCloseoutState(closeout, date);
      result.value = mapCloseoutResult(closeout);
      return result.value;
    } finally {
      running.value = false;
    }
  }

  return {
    running,
    result,
    hasResult,
    run,
  };
});

async function syncCloseoutState(closeout: GlobalCloseoutResult, date: string) {
  const planStore = usePlanStore();
  const statStore = useStatStore();
  const recordStore = useRecordStore();
  const insightStore = useInsightStore();

  applyClarificationState(planStore, "week", closeout.week_plan);
  applyClarificationState(planStore, "month", closeout.month_plan);

  await Promise.all([
    statStore.refreshStats(),
    recordStore.fetchRecords(),
    planStore.loadWeekPlan(date),
    planStore.loadMonthPlan(date),
    insightStore.loadReportCabinet("day"),
    insightStore.loadReportCabinet("week"),
    insightStore.loadReportCabinet("month"),
    insightStore.loadTarotCabinet(),
  ]);
}

function mapCloseoutResult(closeout: GlobalCloseoutResult): DailyCloseoutResultView {
  return {
    score: closeout.score,
    dailyReport: closeout.report,
    weekPlan: closeout.week_plan,
    monthPlan: closeout.month_plan,
    closeoutRunId: closeout.closeout_run_id,
  };
}

function createInitialResult(): DailyCloseoutResultView {
  return {
    score: { ...INITIAL_STEP },
    dailyReport: { ...INITIAL_STEP },
    weekPlan: { ...INITIAL_STEP },
    monthPlan: { ...INITIAL_STEP },
    closeoutRunId: null,
  };
}

function applyClarificationState(
  planStore: ReturnType<typeof usePlanStore>,
  periodType: "week" | "month",
  step: CloseoutStep
) {
  if (step.status !== "needs_clarification" || !step.session_id) {
    return;
  }

  const outcome: PlanAiOutcome = {
    session_id: step.session_id,
    status: "clarifying",
    requires_clarification: true,
    questions: step.questions,
    proposal: null,
  };
  planStore.setAiOutcome(periodType, outcome);
}

export function closeoutStatusLabel(status: CloseoutStepStatus): string {
  const map: Record<CloseoutStepStatus, string> = {
    pending: "等待",
    skipped: "跳过",
    success: "完成",
    needs_clarification: "待回答",
    error: "失败",
  };
  return map[status];
}
