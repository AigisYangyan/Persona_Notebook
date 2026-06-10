<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { NCard, NSpace, NStatistic, NProgress, NTag, NEmpty } from "naive-ui";
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
</script>

<template>
  <div>
    <h1 style="margin-top: 0">Dashboard</h1>
    <n-space vertical :size="24">
      <n-card title="总等级">
        <n-space align="center">
          <n-statistic label="等级" :value="`Lv.${statStore.totalLevel}`" />
          <n-statistic label="总经验" :value="`${statStore.totalExp} / ${statStore.nextLevelExp}`" />
          <n-progress
            type="line"
            :percentage="Math.round((statStore.totalExp / statStore.nextLevelExp) * 100)"
            style="width: 200px"
          />
          <n-statistic label="连续记录" :value="`${streak.current_streak} 天`" />
          <n-statistic label="最长连续" :value="`${streak.longest_streak} 天`" />
        </n-space>
      </n-card>

      <n-card title="五维雷达图">
        <RadarChart :data="statStore.dimensions" />
      </n-card>

      <n-card title="各维度详情">
        <n-space vertical>
          <div
            v-for="dim in statStore.dimensions"
            :key="dim.key"
            style="display: flex; align-items: center; gap: 12px"
          >
            <span style="width: 60px; font-weight: bold">{{ dim.name }}</span>
            <n-progress
              type="line"
              :percentage="Math.min((dim.value / 100) * 100, 100)"
              indicator-placement="inside"
              style="flex: 1"
            />
            <span style="width: 50px; text-align: right">{{ dim.value }}</span>
          </div>
        </n-space>
      </n-card>

      <n-card title="成就徽章">
        <div v-if="unlockedBadges.length > 0" style="margin-bottom: 16px">
          <div style="font-weight: bold; margin-bottom: 8px">已解锁</div>
          <n-space>
            <n-tag
              v-for="badge in unlockedBadges"
              :key="badge.id"
              type="success"
              size="large"
              :title="badge.desc"
            >
              {{ badge.icon }} {{ badge.name }}
            </n-tag>
          </n-space>
        </div>
        <div v-if="lockedBadges.length > 0">
          <div style="font-weight: bold; margin-bottom: 8px; color: #999">未解锁</div>
          <n-space>
            <n-tag
              v-for="badge in lockedBadges"
              :key="badge.id"
              type="default"
              size="large"
              :title="badge.desc"
              style="opacity: 0.5"
            >
              {{ badge.icon }} {{ badge.name }}
            </n-tag>
          </n-space>
        </div>
        <n-empty v-if="badges.length === 0" description="暂无徽章" />
      </n-card>
    </n-space>
  </div>
</template>
