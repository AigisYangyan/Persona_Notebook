<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { NButton, NEmpty, NModal, NPopconfirm, useMessage } from "naive-ui";
import { useInsightStore } from "@/stores/insightStore";
import type { InsightReport } from "@/api/client/tauriCommands";
import { getTodayStr } from "@/utils/date";
import { coerceInsightList, coerceInsightText } from "@/features/insights/display";

const insightStore = useInsightStore();
const message = useMessage();
const selectedDate = ref(getTodayStr());
const contextVisible = ref(false);

const activeReport = computed(() => insightStore.currentTarot);
const tarotPayload = computed(() => readReportPayload(activeReport.value));

const reportText = computed(() =>
  buildTarotText({
    quote: readString(tarotPayload.value.warm_quote),
    encouragement: readString(tarotPayload.value.encouragement, activeReport.value?.summary || ""),
    theme: readString(tarotPayload.value.psychological_theme),
    body: readString(tarotPayload.value.body_signal),
    deep: readString(tarotPayload.value.deeper_reading),
    action: readList(tarotPayload.value.action),
    risk: readString(tarotPayload.value.risk_reminder),
  }),
);

const reportDense = computed(() => reportText.value.join("").length > 1000);

onMounted(async () => {
  await insightStore.loadTarotCabinet();
  if (!insightStore.currentTarot && insightStore.tarotCabinet.length > 0) {
    insightStore.selectTarot(insightStore.tarotCabinet[0]);
  }
});

async function generateTarot() {
  try {
    await insightStore.generateTarot(selectedDate.value);
    message.success("今日塔罗已生成");
  } catch (error) {
    message.error(readError(error, "生成失败"));
  }
}

async function openContext(report: InsightReport) {
  if (!report.context_snapshot_id) {
    return;
  }
  await insightStore.loadContextSnapshot(report.context_snapshot_id);
  contextVisible.value = true;
}

async function deleteReport(report: InsightReport) {
  try {
    await insightStore.removeReport(report.id, "tarot", "day");
    message.success("已删除这条塔罗记录");
  } catch (error) {
    message.error(readError(error, "删除失败"));
  }
}

function readReportPayload(report: InsightReport | null): Record<string, unknown> {
  if (!report || typeof report.content_json !== "object" || report.content_json === null) {
    return {};
  }
  const content = report.content_json as Record<string, unknown>;
  const payload = content.report;
  return typeof payload === "object" && payload !== null ? (payload as Record<string, unknown>) : content;
}

function readString(value: unknown, fallback = ""): string {
  return coerceInsightText(value, fallback);
}

function readList(value: unknown): string[] {
  return coerceInsightList(value);
}

function buildTarotText(input: {
  quote: string;
  encouragement: string;
  theme: string;
  body: string;
  deep: string;
  action: string[];
  risk: string;
}): string[] {
  const paragraphs = [
    input.quote,
    input.encouragement,
    input.theme ? `心理主题：${input.theme}` : "",
    input.body ? `体力信号：${input.body}` : "",
    input.deep ? `深层解读：${input.deep}` : "",
    input.action.length > 0 ? `行动建议：${input.action.join("；")}` : "",
    input.risk ? `风险提醒：${input.risk}` : "",
  ].filter(Boolean);

  return paragraphs.length > 0 ? paragraphs : ["生成后，这里会出现一份完整的单框塔罗报告。"];
}

function readError(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : String(error || fallback);
}
</script>

<template>
  <div class="cyber-page tarot-page">
    <header class="page-head">
      <h1 class="cyber-page-title">TAROT<span class="sub">心理原型牌</span></h1>
      <div class="head-actions">
        <input v-model="selectedDate" class="cyber-date" type="date" />
        <n-button type="primary" :loading="insightStore.generating" @click="generateTarot">生成塔罗</n-button>
      </div>
    </header>

    <div class="tarot-layout">
      <section class="cyber-panel tarot-report" :class="{ dense: reportDense }">
        <div class="report-topline">
          <div class="report-mark">{{ readString(tarotPayload.card_mark, "XI") }}</div>
          <div class="report-heading">
            <h2>{{ readString(tarotPayload.card_name, "等待抽取") }}</h2>
            <div class="report-archetype">
              {{ readString(tarotPayload.archetype, "Psychological Archetype") }}
            </div>
          </div>
          <n-button v-if="activeReport?.context_snapshot_id" size="small" @click="openContext(activeReport)">
            查看 Context
          </n-button>
        </div>

        <div class="report-content">
          <p v-for="paragraph in reportText" :key="paragraph">{{ paragraph }}</p>
        </div>
      </section>

      <aside class="cyber-panel cabinet">
        <div class="cabinet-title">收纳柜</div>
        <div v-if="insightStore.tarotCabinet.length === 0" class="empty-box">
          <n-empty description="暂无塔罗记录" />
        </div>
        <div
          v-for="report in insightStore.tarotCabinet"
          :key="report.id"
          class="cabinet-item"
          :class="{ active: activeReport?.id === report.id }"
          @click="insightStore.selectTarot(report)"
        >
          <span>{{ report.start_date }}</span>
          <strong>{{ report.title || "心理原型牌" }}</strong>
          <small>{{ report.summary }}</small>
          <n-popconfirm @positive-click="deleteReport(report)">
            <template #trigger>
              <n-button size="tiny" quaternary type="error" @click.stop>删除</n-button>
            </template>
            删除这条塔罗记录？
          </n-popconfirm>
        </div>
      </aside>
    </div>

    <n-modal v-model:show="contextVisible" preset="card" title="AI Context" class="context-modal">
      <pre>{{ insightStore.contextSnapshot?.context_json }}</pre>
    </n-modal>
  </div>
</template>

<style scoped>
.tarot-page {
  display: grid;
  gap: 16px;
}

.page-head,
.head-actions,
.report-topline {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.cyber-date {
  width: 160px;
  height: 34px;
  padding: 0 10px;
  border: 1px solid var(--cyber-border);
  background: var(--cyber-bg-input);
  color: var(--cyber-text-primary);
}

.tarot-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 216px;
  gap: 14px;
  align-items: start;
}

.tarot-report,
.cabinet {
  padding: 14px;
}

.tarot-report {
  min-width: 0;
  display: grid;
  gap: 14px;
  min-height: calc(100vh - 210px);
  background:
    radial-gradient(circle at 18% 16%, rgba(0, 255, 170, 0.08), transparent 22%),
    radial-gradient(circle at 84% 18%, rgba(0, 212, 255, 0.08), transparent 24%),
    rgba(5, 12, 28, 0.94);
}

.report-mark,
.cabinet-title,
.cabinet-item span,
.cabinet-item small {
  color: var(--cyber-text-muted);
  letter-spacing: 1.6px;
  font-size: 11px;
}

.report-mark {
  width: 40px;
  font-size: 18px;
  font-weight: 800;
  text-align: center;
}

.report-heading {
  flex: 1;
  min-width: 0;
}

.report-heading h2 {
  margin: 0 0 4px;
  color: var(--cyber-text-primary);
  font-size: 28px;
  line-height: 1.15;
}

.report-archetype {
  color: var(--cyber-success);
  font-size: 13px;
  line-height: 1.4;
}

.report-content {
  display: grid;
  gap: 10px;
}

.report-content p {
  margin: 0;
  color: var(--cyber-text-secondary);
  line-height: 1.56;
  font-size: 14px;
}

.dense .report-content p {
  font-size: 13px;
  line-height: 1.48;
}

.cabinet {
  display: grid;
  gap: 8px;
}

.cabinet-title {
  color: var(--cyber-cyan);
  font-weight: 800;
}

.cabinet-item {
  display: grid;
  gap: 4px;
  padding: 8px;
  border: 1px solid var(--cyber-border);
  background: rgba(8, 18, 38, 0.78);
  color: var(--cyber-text-primary);
  cursor: pointer;
}

.cabinet-item.active,
.cabinet-item:hover {
  border-color: var(--cyber-cyan);
}

.cabinet-item strong,
.cabinet-item small {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.cabinet-item strong {
  -webkit-line-clamp: 2;
}

.cabinet-item small {
  -webkit-line-clamp: 2;
}

.empty-box {
  min-height: 120px;
  display: grid;
  place-items: center;
}

.context-modal pre {
  max-height: 70vh;
  overflow: auto;
  white-space: pre-wrap;
}

@media (max-width: 920px) {
  .tarot-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .page-head,
  .head-actions,
  .report-topline {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
