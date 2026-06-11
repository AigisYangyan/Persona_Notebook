<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
// Naive UI imports removed - using custom cyberpunk styling
import RadarChart from "@/components/charts/RadarChart.vue";
import { useStatStore } from "@/stores/statStore";
import { getStreakInfo, getAllLedger, type StreakInfo, type LedgerEntry } from "@/api/client/tauriCommands";

const statStore = useStatStore();
const streak = ref<StreakInfo>({ current_streak: 0, longest_streak: 0 });
const recentLedger = ref<LedgerEntry[]>([]);

onMounted(async () => {
  await statStore.refreshStats();
  streak.value = await getStreakInfo();
  recentLedger.value = await getAllLedger(50);
});

interface Badge {
  id: string;
  icon: string;
  name: string;
  desc: string;
  unlocked: boolean;
}

const badges = computed<Badge[]>(() => {
  const dims = statStore.dimensions;
  const hasRecords = recentLedger.value.length > 0;
  const hasHybridScore = recentLedger.value.some(
    (entry) => entry.engine === "rules_api" || entry.engine === "api"
  );
  const knowledgeTotal = dims.find((d) => d.key === "knowledge")?.value ?? 0;
  const physiqueTotal = dims.find((d) => d.key === "physique")?.value ?? 0;

  return [
    { id: "first_record", icon: "📝", name: "初次记录", desc: "添加了第一条任务", unlocked: hasRecords },
    { id: "streak_3", icon: "🔥", name: "连续3天", desc: "连续记录3天", unlocked: streak.value.current_streak >= 3 },
    { id: "streak_7", icon: "🔥🔥", name: "连续7天", desc: "连续记录7天", unlocked: streak.value.current_streak >= 7 },
    { id: "knowledge_50", icon: "🧠", name: "学识突破", desc: "学识维度达到50分", unlocked: knowledgeTotal >= 50 },
    { id: "physique_50", icon: "💪", name: "体魄突破", desc: "体魄维度达到50分", unlocked: physiqueTotal >= 50 },
    { id: "first_score", icon: "🎯", name: "首次评分", desc: "完成第一次成长评分", unlocked: hasRecords },
    { id: "ai_score", icon: "🤖", name: "反馈评分", desc: "完成一次 Rules + API 反馈评分", unlocked: hasHybridScore },
  ];
});

const unlockedBadges = computed(() => badges.value.filter((b) => b.unlocked));
const lockedBadges = computed(() => badges.value.filter((b) => !b.unlocked));

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

      <!-- DIMENSION RADAR -->
      <div class="dashboard-section">
        <div class="cyber-section-title">
          DIMENSION RADAR<span class="sub">五维雷达图</span>
        </div>
        <div class="radar-panel cyber-panel">
          <RadarChart :data="statStore.dimensions" />
        </div>
      </div>

      <!-- DIMENSION DETAILS -->
      <div class="dashboard-section">
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

      <!-- ACHIEVEMENTS -->
      <div class="dashboard-section">
        <div class="cyber-section-title">
          ACHIEVEMENTS<span class="sub">成就徽章</span>
        </div>
        <div class="achievements-panel cyber-panel">
          <div v-if="unlockedBadges.length > 0" class="badge-section">
            <div class="badge-section-title">已解锁 UNLOCKED</div>
            <div class="badge-list">
              <div
                v-for="badge in unlockedBadges"
                :key="badge.id"
                class="cyber-badge unlocked"
                :title="badge.desc"
              >
                <span class="badge-icon">{{ badge.icon }}</span>
                <span>{{ badge.name }}</span>
              </div>
            </div>
          </div>
          <div v-if="lockedBadges.length > 0" class="badge-section">
            <div class="badge-section-title" style="color: var(--cyber-text-dim);">未解锁 LOCKED</div>
            <div class="badge-list">
              <div
                v-for="badge in lockedBadges"
                :key="badge.id"
                class="cyber-badge locked"
                :title="badge.desc"
              >
                <span class="badge-icon">🔒</span>
                <span>{{ badge.name }}</span>
              </div>
            </div>
          </div>
          <div v-if="badges.length === 0" style="text-align: center; padding: 24px; color: var(--cyber-text-muted);">
            暂无徽章
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
  padding: 20px;
  min-height: 400px;
}

.radar-panel :deep(.chart) {
  height: 380px;
}

/* Details Panel */
.details-panel {
  padding: 16px 24px;
}

.dim-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(0, 60, 120, 0.15);
}

.dim-row:last-child {
  border-bottom: none;
}

.dim-icon {
  font-size: 16px;
  width: 24px;
  text-align: center;
}

.dim-name {
  width: 50px;
  font-weight: 600;
  color: var(--cyber-text-primary);
  font-size: 14px;
}

.dim-en {
  font-size: 10px;
  color: var(--cyber-text-dim);
  letter-spacing: 1px;
  text-transform: uppercase;
  width: 80px;
}

.dim-percent {
  font-size: 12px;
  font-weight: 600;
  color: var(--cyber-cyan);
  width: 36px;
  text-align: right;
}

.dim-value {
  font-size: 14px;
  font-weight: 700;
  color: var(--cyber-text-secondary);
  width: 40px;
  text-align: right;
}

/* Achievements Panel */
.achievements-panel {
  padding: 20px 24px;
}

.badge-section {
  margin-bottom: 16px;
}

.badge-section:last-child {
  margin-bottom: 0;
}

.badge-section-title {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
  color: var(--cyber-text-secondary);
  margin-bottom: 12px;
}

.badge-list {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.badge-icon {
  font-size: 16px;
}
</style>
