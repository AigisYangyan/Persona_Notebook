<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NAlert, NButton, NInput, NSelect, useMessage } from "naive-ui";
import PlanAiDialog from "@/components/plans/PlanAiDialog.vue";
import GoalProgressSummary from "@/components/plans/GoalProgressSummary.vue";
import PlanGrowthBars from "@/components/plans/PlanGrowthBars.vue";
import PlanItemCard from "@/components/plans/PlanItemCard.vue";
import { formatCycleRange, shiftAnchorDate } from "@/features/plans/periods";
import { useCloseoutStore } from "@/stores/closeoutStore";
import { usePlanStore } from "@/stores/planStore";

const planStore = usePlanStore();
const closeoutStore = useCloseoutStore();
const message = useMessage();

const pageError = ref("");
const showAiDialog = ref(false);
const cycleTitleDraft = ref("");
const cycleSummaryDraft = ref("");
const newGoalTitle = ref("");
const newGoalDescription = ref("");
const newGoalDimension = ref<string | null>(null);

const dimensionOptions = [
  { label: "知识", value: "knowledge" },
  { label: "觉悟", value: "willpower" },
  { label: "表达", value: "expression" },
  { label: "体魄", value: "physique" },
  { label: "羁绊", value: "bond" },
];

const snapshot = computed(() => planStore.monthPlan);
const headerRange = computed(() =>
  snapshot.value ? formatCycleRange(snapshot.value.cycle) : planStore.monthAnchorDate
);

watch(
  () => planStore.aiOutcome,
  (value) => {
    showAiDialog.value = Boolean(value);
  }
);

watch(
  snapshot,
  (value) => {
    cycleTitleDraft.value = value?.cycle.title ?? "";
    cycleSummaryDraft.value = value?.cycle.summary ?? "";
  },
  { immediate: true }
);

onMounted(() => {
  planStore.clearAiState();
  void loadCurrentMonth();
});

async function loadCurrentMonth() {
  pageError.value = "";
  try {
    await planStore.loadMonthPlan();
  } catch (error) {
    pageError.value = readError(error, "加载本月计划失败");
  }
}

async function shiftMonth(delta: number) {
  try {
    const nextAnchor = shiftAnchorDate("month", planStore.monthAnchorDate, delta);
    await planStore.loadMonthPlan(nextAnchor);
  } catch (error) {
    message.error(readError(error, "切换月份失败"));
  }
}

async function saveMonthSummary() {
  try {
    await planStore.saveCycle("month", cycleTitleDraft.value.trim(), cycleSummaryDraft.value.trim());
    message.success("月摘要已保存");
  } catch (error) {
    message.error(readError(error, "保存月摘要失败"));
  }
}

async function createGoal() {
  if (!snapshot.value) {
    return;
  }
  if (!newGoalTitle.value.trim()) {
    message.warning("先写一个 Goal 标题");
    return;
  }
  try {
    await planStore.saveItem("month", {
      cycleId: snapshot.value.cycle.id,
      title: newGoalTitle.value.trim(),
      description: newGoalDescription.value.trim(),
      dimensionKey: newGoalDimension.value,
      isCompleted: false,
    });
    newGoalTitle.value = "";
    newGoalDescription.value = "";
    newGoalDimension.value = null;
    message.success("Goal 已加入本月列表");
  } catch (error) {
    message.error(readError(error, "新增 Goal 失败"));
  }
}

async function saveGoal(payload: {
  itemId: number;
  title: string;
  description: string;
  dimensionKey: string | null;
  sortOrder: number;
  isCompleted: boolean;
}) {
  if (!snapshot.value) {
    return;
  }
  try {
    await planStore.saveItem("month", {
      cycleId: snapshot.value.cycle.id,
      itemId: payload.itemId,
      title: payload.title,
      description: payload.description,
      dimensionKey: payload.dimensionKey,
      sortOrder: payload.sortOrder,
      isCompleted: payload.isCompleted,
    });
    message.success("Goal 已更新");
  } catch (error) {
    message.error(readError(error, "更新 Goal 失败"));
  }
}

async function deleteGoal(itemId: number) {
  if (!snapshot.value) {
    return;
  }
  try {
    await planStore.removeItem("month", snapshot.value.cycle.id, itemId);
    message.success("Goal 已删除");
  } catch (error) {
    message.error(readError(error, "删除 Goal 失败"));
  }
}

async function refreshFromCloseout() {
  try {
    const result = await closeoutStore.run(planStore.monthAnchorDate, "month");
    message.success(
      result.monthPlan.status === "needs_clarification" ? "月计划需要补充回答后再应用" : "月计划已通过统一收拢链路更新"
    );
  } catch (error) {
    message.error(readError(error, "月计划统一更新失败"));
  }
}

async function submitAiAnswers() {
  const hasBlank = planStore.aiAnswers.some((answer) => !answer.trim());
  if (hasBlank) {
    message.warning("先把澄清问题补充完整");
    return;
  }
  try {
    await planStore.submitAiAnswers();
  } catch (error) {
    message.error(readError(error, "提交回答失败"));
  }
}

async function applyAiProposal() {
  try {
    await planStore.applyAiProposal();
    showAiDialog.value = false;
    message.success("AI 已更新本月 Goal 进度");
  } catch (error) {
    message.error(readError(error, "应用 AI 修改失败"));
  }
}

function updateAiAnswer(index: number, value: string) {
  planStore.setAiAnswer(index, value);
}

function readError(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}
</script>

<template>
  <div class="cyber-page month-page">
    <h1 class="cyber-page-title">
      MONTH PLAN<span class="sub">本月总览</span>
    </h1>

    <n-alert v-if="pageError" type="error" :show-icon="false" style="margin-bottom: 16px">
      {{ pageError }}
    </n-alert>

    <div class="month-toolbar cyber-panel">
      <div class="toolbar-copy">
        <div class="toolbar-range">
          <button class="toolbar-btn" @click="shiftMonth(-1)">◀</button>
          <span>{{ headerRange }}</span>
          <button class="toolbar-btn" @click="shiftMonth(1)">▶</button>
        </div>
        <div class="toolbar-summary">
          {{ snapshot?.cycle.ai_summary || "这里更偏向月度观察和校准，而不是逐条执行。" }}
        </div>
      </div>
      <div class="toolbar-actions">
        <n-button :loading="planStore.loading" @click="loadCurrentMonth">刷新</n-button>
        <n-button type="primary" :loading="closeoutStore.running" @click="refreshFromCloseout">统一更新</n-button>
      </div>
    </div>

    <div v-if="snapshot" class="month-layout">
      <section class="month-main">
        <div class="month-block cyber-panel">
          <div class="month-block-head">
            <div class="month-block-title">Month Summary</div>
            <n-button :loading="planStore.saving" @click="saveMonthSummary">保存摘要</n-button>
          </div>
          <div class="month-form">
            <n-input v-model:value="cycleTitleDraft" placeholder="本月标题" />
            <n-input
              v-model:value="cycleSummaryDraft"
              type="textarea"
              placeholder="写下这个月真正想推进的方向，后面交给 AI 和每周执行去修正。"
              :autosize="{ minRows: 3, maxRows: 5 }"
            />
          </div>
        </div>

        <GoalProgressSummary
          title="Month Goals 总进度"
          :summary="snapshot.goal_progress"
          description="这条只表示目标推进，不代表能力成长。"
        />

        <div class="month-block cyber-panel">
          <div class="month-block-title">Month Goals</div>
          <div v-if="snapshot.items.length === 0" class="empty-hint">
            本月还没有 Goal，可以先从一个最想推进的方向开始。
          </div>
          <div v-else class="goal-list">
            <PlanItemCard
              v-for="item in snapshot.items"
              :key="item.id"
              :item="item"
              :busy="planStore.saving"
              @save="saveGoal"
              @remove="deleteGoal"
            />
          </div>
        </div>

        <div class="month-block cyber-panel">
          <div class="month-block-title">新增 Month Goal</div>
          <div class="month-form">
            <n-input v-model:value="newGoalTitle" placeholder="Goal 标题" />
            <n-input
              v-model:value="newGoalDescription"
              type="textarea"
              placeholder="写清对象、动作和结果，AI 才能更稳地判断月度推进。"
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
            <div class="month-form-row">
              <n-select
                v-model:value="newGoalDimension"
                :options="dimensionOptions"
                clearable
                placeholder="关联维度"
              />
              <n-button type="primary" :loading="planStore.saving" @click="createGoal">添加</n-button>
            </div>
          </div>
        </div>
      </section>

      <section class="month-side">
        <div class="month-block cyber-panel">
          <div class="month-block-title">Week Rollup</div>
          <div v-if="snapshot.related_weeks.length === 0" class="empty-hint">
            本月还没有周计划落点。
          </div>
          <div v-else class="rollup-list">
            <div v-for="week in snapshot.related_weeks" :key="week.cycle_id" class="rollup-item">
              <div class="rollup-head">
                <span>{{ week.title }}</span>
                <span>{{ week.average_progress }}%</span>
              </div>
              <div class="rollup-meta">
                {{ week.start_date }} - {{ week.end_date }} / {{ week.completed_items }}/{{ week.total_items }} 已完成
              </div>
              <div class="rollup-summary">{{ week.summary || "这一周还没有摘要。" }}</div>
            </div>
          </div>
        </div>

        <PlanGrowthBars :growth="snapshot.growth" title="Month Growth" />
      </section>
    </div>

    <PlanAiDialog
      v-model:show="showAiDialog"
      :outcome="planStore.aiOutcome"
      :answers="planStore.aiAnswers"
      :loading="planStore.aiLoading"
      @update-answer="({ index, value }) => updateAiAnswer(index, value)"
      @submit-answers="submitAiAnswers"
      @apply="applyAiProposal"
      @close="planStore.clearAiState"
    />
  </div>
</template>

<style scoped>
.month-page {
  display: grid;
  gap: 18px;
}

.month-toolbar,
.month-block {
  padding: 18px;
}

.month-toolbar {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

.toolbar-copy {
  display: grid;
  gap: 8px;
}

.toolbar-range {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 14px;
  font-weight: 700;
  color: var(--cyber-cyan);
}

.toolbar-btn {
  width: 36px;
  height: 36px;
  border: 1px solid var(--cyber-border);
  background: rgba(0, 28, 60, 0.5);
  color: var(--cyber-cyan);
  cursor: pointer;
}

.toolbar-summary,
.empty-hint,
.rollup-meta,
.rollup-summary,
.toolbar-hint {
  font-size: 13px;
  line-height: 1.6;
  color: var(--cyber-text-muted);
}

.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.month-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(340px, 1fr);
  gap: 18px;
}

.month-main,
.month-side,
.month-form,
.goal-list,
.rollup-list {
  display: grid;
  gap: 14px;
}

.month-block-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 14px;
}

.month-block-title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 1px;
  text-transform: uppercase;
  color: var(--cyber-cyan);
}

.month-form-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
}

 .rollup-item {
  padding: 12px;
  border: 1px solid rgba(0, 180, 255, 0.18);
  background: rgba(0, 20, 48, 0.4);
}

.rollup-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 6px;
  color: var(--cyber-text-primary);
  font-size: 13px;
  font-weight: 700;
}

@media (max-width: 1100px) {
  .month-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .month-toolbar,
  .month-block-head {
    flex-direction: column;
  }

  .month-form-row {
    grid-template-columns: 1fr;
  }
}
</style>
