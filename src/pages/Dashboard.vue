<script setup lang="ts">
import { ref, onMounted } from "vue";
import RadarChart from "@/components/charts/RadarChart.vue";
import { useStatStore } from "@/stores/statStore";
import { getStreakInfo, type StreakInfo } from "@/api/client/tauriCommands";

const statStore = useStatStore();
const streak = ref<StreakInfo>({ current_streak: 0, longest_streak: 0 });

onMounted(async () => {
  await statStore.refreshStats();
  streak.value = await getStreakInfo();
});

const dimIcons: Record<string, string> = {
  knowledge: '📖',
  willpower: '🛡️',
  expression: '🗣️',
  physique: '💪',
  bond: '👥',
};

const dimColors: Record<string, string> = {
  knowledge: '#00d4ff',
  willpower: '#ffcc00',
  expression: '#00ffaa',
  physique: '#ff3366',
  bond: '#9966ff',
};
</script>

<template>
  <div class="cyber-page">
    <h1 class="cyber-page-title">
      DASHBOARD<span class="sub">仪表盘</span>
    </h1>

    <div class="dashboard-grid">
      <!-- OVERVIEW -->
      <div class="dashboard-section">
        <div class="cyber-section-title">
          OVERVIEW<span class="sub">总览</span>
        </div>
        <div class="overview-panel cyber-panel">
          <div class="overview-stats">
            <div class="stat-group">
              <div class="cyber-stat-label">LEVEL 等级</div>
              <div class="cyber-stat-value">Lv.{{ statStore.totalLevel }}</div>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-group exp-group">
              <div class="cyber-stat-label">EXP 经验值</div>
              <div class="exp-row">
                <span class="exp-value">{{ statStore.totalExp }} / {{ statStore.nextLevelExp }}</span>
                <span class="exp-percent">{{ Math.round((statStore.totalExp / statStore.nextLevelExp) * 100) }}%</span>
              </div>
              <div class="cyber-progress-track" style="margin-top: 6px; width: 180px;">
                <div
                  class="cyber-progress-fill"
                  :style="{ width: `${Math.min((statStore.totalExp / statStore.nextLevelExp) * 100, 100)}%` }"
                ></div>
              </div>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-group">
              <div class="cyber-stat-label">STREAK 连续记录</div>
              <div class="cyber-stat-value">{{ streak.current_streak }}<span class="unit">天</span></div>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-group">
              <div class="cyber-stat-label">BEST 最长连续</div>
              <div class="cyber-stat-value">{{ streak.longest_streak }}<span class="unit">天</span></div>
            </div>
          </div>
        </div>
      </div>

      <!-- RADAR + DETAILS -->
      <div class="dashboard-split">
        <div class="dashboard-section radar-section">
          <div class="cyber-section-title">
            DIMENSION RADAR<span class="sub">五维雷达图</span>
          </div>
          <div class="radar-panel cyber-panel">
            <RadarChart :data="statStore.dimensions" />
          </div>
        </div>

        <div class="dashboard-section details-section">
          <div class="cyber-section-title">
            DIMENSION DETAILS<span class="sub">各维度详情</span>
          </div>
          <div class="details-panel cyber-panel">
            <div
              v-for="dim in statStore.dimensions"
              :key="dim.key"
              class="dim-row"
            >
              <span class="dim-icon">{{ dimIcons[dim.key] || '◆' }}</span>
              <span class="dim-name">{{ dim.name }}</span>
              <span class="dim-en">{{ dim.key.toUpperCase() }}</span>
              <div class="cyber-progress-track" style="flex: 1; margin: 0 12px;">
                <div
                  class="cyber-progress-fill"
                  :style="{ width: `${Math.min((dim.value / 100) * 100, 100)}%`, background: `linear-gradient(90deg, ${dimColors[dim.key] || 'var(--cyber-cyan)'}88, ${dimColors[dim.key] || 'var(--cyber-cyan)'})` }"
                ></div>
              </div>
              <span class="dim-percent">{{ Math.round((dim.value / 100) * 100) }}%</span>
              <span class="dim-value">{{ dim.value }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-grid {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.dashboard-section {
  position: relative;
}

.dashboard-split {
  display: grid;
  grid-template-columns: minmax(0, 2fr) minmax(300px, 1fr);
  gap: 24px;
  align-items: start;
}

/* Overview Panel */
.overview-panel {
  padding: 20px 24px;
}

.overview-stats {
  display: flex;
  align-items: center;
  gap: 0;
}

.stat-group {
  display: flex;
  flex-direction: column;
  padding: 0 24px;
  min-width: 100px;
}

.stat-group:first-child {
  padding-left: 0;
}

.stat-divider {
  width: 1px;
  height: 50px;
  background: linear-gradient(180deg, transparent, var(--cyber-border), transparent);
}

.exp-group {
  flex: 1;
  min-width: 200px;
}

.exp-row {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.exp-value {
  font-size: 20px;
  font-weight: 700;
  font-style: italic;
  color: var(--cyber-text-primary);
}

.exp-percent {
  font-size: 13px;
  color: var(--cyber-cyan);
  font-weight: 600;
}

.unit {
  font-size: 18px;
  font-weight: 500;
  font-style: normal;
  color: var(--cyber-text-muted);
  margin-left: 4px;
}

/* Radar Panel */
.radar-panel {
  padding: 24px;
  min-height: 560px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.radar-panel :deep(.chart) {
  height: 540px;
}

/* Details Panel */
.details-panel {
  padding: 18px 22px;
  min-height: 560px;
}

.dim-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 0;
  border-bottom: 1px solid rgba(0, 60, 120, 0.15);
}

.dim-row:last-child {
  border-bottom: none;
}

.dim-icon {
  font-size: 18px;
  width: 26px;
  text-align: center;
}

.dim-name {
  width: 50px;
  font-weight: 700;
  color: var(--cyber-text-primary);
  font-size: 15px;
}

.dim-en {
  font-size: 11px;
  color: var(--cyber-text-muted);
  letter-spacing: 1px;
  text-transform: uppercase;
  width: 80px;
}

.dim-percent {
  font-size: 13px;
  font-weight: 700;
  color: var(--cyber-cyan);
  width: 40px;
  text-align: right;
}

.dim-value {
  font-size: 15px;
  font-weight: 800;
  color: var(--cyber-text-secondary);
  width: 44px;
  text-align: right;
}

@media (max-width: 1100px) {
  .dashboard-split {
    grid-template-columns: 1fr;
  }

  .overview-stats {
    flex-wrap: wrap;
    gap: 16px;
  }

  .stat-divider {
    display: none;
  }

  .stat-group {
    padding: 0;
    min-width: 120px;
  }
}
</style>
