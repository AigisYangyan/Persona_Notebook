<script setup lang="ts">
import { NProgress } from "naive-ui";
import type { GoalProgressSummary } from "@/api/client/tauriCommands";

defineProps<{
  title: string;
  summary: GoalProgressSummary;
  description?: string;
}>();
</script>

<template>
  <div class="goal-progress cyber-panel">
    <div class="goal-progress-head">
      <div>
        <div class="goal-progress-title">{{ title }}</div>
        <div v-if="description" class="goal-progress-description">{{ description }}</div>
      </div>
      <div class="goal-progress-percent">{{ summary.average_progress_percent }}%</div>
    </div>

    <n-progress
      type="line"
      :percentage="summary.average_progress_percent"
      :show-indicator="false"
      :height="10"
    />

    <div class="goal-progress-metrics">
      <span>{{ summary.total_items }} 个 Goal</span>
      <span>{{ summary.completed_items }} 已完成</span>
      <span>{{ summary.active_goal_count }} 进行中</span>
    </div>
  </div>
</template>

<style scoped>
.goal-progress {
  padding: 16px;
  display: grid;
  gap: 12px;
}

.goal-progress-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.goal-progress-title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 1px;
  text-transform: uppercase;
  color: var(--cyber-cyan);
}

.goal-progress-description {
  margin-top: 4px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--cyber-text-muted);
}

.goal-progress-percent {
  font-size: 24px;
  font-weight: 800;
  color: var(--cyber-text-primary);
}

.goal-progress-metrics {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  font-size: 13px;
  color: var(--cyber-text-muted);
}

@media (max-width: 760px) {
  .goal-progress-head {
    flex-direction: column;
  }
}
</style>
