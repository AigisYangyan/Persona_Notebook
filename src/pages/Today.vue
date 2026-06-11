<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  NAlert,
  NButton,
  NInput,
  NInputNumber,
  NModal,
  NRate,
  useMessage,
} from "naive-ui";
import { useRecordStore } from "@/stores/recordStore";
import { useStatStore } from "@/stores/statStore";
import { getTodayStr } from "@/utils/date";

const recordStore = useRecordStore();
const statStore = useStatStore();
const message = useMessage();

const showAddModal = ref(false);
const showConfirmModal = ref(false);
const newTitle = ref("");
const newMinutes = ref<number | null>(30);
const newStar = ref(0);
const analyzeError = ref("");
const analyzing = ref(false);
const confirming = ref(false);

onMounted(() => {
  void recordStore.fetchRecords();
  void statStore.refreshStats();
});

async function handleAdd() {
  if (!newTitle.value.trim()) {
    message.warning("请输入任务标题");
    return;
  }
  if (!newMinutes.value || newMinutes.value <= 0) {
    message.warning("请输入有效时长");
    return;
  }

  try {
    await recordStore.addRecord(newTitle.value.trim(), newMinutes.value, newStar.value);
    newTitle.value = "";
    newMinutes.value = 30;
    newStar.value = 0;
    showAddModal.value = false;
    message.success("任务已添加");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`添加失败: ${messageText}`);
  }
}

async function handleDelete(id: number) {
  try {
    await recordStore.removeRecord(id);
    message.success("任务已删除");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`删除失败: ${messageText}`);
  }
}

async function handleAnalyze() {
  if (recordStore.records.length === 0) {
    message.warning("请先添加任务");
    return;
  }

  analyzing.value = true;
  analyzeError.value = "";
  try {
    await recordStore.analyzeToday();
    showConfirmModal.value = true;
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    analyzeError.value = messageText || "评分失败";
    message.error(analyzeError.value);
  } finally {
    analyzing.value = false;
  }
}

async function handleConfirm() {
  confirming.value = true;
  try {
    await recordStore.confirmPendingPreview();
    await statStore.refreshStats();
    showConfirmModal.value = false;
    message.success("评分已写入账本");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`确认写入失败: ${messageText}`);
  } finally {
    confirming.value = false;
  }
}

function handleCancel() {
  showConfirmModal.value = false;
  recordStore.clearPendingPreview();
}

function getDimName(key: string): string {
  const map: Record<string, string> = {
    knowledge: "学识",
    willpower: "觉悟",
    expression: "表达",
    physique: "体魄",
    bond: "羁绊",
  };
  return map[key] || key;
}

function getWeekday(): string {
  const days = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
  return days[new Date().getDay()];
}
</script>

<template>
  <div class="cyber-page">
    <h1 class="cyber-page-title">
      TODAY RECORD<span class="sub">今日记录</span>
    </h1>

    <!-- Date & Actions Header -->
    <div class="today-header cyber-panel">
      <div class="date-section">
        <div class="date-label">DATE 日期</div>
        <div class="date-display">
          <span class="date-value">{{ getTodayStr() }}</span>
          <span class="date-weekday">{{ getWeekday() }}</span>
        </div>
      </div>
      <div class="action-buttons">
        <button class="cyber-btn primary" @click="showAddModal = true">
          <span class="btn-icon">+</span>
          <div class="btn-text">
            <span class="btn-label">添加任务</span>
            <span class="btn-sub">ADD TASK</span>
          </div>
        </button>
        <button class="cyber-btn" :disabled="analyzing" @click="handleAnalyze">
          <span class="btn-icon">📈</span>
          <div class="btn-text">
            <span class="btn-label">分析今日成长</span>
            <span class="btn-sub">ANALYZE TODAY GROWTH</span>
          </div>
        </button>
      </div>
    </div>

    <n-alert
      v-if="analyzeError"
      type="error"
      :show-icon="false"
      style="margin-top: 16px"
    >
      {{ analyzeError }}
    </n-alert>

    <!-- Task List -->
    <div class="task-section" style="margin-top: 20px;">
      <div class="cyber-section-title">
        今日任务<span class="sub">TODAY TASKS</span>
      </div>
      <div class="task-panel cyber-panel">
        <div
          v-if="recordStore.records.length === 0"
          class="task-empty"
        >
          今天还没有记录任务，点击「添加任务」开始吧
        </div>
        <div v-else class="task-list">
          <div
            v-for="record in recordStore.records"
            :key="record.id"
            class="cyber-task-item"
          >
            <div class="task-checkbox">
              <div class="checkbox-box"></div>
            </div>
            <div class="task-info">
              <div class="task-title">{{ record.title }}</div>
              <div class="task-subtitle">{{ record.title.toUpperCase() }}</div>
            </div>
            <div class="task-meta">
              <div class="meta-item">
                <span class="meta-icon">⏱</span>
                <div class="meta-text">
                  <span class="meta-value">{{ record.minutes }}分钟</span>
                  <span class="meta-label">DURATION</span>
                </div>
              </div>
              <div class="meta-item">
                <span class="meta-icon">⭐</span>
                <div class="meta-text">
                  <span class="meta-value">
                    <span v-for="s in record.difficulty_star" :key="s" class="star-filled">★</span>
                    <span v-for="s in (3 - record.difficulty_star)" :key="`e${s}`" class="star-empty">★</span>
                  </span>
                  <span class="meta-label">DIFFICULTY</span>
                </div>
              </div>
            </div>
            <button class="task-delete" @click="handleDelete(record.id)">
              <span class="del-icon">🗑</span>
              <span class="del-label">删除</span>
              <span class="del-sub">DELETE</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Task Modal -->
    <n-modal
      v-model:show="showAddModal"
      title="添加任务"
      preset="card"
      style="width: 420px"
      class="cyber-modal"
    >
      <div class="modal-form">
        <div class="form-row">
          <label class="form-label">任务描述</label>
          <n-input v-model:value="newTitle" placeholder="今天做了什么？" />
        </div>
        <div class="form-row">
          <label class="form-label">耗时（分钟）</label>
          <n-input-number v-model:value="newMinutes" :min="1" :max="1440" />
        </div>
        <div class="form-row">
          <label class="form-label">难度星级</label>
          <div class="rate-row">
            <n-rate v-model:value="newStar" :count="3" />
            <span class="rate-hint">{{ newStar === 0 ? '未标记' : '' }}</span>
          </div>
        </div>
        <n-button type="primary" block @click="handleAdd">确认添加</n-button>
      </div>
    </n-modal>

    <!-- Confirm Modal -->
    <n-modal
      v-model:show="showConfirmModal"
      title="今日成长分析结果"
      preset="card"
      style="width: 580px"
      class="cyber-modal"
    >
      <div class="modal-form">
        <div
          v-for="(item, idx) in recordStore.pendingPreview"
          :key="`${item.title}-${idx}`"
          class="preview-item"
        >
          <strong>{{ item.title }}</strong>
          <div class="preview-meta">
            分类: {{ item.category }}
            <span style="margin-left: 8px">
              置信度: {{ (item.confidence * 100).toFixed(0) }}%
            </span>
          </div>
          <div class="preview-changes">
            <template v-if="item.changes.length > 0">
              <span
                v-for="change in item.changes"
                :key="change.dimension_key"
                class="change-tag"
              >
                {{ getDimName(change.dimension_key) }} +{{ change.change_value }}
              </span>
            </template>
            <span v-else style="color: var(--cyber-text-muted)">无新增成长（可能已达单日上限）</span>
          </div>
          <div class="preview-reason">{{ item.reason }}</div>
        </div>

        <div
          v-if="recordStore.pendingSummary"
          class="preview-summary"
        >
          {{ recordStore.pendingSummary }}
        </div>

        <div class="modal-actions">
          <n-button @click="handleCancel">取消</n-button>
          <n-button type="primary" :loading="confirming" @click="handleConfirm">
            确认写入账本
          </n-button>
        </div>
      </div>
    </n-modal>
  </div>
</template>

<style scoped>
.today-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  gap: 24px;
}

.date-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.date-label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 2px;
  color: var(--cyber-cyan);
}

.date-display {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.date-value {
  font-size: 32px;
  font-weight: 900;
  font-style: italic;
  color: var(--cyber-text-primary);
  letter-spacing: 1px;
  text-shadow: 0 0 12px rgba(0, 212, 255, 0.2);
}

.date-weekday {
  font-size: 14px;
  font-weight: 700;
  color: var(--cyber-bg-base);
  background: var(--cyber-cyan);
  padding: 2px 10px;
  border-radius: 2px;
  letter-spacing: 2px;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.cyber-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 20px;
  background: rgba(0, 60, 120, 0.2);
  border: 1px solid var(--cyber-border);
  border-radius: 4px;
  color: var(--cyber-text-secondary);
  cursor: pointer;
  transition: all 0.25s;
  font-family: inherit;
}

.cyber-btn:hover {
  border-color: var(--cyber-border-hover);
  background: rgba(0, 180, 255, 0.1);
  color: var(--cyber-text-primary);
}

.cyber-btn.primary {
  background: linear-gradient(135deg, rgba(0, 100, 200, 0.3), rgba(0, 180, 255, 0.2));
  border-color: var(--cyber-cyan);
  box-shadow: 0 0 12px rgba(0, 180, 255, 0.15);
}

.cyber-btn.primary:hover {
  box-shadow: 0 0 16px rgba(0, 212, 255, 0.3);
}

.cyber-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-icon {
  font-size: 20px;
  line-height: 1;
}

.btn-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
}

.btn-label {
  font-size: 14px;
  font-weight: 600;
}

.btn-sub {
  font-size: 9px;
  font-weight: 500;
  letter-spacing: 1px;
  color: var(--cyber-text-muted);
}

/* Task Section */
.task-panel {
  padding: 16px 20px;
  min-height: 200px;
}

.task-empty {
  text-align: center;
  padding: 48px;
  color: var(--cyber-text-muted);
  font-size: 14px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cyber-task-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 18px;
  background: rgba(0, 15, 40, 0.4);
  border: 1px solid var(--cyber-border);
  border-left: 3px solid var(--cyber-cyan);
  border-radius: 0 4px 4px 0;
  transition: all 0.2s;
}

.cyber-task-item:hover {
  background: rgba(0, 30, 60, 0.4);
  border-color: var(--cyber-border-hover);
}

.task-checkbox {
  display: flex;
  align-items: center;
}

.checkbox-box {
  width: 18px;
  height: 18px;
  border: 2px solid var(--cyber-border);
  border-radius: 3px;
  transition: all 0.2s;
}

.cyber-task-item:hover .checkbox-box {
  border-color: var(--cyber-cyan);
}

.task-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.task-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--cyber-text-primary);
}

.task-subtitle {
  font-size: 10px;
  color: var(--cyber-text-dim);
  letter-spacing: 1px;
  text-transform: uppercase;
}

.task-meta {
  display: flex;
  gap: 24px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.meta-icon {
  font-size: 16px;
  color: var(--cyber-text-muted);
}

.meta-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.meta-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--cyber-cyan);
}

.star-filled {
  color: var(--cyber-cyan);
  text-shadow: 0 0 4px rgba(0, 212, 255, 0.5);
}

.star-empty {
  color: var(--cyber-text-dim);
}

.meta-label {
  font-size: 9px;
  color: var(--cyber-text-dim);
  letter-spacing: 1px;
}

.task-delete {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  padding: 6px 12px;
  background: rgba(255, 51, 102, 0.08);
  border: 1px solid rgba(255, 51, 102, 0.2);
  border-radius: 4px;
  color: var(--cyber-danger);
  cursor: pointer;
  transition: all 0.2s;
  font-family: inherit;
  min-width: 60px;
}

.task-delete:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--cyber-danger);
  box-shadow: 0 0 8px rgba(255, 51, 102, 0.2);
}

.del-icon {
  font-size: 14px;
}

.del-label {
  font-size: 12px;
  font-weight: 600;
}

.del-sub {
  font-size: 8px;
  letter-spacing: 1px;
  opacity: 0.7;
}

/* Modal Form */
.modal-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 8px 4px;
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--cyber-text-secondary);
  letter-spacing: 1px;
}

.rate-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.rate-hint {
  font-size: 12px;
  color: var(--cyber-text-muted);
}

/* Preview Items */
.preview-item {
  padding: 14px 16px;
  background: rgba(0, 20, 50, 0.4);
  border: 1px solid var(--cyber-border);
  border-radius: 4px;
  font-size: 14px;
}

.preview-item strong {
  color: var(--cyber-text-primary);
  font-size: 15px;
}

.preview-meta {
  color: var(--cyber-text-muted);
  font-size: 12px;
  margin-top: 6px;
}

.preview-changes {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.change-tag {
  padding: 3px 10px;
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid rgba(0, 212, 255, 0.25);
  border-radius: 3px;
  font-size: 12px;
  color: var(--cyber-cyan);
}

.preview-reason {
  color: var(--cyber-text-dim);
  font-size: 12px;
  margin-top: 6px;
  line-height: 1.5;
}

.preview-summary {
  padding: 10px 14px;
  background: rgba(0, 40, 80, 0.2);
  border-radius: 4px;
  font-size: 13px;
  color: var(--cyber-text-secondary);
  line-height: 1.5;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
}
</style>
