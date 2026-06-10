<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NCard, NButton, NTag, useMessage } from "naive-ui";
import { getAllLedger, rollbackLedger, type LedgerEntry } from "@/api/client/tauriCommands";
import { useStatStore } from "@/stores/statStore";

const entries = ref<LedgerEntry[]>([]);
const loading = ref(false);
const message = useMessage();
const statStore = useStatStore();

onMounted(() => {
  fetchLedger();
});

async function fetchLedger() {
  loading.value = true;
  try {
    entries.value = await getAllLedger(200);
  } catch (e: any) {
    message.error("加载账本失败: " + e.message);
  } finally {
    loading.value = false;
  }
}

async function handleRollback(id: number) {
  try {
    await rollbackLedger(id);
    message.success("已撤销该评分");
    await fetchLedger();
    await statStore.refreshStats();
  } catch (e: any) {
    message.error("撤销失败: " + e.message);
  }
}

function getDimColor(key: string): string {
  const colors: Record<string, string> = {
    knowledge: "info",
    willpower: "warning",
    expression: "success",
    physique: "error",
    bond: "default",
  };
  return colors[key] || "default";
}

function formatEngineLabel(engine: string): string {
  if (engine === "rules_api") {
    return "Rules+API";
  }
  if (engine === "api") {
    return "API";
  }
  if (engine === "local" || engine === "rules") {
    return "Rules";
  }
  return engine;
}
</script>

<template>
  <div>
    <h1 style="margin-top: 0">成长账本</h1>
    <n-card>
      <div v-if="loading" style="text-align: center; padding: 24px; color: #999">加载中...</div>
      <div v-else-if="entries.length === 0" style="text-align: center; padding: 24px; color: #999">
        暂无评分记录，去「今日记录」页面完成评分吧
      </div>
      <div v-else>
        <div
          v-for="entry in entries"
          :key="entry.id"
          style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #f0f0f0"
        >
          <div style="flex: 1">
            <div style="display: flex; align-items: center; gap: 8px">
              <n-tag :type="getDimColor(entry.dimension_key) as any" size="small">
                {{ entry.dimension_name }}
              </n-tag>
              <span style="font-weight: bold; color: #18a058">+{{ entry.change_value }}</span>
              <span style="color: #999; font-size: 12px">{{ entry.date }}</span>
              <span style="color: #999; font-size: 12px">{{ formatEngineLabel(entry.engine) }}</span>
            </div>
            <div style="margin-top: 4px; font-size: 13px">
              {{ entry.source_title }}
            </div>
            <div style="color: #999; font-size: 12px">{{ entry.reason }}</div>
          </div>
          <n-button size="small" type="error" ghost @click="handleRollback(entry.id)">撤销</n-button>
        </div>
      </div>
    </n-card>
  </div>
</template>
