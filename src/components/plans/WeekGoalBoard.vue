<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NAlert, NButton, NInput, NSelect, useMessage } from "naive-ui";
import GoalProgressSummary from "@/components/plans/GoalProgressSummary.vue";
import PlanAiDialog from "@/components/plans/PlanAiDialog.vue";
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

const snapshot = computed(() => planStore.weekPlan);
const headerRange = computed(() =>
  snapshot.value ? formatCycleRange(snapshot.value.cycle) : planStore.weekAnchorDate
);
const headerSummary = computed(
  () => snapshot.value?.cycle.ai_summary || "这一周的 Goal 推进和 My Day 成长，会一起沉淀在这里。"
);

watch(
  () => planStore.aiOutcome,
  (value) => {
    showAiDialog.value = Boolean(value);
  }
);

onMounted(() => {
  void loadCurrentWeek();
});

async function loadCurrentWeek() {
  pageError.value = "";
  try {
    await planStore.loadWeekPlan();
    if (!planStore.aiOutcome) {
      await planStore.restoreLatestAiOutcome("week");
    }
  } catch (error) {
    pageError.value = readError(error, "加载本周 Goal 失败");
  }
}

async function shiftWeek(delta: number) {
  try {
    const nextAnchor = shiftAnchorDate("week", planStore.weekAnchorDate, delta);
    await planStore.loadWeekPlan(nextAnchor);
    await planStore.restoreLatestAiOutcome("week", nextAnchor);
  } catch (error) {
    message.error(readError(error, "切换周视图失败"));
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
    await planStore.saveItem("week", {
      cycleId: snapshot.value.cycle.id,
      title: newGoalTitle.value.trim(),
      description: newGoalDescription.value.trim(),
      dimensionKey: newGoalDimension.value,
      isCompleted: false,
    });
    newGoalTitle.value = "";
    newGoalDescription.value = "";
    newGoalDimension.value = null;
    message.success("Goal 已加入本周列表");
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
    await planStore.saveItem("week", {
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
    await planStore.removeItem("week", snapshot.value.cycle.id, itemId);
    message.success("Goal 已删除");
  } catch (error) {
    message.error(readError(error, "删除 Goal 失败"));
  }
}

async function refreshFromCloseout() {
  try {
    const result = await closeoutStore.run(planStore.weekAnchorDate, "week");
    message.success(
      result.weekPlan.status === "needs_clarification"
        ? "周计划需要补充回答后再应用"
        : "周计划已通过统一收拢链路更新"
    );
  } catch (error) {
    message.error(readError(error, "周计划统一更新失败"));
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
    message.success("AI 已更新本周 Goal 进度");
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
  <div class="cyber-page goal-page">
    <h1 class="cyber-page-title">WEEK GOALS<span class="sub">本周目标</span></h1>

    <n-alert v-if="pageError" type="error" :show-icon="false" style="margin-bottom: 16px">
      {{ pageError }}
    </n-alert>

    <div class="goal-toolbar cyber-panel">
      <div class="toolbar-copy">
        <div class="toolbar-range">
          <button class="toolbar-btn" @click="shiftWeek(-1)">◀</button>
          <span>{{ headerRange }}</span>
          <button class="toolbar-btn" @click="shiftWeek(1)">▶</button>
        </div>
        <div class="toolbar-summary">{{ headerSummary }}</div>
      </div>
      <div class="toolbar-actions">
        <n-button :loading="planStore.loading" @click="loadCurrentWeek">刷新</n-button>
        <n-button type="primary" :loading="closeoutStore.running" @click="refreshFromCloseout">统一更新</n-button>
      </div>
    </div>

    <div v-if="snapshot" class="goal-layout">
      <section class="goal-main">
        <GoalProgressSummary
          title="Goal 总进度"
          :summary="snapshot.goal_progress"
          description="这条只表示 Goal 完成度，不等于能力成长。"
        />

        <div class="goal-block cyber-panel">
          <div class="block-title">Week Goals</div>
          <div v-if="snapshot.items.length === 0" class="empty-hint">本周还没有 Goal，先在下面补第一条。</div>
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

        <div class="goal-block cyber-panel">
          <div class="block-title">新增 Goal</div>
          <div class="goal-form">
            <n-input v-model:value="newGoalTitle" placeholder="Goal 标题" />
            <n-input
              v-model:value="newGoalDescription"
              type="textarea"
              placeholder="把这条 Goal 写得更具体一点，AI 才更容易判断进度"
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
            <div class="goal-form-row">
              <n-select v-model:value="newGoalDimension" :options="dimensionOptions" clearable placeholder="关联维度" />
              <n-button type="primary" :loading="planStore.saving" @click="createGoal">添加</n-button>
            </div>
          </div>
        </div>
      </section>

      <section class="goal-side">
        <PlanGrowthBars :growth="snapshot.growth" title="My Day Growth" />
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
.goal-page {
  display: grid;
  gap: 18px;
}

.goal-toolbar,
.goal-block {
  padding: 18px;
}

.goal-toolbar {
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

.goal-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.35fr) minmax(320px, 0.95fr);
  gap: 18px;
}

.goal-main,
.goal-side,
.goal-list,
.goal-form {
  display: grid;
  gap: 14px;
}

.block-title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 1px;
  text-transform: uppercase;
  color: var(--cyber-cyan);
}

.goal-form-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
}

@media (max-width: 1100px) {
  .goal-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .goal-toolbar {
    flex-direction: column;
  }

  .goal-form-row {
    grid-template-columns: 1fr;
  }
}
</style>
