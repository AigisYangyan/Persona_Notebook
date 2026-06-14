<script setup lang="ts">
import { NButton, NInput, NModal } from "naive-ui";
import type { PlanAiOutcome } from "@/api/client/tauriCommands";

defineProps<{
  show: boolean;
  outcome: PlanAiOutcome | null;
  answers: string[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  "update:show": [value: boolean];
  "update-answer": [payload: { index: number; value: string }];
  submitAnswers: [];
  apply: [];
  close: [];
}>();

function handleClose() {
  emit("update:show", false);
  emit("close");
}
</script>

<template>
  <n-modal :show="show" preset="card" class="cyber-modal" style="max-width: 760px" @close="handleClose">
    <template #header>
      <div class="dialog-title">AI Goal Review</div>
    </template>

    <div v-if="outcome?.requires_clarification" class="dialog-body">
      <div class="dialog-copy">AI 还不能稳定理解这组 Goal，先把你的真实意图问清楚再调整。</div>
      <div v-for="(question, index) in outcome.questions" :key="`${index}-${question}`" class="qa-block">
        <div class="question">{{ question }}</div>
        <n-input
          :value="answers[index] ?? ''"
          type="textarea"
          placeholder="在这里补充你的真实意图"
          :autosize="{ minRows: 2, maxRows: 4 }"
          @update:value="(value) => emit('update-answer', { index, value })"
        />
      </div>
      <div class="dialog-actions">
        <n-button @click="handleClose">关闭</n-button>
        <n-button type="primary" :loading="loading" @click="emit('submitAnswers')">提交回答</n-button>
      </div>
    </div>

    <div v-else-if="outcome?.proposal" class="dialog-body">
      <div class="dialog-copy">{{ outcome.proposal.ai_summary }}</div>
      <div class="proposal-block">
        <div class="proposal-title">{{ outcome.proposal.title }}</div>
        <div class="proposal-summary">{{ outcome.proposal.summary }}</div>
      </div>
      <div class="proposal-list">
        <div
          v-for="item in outcome.proposal.items"
          :key="`${item.sort_order}-${item.title}`"
          class="proposal-item"
        >
          <div class="proposal-item-head">
            <span>{{ item.title }}</span>
            <span>{{ item.progress_percent }}%</span>
          </div>
          <div class="proposal-item-desc">{{ item.description }}</div>
          <div class="proposal-item-comment">{{ item.ai_comment }}</div>
        </div>
      </div>
      <div class="dialog-actions">
        <n-button @click="handleClose">稍后再说</n-button>
        <n-button type="primary" :loading="loading" @click="emit('apply')">应用修改</n-button>
      </div>
    </div>
  </n-modal>
</template>

<style scoped>
.dialog-title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--cyber-cyan);
}

.dialog-body {
  display: grid;
  gap: 16px;
}

.dialog-copy,
.proposal-summary,
.proposal-item-desc,
.proposal-item-comment {
  font-size: 13px;
  line-height: 1.6;
  color: var(--cyber-text-secondary);
}

.qa-block,
.proposal-item,
.proposal-block {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid rgba(0, 180, 255, 0.18);
  background: rgba(0, 28, 60, 0.28);
}

.question,
.proposal-title {
  font-weight: 700;
  color: var(--cyber-text-primary);
}

.proposal-list {
  display: grid;
  gap: 10px;
}

.proposal-item-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  color: var(--cyber-cyan);
  font-size: 12px;
  font-weight: 700;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
