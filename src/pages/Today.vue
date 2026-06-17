<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { NButton, NInput, NInputNumber, NModal, NRate, useMessage } from "naive-ui";
import TaskTimePieChart from "@/components/charts/TaskTimePieChart.vue";
import PlanAiDialog from "@/components/plans/PlanAiDialog.vue";
import TodayTaskTreeItem from "@/components/TodayTaskTreeItem.vue";
import { formatDuration } from "@/features/records/taskMetrics";
import { closeoutStatusLabel, useCloseoutStore } from "@/stores/closeoutStore";
import { usePlanStore } from "@/stores/planStore";
import { useRecordStore } from "@/stores/recordStore";
import { useStatStore } from "@/stores/statStore";
import { getTodayStr } from "@/utils/date";

type ClarificationStepKey = "weekPlan" | "monthPlan";

const recordStore = useRecordStore();
const statStore = useStatStore();
const closeoutStore = useCloseoutStore();
const planStore = usePlanStore();
const message = useMessage();

const showTaskModal = ref(false);
const showManualEntryModal = ref(false);
const showAiDialog = ref(false);

const taskModalParentId = ref<number | null>(null);
const taskModalTitle = ref("");
const taskModalDifficulty = ref(0);

const manualEntryTitle = ref("");
const manualEntryMinutes = ref<number | null>(30);
const manualEntryDifficulty = ref(0);
const manualEntryParentId = ref<number | null>(null);

const mutatingId = ref<number | null>(null);

const runningLabel = computed(() => recordStore.runningRecord?.title ?? "暂无运行任务");
const trackedLabel = computed(() => formatDuration(recordStore.totalTrackedSeconds));
const taskCountLabel = computed(() => `${recordStore.records.length}`);
const closeoutSteps = computed(() => [
  { key: "score", label: "今日点数", ...closeoutStore.result.score },
  { key: "dailyReport", label: "每日报告", ...closeoutStore.result.dailyReport },
  { key: "weekPlan", label: "Week Plan", ...closeoutStore.result.weekPlan },
  { key: "monthPlan", label: "Month Plan", ...closeoutStore.result.monthPlan },
]);

watch(
  () => planStore.aiOutcome,
  (value) => {
    showAiDialog.value = Boolean(value);
  }
);

onMounted(() => {
  recordStore.startClock();
  void Promise.all([recordStore.fetchRecords(), statStore.refreshStats(), closeoutStore.loadLatest(getTodayStr())]);
});

onUnmounted(() => {
  recordStore.stopClock();
});

function openTaskModal(parentId: number | null = null) {
  taskModalParentId.value = parentId;
  taskModalTitle.value = "";
  taskModalDifficulty.value = 0;
  showTaskModal.value = true;
}

function openManualEntryModal(parentId: number | null = null) {
  manualEntryParentId.value = parentId;
  manualEntryTitle.value = "";
  manualEntryMinutes.value = 30;
  manualEntryDifficulty.value = 0;
  showManualEntryModal.value = true;
}

async function handleSubmitTask() {
  if (!taskModalTitle.value.trim()) {
    message.warning("请输入任务标题");
    return;
  }

  try {
    if (taskModalParentId.value === null) {
      await recordStore.addRecord(taskModalTitle.value.trim(), taskModalDifficulty.value, "stopwatch", null);
    } else {
      await recordStore.addSubRecord(
        taskModalParentId.value,
        taskModalTitle.value.trim(),
        taskModalDifficulty.value,
        "stopwatch",
        null
      );
    }
    showTaskModal.value = false;
    message.success(taskModalParentId.value === null ? "任务已添加" : "子任务已添加");
  } catch (error) {
    message.error(readError(error, "添加任务失败"));
  }
}

async function handleSubmitManualEntry() {
  if (!manualEntryTitle.value.trim()) {
    message.warning("请输入任务标题");
    return;
  }
  const minutes = Math.max(0, Math.floor(manualEntryMinutes.value ?? 0));
  if (minutes <= 0) {
    message.warning("请输入大于 0 的分钟数");
    return;
  }

  try {
    await recordStore.addManualRecord(
      manualEntryTitle.value.trim(),
      minutes,
      manualEntryDifficulty.value,
      manualEntryParentId.value
    );
    showManualEntryModal.value = false;
    message.success("任务耗时已导入");
  } catch (error) {
    message.error(readError(error, "导入任务耗时失败"));
  }
}

async function withMutation(id: number, action: () => Promise<void>, successText?: string) {
  mutatingId.value = id;
  try {
    await action();
    if (successText) {
      message.success(successText);
    }
  } catch (error) {
    message.error(readError(error, "操作失败"));
  } finally {
    mutatingId.value = null;
  }
}

async function handleDailyCloseout() {
  try {
    const result = await closeoutStore.run(getTodayStr());
    const needsClarification = [result.weekPlan, result.monthPlan].some(
      (step) => step.status === "needs_clarification"
    );
    message.success(needsClarification ? "夜间收拢完成，周/月计划还有待补充回答" : "夜间收拢完成");
  } catch (error) {
    message.error(readError(error, "夜间收拢失败"));
  }
}

async function openClarification(stepKey: ClarificationStepKey) {
  const step = closeoutStore.result[stepKey];
  if (step.status !== "needs_clarification") {
    return;
  }

  try {
    const periodType = stepKey === "weekPlan" ? "week" : "month";
    if (step.session_id) {
      await planStore.loadAiOutcome(step.session_id, periodType);
    } else {
      const restored = await planStore.restoreLatestAiOutcome(periodType, getTodayStr());
      if (!restored) {
        message.warning("暂时没有找到可补答的问题");
        return;
      }
    }
  } catch (error) {
    message.error(readError(error, "打开补充回答失败"));
  }
}

function handleCloseoutStepClick(stepKey: string) {
  if (stepKey === "weekPlan" || stepKey === "monthPlan") {
    void openClarification(stepKey);
  }
}

async function submitAiAnswers() {
  const hasBlank = planStore.aiAnswers.some((answer) => !answer.trim());
  if (hasBlank) {
    message.warning("先把问题补充完整");
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
    await closeoutStore.loadLatest(getTodayStr());
    message.success("计划 AI 修改已应用");
  } catch (error) {
    message.error(readError(error, "应用 AI 修改失败"));
  }
}

function updateAiAnswer(index: number, value: string) {
  planStore.setAiAnswer(index, value);
}

function getWeekday(): string {
  const days = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
  return days[new Date().getDay()];
}

function readError(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}
</script>

<template>
  <div class="cyber-page">
    <h1 class="cyber-page-title">TODAY RECORD<span class="sub">今日记录</span></h1>

    <div class="today-header cyber-panel">
      <div class="date-section">
        <div class="date-label">DATE 日期</div>
        <div class="date-display">
          <span class="date-value">{{ getTodayStr() }}</span>
          <span class="date-weekday">{{ getWeekday() }}</span>
        </div>
      </div>
      <div class="action-buttons">
        <button class="cyber-btn primary" @click="openTaskModal()">
          <span class="btn-icon">+</span>
          <div class="btn-text">
            <span class="btn-label">添加任务</span>
            <span class="btn-sub">ADD TASK</span>
          </div>
        </button>
        <button class="cyber-btn" @click="openManualEntryModal()">
          <span class="btn-icon">M</span>
          <div class="btn-text">
            <span class="btn-label">导入任务+时间</span>
            <span class="btn-sub">MANUAL ENTRY</span>
          </div>
        </button>
        <button class="cyber-btn closeout" :disabled="closeoutStore.running" @click="handleDailyCloseout">
          <span class="btn-icon">N</span>
          <div class="btn-text">
            <span class="btn-label">夜间收拢</span>
            <span class="btn-sub">GLOBAL CLOSEOUT</span>
          </div>
        </button>
      </div>
    </div>

    <div v-if="closeoutStore.running || closeoutStore.hasResult" class="closeout-panel cyber-panel">
      <div class="cyber-section-title">夜间收拢<span class="sub">DAILY CLOSEOUT</span></div>
      <div class="closeout-steps">
        <button
          v-for="step in closeoutSteps"
          :key="step.key"
          class="closeout-step"
          :class="[step.status, { clickable: step.status === 'needs_clarification' }]"
          :disabled="step.status !== 'needs_clarification'"
          @click="handleCloseoutStepClick(step.key)"
        >
          <span class="step-label">{{ step.label }}</span>
          <strong>{{ closeoutStatusLabel(step.status) }}</strong>
          <small>{{ step.message || "等待执行" }}</small>
          <span v-if="step.status === 'needs_clarification'" class="step-action">点击补充回答</span>
        </button>
      </div>
    </div>

    <div class="today-grid">
      <section class="left-column">
        <div class="stats-row">
          <div class="stat-card cyber-panel">
            <div class="cyber-stat-label">TRACKED</div>
            <div class="cyber-stat-value">{{ trackedLabel }}</div>
            <div class="cyber-stat-sublabel">今日累计有效时长</div>
          </div>
          <div class="stat-card cyber-panel">
            <div class="cyber-stat-label">RUNNING</div>
            <div class="running-title">{{ runningLabel }}</div>
            <div class="cyber-stat-sublabel">同一时刻只运行一个任务</div>
          </div>
          <div class="stat-card cyber-panel">
            <div class="cyber-stat-label">DONE</div>
            <div class="cyber-stat-value">{{ recordStore.completedCount }}</div>
            <div class="cyber-stat-sublabel">共 {{ taskCountLabel }} 个任务</div>
          </div>
        </div>

        <div class="cyber-section-title">今日任务<span class="sub">TODAY TASKS</span></div>
        <div class="task-panel cyber-panel">
          <div v-if="recordStore.records.length === 0" class="task-empty">
            今天还没有任务。你可以直接新建任务开始计时，也可以把外部完成的任务和耗时手动导入进来。
          </div>
          <div v-else class="task-list">
            <TodayTaskTreeItem
              v-for="node in recordStore.taskTree"
              :key="node.record.id"
              :node="node"
              @toggle-completed="
                withMutation(
                  $event.id,
                  () => recordStore.setCompleted($event.id, $event.isCompleted),
                  $event.isCompleted ? '任务已完成' : '任务已恢复'
                )
              "
              @start="withMutation($event, () => recordStore.startTimer($event), '开始计时')"
              @pause="withMutation($event, () => recordStore.pauseTimer($event), '暂停计时')"
              @reset="withMutation($event, () => recordStore.resetTimer($event), '计时已归零')"
              @add-subtask="openTaskModal($event)"
              @remove="withMutation($event, () => recordStore.removeRecord($event), '任务已删除')"
            />
          </div>
          <div class="task-import-row">
            <n-button secondary @click="openManualEntryModal()">补录外部任务时间</n-button>
          </div>
          <div v-if="mutatingId !== null" class="mutation-hint">正在更新任务 #{{ mutatingId }} ...</div>
        </div>
      </section>

      <section class="right-column">
        <div class="cyber-section-title">时间占比<span class="sub">TIME SHARE</span></div>
        <div class="chart-panel cyber-panel">
          <TaskTimePieChart :items="recordStore.chartItems" />
        </div>
      </section>
    </div>

    <n-modal
      v-model:show="showTaskModal"
      :title="taskModalParentId === null ? '添加任务' : '添加子任务'"
      preset="card"
      style="width: 440px"
      class="cyber-modal"
    >
      <div class="modal-form">
        <div class="form-row">
          <label class="form-label">任务标题</label>
          <n-input v-model:value="taskModalTitle" placeholder="今天要推进什么？" />
        </div>
        <div class="form-row">
          <label class="form-label">难度星级</label>
          <n-rate v-model:value="taskModalDifficulty" :count="3" />
        </div>
        <n-button type="primary" block @click="handleSubmitTask">确认</n-button>
      </div>
    </n-modal>

    <n-modal
      v-model:show="showManualEntryModal"
      title="导入任务时间"
      preset="card"
      style="width: 460px"
      class="cyber-modal"
    >
      <div class="modal-form">
        <div class="form-row">
          <label class="form-label">任务标题</label>
          <n-input v-model:value="manualEntryTitle" placeholder="例如：线下自习 / 通勤复盘 / 健身" />
        </div>
        <div class="form-row">
          <label class="form-label">耗时（分钟）</label>
          <n-input-number v-model:value="manualEntryMinutes" :min="1" :max="1440" style="width: 100%" />
        </div>
        <div class="form-row">
          <label class="form-label">难度星级</label>
          <n-rate v-model:value="manualEntryDifficulty" :count="3" />
        </div>
        <n-button type="primary" block @click="handleSubmitManualEntry">写入记录</n-button>
      </div>
    </n-modal>

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
.today-header,
.closeout-panel,
.task-panel,
.chart-panel,
.stat-card {
  padding: 18px;
}

.today-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: center;
}

.date-section {
  display: grid;
  gap: 6px;
}

.date-label,
.btn-sub,
.cyber-stat-label {
  font-size: 12px;
  letter-spacing: 2px;
  color: var(--cyber-text-muted);
}

.date-display {
  display: flex;
  gap: 12px;
  align-items: baseline;
}

.date-value {
  font-size: 28px;
  font-weight: 800;
  color: var(--cyber-text-primary);
}

.date-weekday {
  color: var(--cyber-cyan);
  font-weight: 700;
  letter-spacing: 2px;
}

.action-buttons {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.cyber-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border: 1px solid var(--cyber-border);
  background: rgba(0, 28, 60, 0.5);
  color: var(--cyber-text-primary);
  cursor: pointer;
}

.cyber-btn.primary {
  border-color: rgba(0, 212, 255, 0.4);
}

.cyber-btn.closeout {
  border-color: rgba(0, 255, 170, 0.32);
}

.btn-icon {
  width: 26px;
  text-align: center;
  color: var(--cyber-cyan);
  font-weight: 800;
}

.btn-text {
  display: grid;
}

.btn-label {
  font-size: 14px;
  font-weight: 700;
}

.closeout-panel {
  margin-top: 16px;
}

.closeout-steps {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.closeout-step {
  display: grid;
  gap: 6px;
  padding: 12px;
  border: 1px solid rgba(0, 180, 255, 0.14);
  background: rgba(0, 18, 45, 0.45);
  text-align: left;
}

.closeout-step.clickable {
  cursor: pointer;
}

.closeout-step.success {
  border-color: rgba(0, 255, 170, 0.26);
}

.closeout-step.skipped,
.closeout-step.needs_clarification {
  border-color: rgba(255, 204, 0, 0.26);
}

.closeout-step.error {
  border-color: rgba(255, 51, 102, 0.28);
}

.step-label {
  color: var(--cyber-text-muted);
  font-size: 12px;
}

.closeout-step strong {
  color: var(--cyber-text-primary);
}

.closeout-step small,
.step-action {
  color: var(--cyber-text-secondary);
  line-height: 1.5;
}

.step-action {
  color: var(--cyber-cyan);
  font-size: 12px;
}

.today-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.35fr) minmax(320px, 0.9fr);
  gap: 18px;
  margin-top: 18px;
}

.left-column,
.right-column,
.task-list {
  display: grid;
  gap: 14px;
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 14px;
}

.cyber-stat-value,
.running-title {
  margin-top: 6px;
  font-size: 24px;
  font-weight: 800;
  color: var(--cyber-text-primary);
}

.running-title {
  font-size: 18px;
  line-height: 1.4;
}

.cyber-stat-sublabel {
  margin-top: 8px;
  color: var(--cyber-text-muted);
  font-size: 12px;
}

.task-empty,
.mutation-hint {
  color: var(--cyber-text-muted);
  line-height: 1.7;
}

.task-import-row {
  display: flex;
  justify-content: flex-end;
}

.modal-form {
  display: grid;
  gap: 14px;
}

.form-row {
  display: grid;
  gap: 8px;
}

.form-label {
  color: var(--cyber-text-secondary);
  font-size: 13px;
}

@media (max-width: 1100px) {
  .today-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 900px) {
  .stats-row {
    grid-template-columns: 1fr;
  }

  .closeout-steps {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 720px) {
  .today-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .action-buttons {
    width: 100%;
    justify-content: stretch;
  }

  .cyber-btn {
    width: 100%;
  }

  .closeout-steps {
    grid-template-columns: 1fr;
  }
}
</style>
