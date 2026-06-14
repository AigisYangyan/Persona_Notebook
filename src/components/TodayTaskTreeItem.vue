<script setup lang="ts">
import { computed } from "vue";
import type { TaskTreeItem } from "@/features/records/taskMetrics";
import { formatDuration } from "@/features/records/taskMetrics";

defineOptions({
  name: "TodayTaskTreeItem",
});

const props = defineProps<{
  node: TaskTreeItem;
  depth?: number;
}>();

const emit = defineEmits<{
  (event: "toggle-completed", payload: { id: number; isCompleted: boolean }): void;
  (event: "start", id: number): void;
  (event: "pause", id: number): void;
  (event: "reset", id: number): void;
  (event: "add-subtask", id: number): void;
  (event: "remove", id: number): void;
}>();

const indentStyle = computed(() => ({
  marginLeft: `${(props.depth ?? 0) * 18}px`,
}));
</script>

<template>
  <div class="task-node" :style="indentStyle">
    <div
      class="task-row"
      :class="{
        completed: node.record.is_completed,
        running: node.isRunning,
        nested: (depth ?? 0) > 0,
      }"
    >
      <button
        class="check-btn"
        type="button"
        @click="
          emit('toggle-completed', {
            id: node.record.id,
            isCompleted: !node.record.is_completed,
          })
        "
      >
        <span v-if="node.record.is_completed">✓</span>
      </button>

      <div class="task-main">
        <div class="task-title-row">
          <span class="task-title">{{ node.record.title }}</span>
          <span v-if="node.children.length > 0" class="task-chip muted">
            {{ node.children.length }} 个子任务
          </span>
          <span v-if="node.record.is_completed" class="task-chip success">已完成</span>
        </div>
        <div class="task-subtitle">
          SELF {{ formatDuration(node.selfElapsedSeconds) }}
          <span v-if="node.children.length > 0"> / TOTAL {{ formatDuration(node.totalElapsedSeconds) }}</span>
        </div>
      </div>

      <div class="task-actions">
        <button
          v-if="!node.isRunning"
          class="action-btn primary"
          type="button"
          @click="emit('start', node.record.id)"
        >
          开始
        </button>
        <button
          v-else
          class="action-btn warning"
          type="button"
          @click="emit('pause', node.record.id)"
        >
          暂停
        </button>
        <button class="action-btn" type="button" @click="emit('reset', node.record.id)">
          归零
        </button>
        <button class="action-btn" type="button" @click="emit('add-subtask', node.record.id)">
          子任务
        </button>
        <button class="action-btn danger" type="button" @click="emit('remove', node.record.id)">
          删除
        </button>
      </div>
    </div>

    <div v-if="node.children.length > 0" class="children-list">
      <TodayTaskTreeItem
        v-for="child in node.children"
        :key="child.record.id"
        :node="child"
        :depth="(depth ?? 0) + 1"
        @toggle-completed="emit('toggle-completed', $event)"
        @start="emit('start', $event)"
        @pause="emit('pause', $event)"
        @reset="emit('reset', $event)"
        @add-subtask="emit('add-subtask', $event)"
        @remove="emit('remove', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.task-node {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-row {
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr) auto;
  gap: 14px;
  align-items: center;
  padding: 14px 16px;
  background: rgba(0, 15, 40, 0.42);
  border: 1px solid var(--cyber-border);
  border-left: 3px solid var(--cyber-cyan);
  border-radius: 4px;
}

.task-row.nested {
  background: rgba(0, 15, 40, 0.3);
}

.task-row.running {
  box-shadow: 0 0 0 1px rgba(0, 212, 255, 0.2), inset 0 0 18px rgba(0, 212, 255, 0.05);
}

.task-row.completed {
  opacity: 0.78;
  border-left-color: var(--cyber-success);
}

.check-btn {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: 1px solid var(--cyber-border);
  background: rgba(0, 20, 50, 0.65);
  color: var(--cyber-success);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.task-main {
  min-width: 0;
}

.task-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.task-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--cyber-text-primary);
}

.task-subtitle {
  margin-top: 5px;
  font-size: 12px;
  color: var(--cyber-text-muted);
  letter-spacing: 1px;
}

.task-chip {
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 11px;
  color: var(--cyber-cyan);
  border: 1px solid rgba(0, 212, 255, 0.25);
  background: rgba(0, 212, 255, 0.08);
}

.task-chip.muted {
  color: var(--cyber-text-secondary);
  border-color: rgba(160, 196, 232, 0.18);
  background: rgba(160, 196, 232, 0.06);
}

.task-chip.success {
  color: var(--cyber-success);
  border-color: rgba(0, 255, 170, 0.26);
  background: rgba(0, 255, 170, 0.08);
}

.task-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.action-btn {
  min-width: 58px;
  padding: 7px 10px;
  border-radius: 4px;
  border: 1px solid var(--cyber-border);
  background: rgba(0, 60, 120, 0.16);
  color: var(--cyber-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(0, 180, 255, 0.12);
  border-color: var(--cyber-border-hover);
  color: var(--cyber-text-primary);
}

.action-btn.primary {
  border-color: rgba(0, 212, 255, 0.45);
  color: var(--cyber-cyan);
}

.action-btn.warning {
  border-color: rgba(255, 204, 0, 0.45);
  color: var(--cyber-warning);
}

.action-btn.danger {
  border-color: rgba(255, 51, 102, 0.45);
  color: var(--cyber-danger);
}

.children-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

@media (max-width: 900px) {
  .task-row {
    grid-template-columns: 28px minmax(0, 1fr);
  }

  .task-actions {
    grid-column: 1 / -1;
    justify-content: flex-start;
    padding-left: 42px;
  }
}
</style>
