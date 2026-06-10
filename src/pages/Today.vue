<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  NAlert,
  NButton,
  NCard,
  NInput,
  NInputNumber,
  NModal,
  NRate,
  NSpace,
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
</script>

<template>
  <div>
    <h1 style="margin-top: 0">今日记录 — {{ getTodayStr() }}</h1>
    <n-space vertical :size="16">
      <n-card>
        <n-space>
          <n-button type="primary" @click="showAddModal = true">添加任务</n-button>
          <n-button :loading="analyzing" @click="handleAnalyze">分析今日成长</n-button>
        </n-space>
        <n-alert
          v-if="analyzeError"
          type="error"
          :show-icon="false"
          style="margin-top: 12px"
        >
          {{ analyzeError }}
        </n-alert>
      </n-card>

      <n-card title="今日任务">
        <div
          v-if="recordStore.records.length === 0"
          style="text-align: center; color: #999; padding: 24px"
        >
          今天还没有记录任务，点击「添加任务」开始吧
        </div>
        <div v-else>
          <div
            v-for="record in recordStore.records"
            :key="record.id"
            style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #f0f0f0"
          >
            <div>
              <div style="font-weight: 500">{{ record.title }}</div>
              <div style="color: #999; font-size: 12px">
                {{ record.minutes }} 分钟
                {{ record.difficulty_star > 0 ? "★".repeat(record.difficulty_star) : "" }}
              </div>
            </div>
            <n-button size="small" @click="handleDelete(record.id)">删除</n-button>
          </div>
        </div>
      </n-card>
    </n-space>

    <n-modal v-model:show="showAddModal" title="添加任务" preset="card" style="width: 400px">
      <n-space vertical>
        <div>
          <label>任务描述</label>
          <n-input v-model:value="newTitle" placeholder="今天做了什么？" />
        </div>
        <div>
          <label>耗时（分钟）</label>
          <n-input-number v-model:value="newMinutes" :min="1" :max="1440" />
        </div>
        <div>
          <label>难度星级</label>
          <n-rate v-model:value="newStar" :count="3" />
          <span style="color: #999; margin-left: 8px">{{ newStar === 0 ? "未标记" : "" }}</span>
        </div>
        <n-button type="primary" block @click="handleAdd">确认添加</n-button>
      </n-space>
    </n-modal>

    <n-modal
      v-model:show="showConfirmModal"
      title="今日成长分析结果"
      preset="card"
      style="width: 560px"
    >
      <n-space vertical>
        <div
          v-for="(item, idx) in recordStore.pendingPreview"
          :key="`${item.title}-${idx}`"
          style="padding: 12px; border: 1px solid #eee; border-radius: 8px"
        >
          <strong>{{ item.title }}</strong>
          <div style="color: #666; font-size: 13px; margin-top: 4px">
            分类: {{ item.category }}
            <span style="margin-left: 8px">
              置信度: {{ (item.confidence * 100).toFixed(0) }}%
            </span>
          </div>
          <div style="margin-top: 6px">
            <template v-if="item.changes.length > 0">
              <span
                v-for="change in item.changes"
                :key="change.dimension_key"
                style="margin-right: 12px"
              >
                {{ getDimName(change.dimension_key) }} +{{ change.change_value }}
              </span>
            </template>
            <span v-else style="color: #999">无新增成长（可能已达单日上限）</span>
          </div>
          <div style="color: #999; font-size: 12px; margin-top: 4px">{{ item.reason }}</div>
        </div>

        <div
          v-if="recordStore.pendingSummary"
          style="padding: 8px; background: #f5f5f5; border-radius: 4px; font-size: 13px"
        >
          {{ recordStore.pendingSummary }}
        </div>

        <n-space justify="end">
          <n-button @click="handleCancel">取消</n-button>
          <n-button type="primary" :loading="confirming" @click="handleConfirm">
            确认写入账本
          </n-button>
        </n-space>
      </n-space>
    </n-modal>
  </div>
</template>
