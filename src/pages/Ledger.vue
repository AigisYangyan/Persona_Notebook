<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useMessage } from "naive-ui";
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

const dimLabels: Record<string, string> = {
  knowledge: '学识',
  willpower: '觉悟',
  expression: '表达',
  physique: '体魄',
  bond: '羁绊',
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
      GROWTH <span style="color: var(--cyber-cyan)">LEDGER</span>
      <span class="sub">成长账本</span>
    </h1>

    <div class="ledger-subtitle">
      记录你的每一次成长与积累。
    </div>

    <div class="ledger-panel cyber-panel">
      <div v-if="loading" class="ledger-empty">加载中...</div>
      <div v-else-if="entries.length === 0" class="ledger-empty">
        暂无评分记录，去「今日记录」页面完成评分吧
      </div>
      <div v-else class="ledger-list">
        <div
          v-for="entry in entries"
          :key="entry.id"
          class="ledger-card"
          :style="{ borderLeftColor: dimColors[entry.dimension_key] || 'var(--cyber-cyan)' }"
        >
          <div class="ledger-card-header">
            <div class="ledger-tags">
              <span
                class="dim-tag"
                :style="{
                  color: dimColors[entry.dimension_key] || 'var(--cyber-cyan)',
                  borderColor: dimColors[entry.dimension_key] || 'var(--cyber-cyan)',
                  background: (dimColors[entry.dimension_key] || 'var(--cyber-cyan)') + '15',
                }"
              >
                {{ dimLabels[entry.dimension_key] || entry.dimension_name }}
              </span>
              <span class="value-tag">
                <span class="value-icon">✦</span>
                <span class="value-num">+{{ entry.change_value }}</span>
              </span>
            </div>
            <div class="ledger-meta">
              <span class="meta-date">📅 {{ entry.date }}</span>
              <span class="meta-engine">{{ formatEngineLabel(entry.engine) }}</span>
            </div>
          </div>
          <div class="ledger-title">{{ entry.source_title }}</div>
          <div class="ledger-reason">{{ entry.reason }}</div>
          <div class="ledger-actions">
            <button class="rollback-btn" @click="handleRollback(entry.id)">
              <span>撤销</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ledger-subtitle {
  font-size: 15px;
  color: var(--cyber-text-secondary);
  font-style: italic;
  margin: -12px 0 20px 4px;
  letter-spacing: 1px;
}

.ledger-panel {
  padding: 20px 24px;
  min-height: 300px;
}

.ledger-empty {
  text-align: center;
  padding: 48px;
  color: var(--cyber-text-muted);
  font-size: 14px;
}

.ledger-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ledger-card {
  padding: 16px 20px;
  background: rgba(0, 12, 35, 0.5);
  border: 1px solid var(--cyber-border);
  border-left: 3px solid var(--cyber-cyan);
  border-radius: 0 6px 6px 0;
  transition: all 0.25s;
  position: relative;
}

.ledger-card:hover {
  background: rgba(0, 25, 55, 0.5);
  border-color: var(--cyber-border-hover);
  box-shadow: 0 0 16px rgba(0, 180, 255, 0.08);
}

.ledger-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
  flex-wrap: wrap;
  gap: 8px;
}

.ledger-tags {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dim-tag {
  padding: 3px 10px;
  border: 1px solid;
  border-radius: 3px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
}

.value-tag {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  font-weight: 700;
  color: var(--cyber-success);
}

.value-icon {
  font-size: 12px;
  color: var(--cyber-success);
  text-shadow: 0 0 4px rgba(0, 255, 170, 0.4);
}

.value-num {
  font-style: italic;
}

.ledger-meta {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 12px;
  color: var(--cyber-text-muted);
}

.meta-date {
  letter-spacing: 1px;
}

.meta-engine {
  padding: 2px 8px;
  background: rgba(0, 60, 120, 0.2);
  border-radius: 3px;
  font-size: 11px;
  letter-spacing: 1px;
}

.ledger-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--cyber-text-primary);
  margin-bottom: 6px;
}

.ledger-reason {
  font-size: 13px;
  color: var(--cyber-text-muted);
  line-height: 1.5;
  margin-bottom: 12px;
}

.ledger-actions {
  display: flex;
  justify-content: flex-end;
}

.rollback-btn {
  padding: 6px 16px;
  background: transparent;
  border: 1px solid rgba(255, 51, 102, 0.3);
  border-radius: 4px;
  color: var(--cyber-danger);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  font-family: inherit;
}

.rollback-btn:hover {
  background: rgba(255, 51, 102, 0.1);
  border-color: var(--cyber-danger);
  box-shadow: 0 0 8px rgba(255, 51, 102, 0.15);
}
</style>
