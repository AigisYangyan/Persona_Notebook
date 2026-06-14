<script setup lang="ts">
import { computed } from "vue";
import { NProgress } from "naive-ui";
import type { PlanGrowthSnapshot } from "@/api/client/tauriCommands";
import { summarizeGrowth } from "@/features/plans/periods";

const props = defineProps<{
  growth: PlanGrowthSnapshot;
  title: string;
}>();

const summaryLine = computed(() => summarizeGrowth(props.growth.dimensions));
</script>

<template>
  <div class="growth-panel cyber-panel">
    <div class="growth-head">
      <div>
        <div class="growth-title">{{ title }}</div>
        <div class="growth-summary">{{ summaryLine }}</div>
      </div>
      <div class="growth-metrics">
        <span>{{ growth.active_days }} 活跃日</span>
        <span>{{ growth.analyzed_days }} 已评分日</span>
        <span>{{ growth.record_count }} 条记录</span>
      </div>
    </div>

    <div class="growth-bars">
      <div
        v-for="dimension in growth.dimensions"
        :key="dimension.key"
        class="growth-row"
      >
        <div class="growth-labels">
          <span class="dim-name">{{ dimension.name }}</span>
          <span class="dim-meta">{{ dimension.total }} / {{ dimension.max_total }}</span>
        </div>
        <n-progress
          type="line"
          :percentage="dimension.progress_percent"
          :show-indicator="false"
          :height="10"
        />
        <div class="growth-headline">{{ dimension.headline }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.growth-panel {
  padding: 18px;
  display: grid;
  gap: 16px;
}

.growth-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

.growth-title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--cyber-cyan);
  text-transform: uppercase;
}

.growth-summary {
  margin-top: 6px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--cyber-text-secondary);
}

.growth-metrics {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: flex-end;
  font-size: 13px;
  color: var(--cyber-text-muted);
}

.growth-bars {
  display: grid;
  gap: 12px;
}

.growth-row {
  display: grid;
  gap: 6px;
}

.growth-labels {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}

.dim-name {
  color: var(--cyber-text-primary);
  font-weight: 600;
}

.dim-meta,
.growth-headline {
  color: var(--cyber-text-muted);
  font-size: 13px;
}

@media (max-width: 760px) {
  .growth-head {
    flex-direction: column;
  }

  .growth-metrics {
    justify-content: flex-start;
  }
}
</style>
