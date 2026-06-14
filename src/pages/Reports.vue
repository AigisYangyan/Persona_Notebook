<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NButton, NEmpty, NModal, NPopconfirm, NTabPane, NTabs, useMessage } from "naive-ui";
import { useInsightStore } from "@/stores/insightStore";
import type { InsightPeriodType, InsightReport } from "@/api/client/tauriCommands";
import { getTodayStr } from "@/utils/date";
import { normalizeInsightPeriod, reportKindLabel } from "@/features/insights/periods";

const insightStore = useInsightStore();
const message = useMessage();
const selectedPeriod = ref<InsightPeriodType>("day");
const anchorDate = ref(getTodayStr());
const contextVisible = ref(false);

const activeReport = computed(() => insightStore.currentReport);
const reportPayload = computed(() => readReportPayload(activeReport.value));
const periodLabel = computed(() => reportKindLabel("report", selectedPeriod.value));

const reportText = computed(() =>
  buildReportText({
    summary:
      activeReport.value?.summary ||
      "报告会结合任务、成长账本、日记、羁绊、周月计划和长期记忆，不只做复盘，也会继续向下挖原因与下一步。",
    completed: readList(reportPayload.value.completed),
    unfinished: readList(reportPayload.value.unfinished),
    timeFocus: readText(reportPayload.value.time_focus),
    growth: readText(reportPayload.value.growth_changes),
    plan: readText(reportPayload.value.plan_progress),
    bond: readText(reportPayload.value.journal_and_bond_observations),
    root: readText(reportPayload.value.root_causes),
    leverage: readText(reportPayload.value.leverage_points),
    remedies: readList(reportPayload.value.concrete_remedies),
    next: readList(reportPayload.value.next_actions),
    gaps: readText(reportPayload.value.not_enough_data),
  }),
);

const reportDense = computed(() => reportText.value.join("").length > 1500);

watch(selectedPeriod, async (period) => {
  await insightStore.loadReportCabinet(normalizeInsightPeriod(period));
});

onMounted(async () => {
  await insightStore.loadReportCabinet(selectedPeriod.value);
  if (!insightStore.currentReport && insightStore.reportCabinet.length > 0) {
    insightStore.selectReport(insightStore.reportCabinet[0]);
  }
});

async function generateReport() {
  try {
    await insightStore.generateReport(selectedPeriod.value, anchorDate.value);
    message.success(`${periodLabel.value}已生成`);
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
    await insightStore.removeReport(report.id, "report", report.period_type);
    message.success("已删除这条报告记录");
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

function readList(value: unknown): string[] {
  if (Array.isArray(value)) {
    return value.map((item) => (typeof item === "string" ? item.trim() : JSON.stringify(item))).filter(Boolean);
  }
  return typeof value === "string" && value.trim() ? [value.trim()] : [];
}

function readText(value: unknown): string {
  if (typeof value === "string" && value.trim()) {
    return value.trim();
  }
  if (Array.isArray(value) && value.length > 0) {
    return readList(value).join("；");
  }
  return "";
}

function buildReportText(input: {
  summary: string;
  completed: string[];
  unfinished: string[];
  timeFocus: string;
  growth: string;
  plan: string;
  bond: string;
  root: string;
  leverage: string;
  remedies: string[];
  next: string[];
  gaps: string;
}): string[] {
  const paragraphs = [
    input.summary,
    input.completed.length > 0 ? `已完成：${input.completed.join("；")}` : "",
    input.unfinished.length > 0 ? `未完成：${input.unfinished.join("；")}` : "",
    input.timeFocus ? `时间重心：${input.timeFocus}` : "",
    input.growth ? `成长变化：${input.growth}` : "",
    input.plan ? `计划推进：${input.plan}` : "",
    input.bond ? `情绪与关系：${input.bond}` : "",
    input.root ? `问题根因：${input.root}` : "",
    input.leverage ? `关键支点：${input.leverage}` : "",
    input.remedies.length > 0 ? `解决措施：${input.remedies.join("；")}` : "",
    input.next.length > 0 ? `下一步：${input.next.join("；")}` : "",
    input.gaps ? `数据缺口：${input.gaps}` : "",
  ].filter(Boolean);

  return paragraphs.length > 0 ? paragraphs : ["生成后，这里会出现一份完整的单框报告。"];
}

function readError(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : String(error || fallback);
}
</script>

<template>
  <div class="cyber-page reports-page">
    <header class="page-head">
      <h1 class="cyber-page-title">REPORTS<span class="sub">生成报告</span></h1>
      <div class="head-actions">
        <input v-model="anchorDate" class="cyber-date" type="date" />
        <n-button type="primary" :loading="insightStore.generating" @click="generateReport">
          生成{{ periodLabel }}
        </n-button>
      </div>
    </header>

    <n-tabs v-model:value="selectedPeriod" class="period-tabs" type="segment">
      <n-tab-pane name="day" tab="每日" />
      <n-tab-pane name="week" tab="每周" />
      <n-tab-pane name="month" tab="每月" />
    </n-tabs>

    <div class="reports-layout">
      <section class="cyber-panel report-sheet" :class="{ dense: reportDense }">
        <div class="sheet-head">
          <div class="sheet-heading">
            <div class="report-kind">
              {{ activeReport ? reportKindLabel("report", activeReport.period_type) : periodLabel }}
            </div>
            <h2>{{ activeReport?.title || "等待生成报告" }}</h2>
          </div>
          <n-button v-if="activeReport?.context_snapshot_id" size="small" @click="openContext(activeReport)">
            查看 Context
          </n-button>
        </div>

        <div class="sheet-content">
          <p v-for="paragraph in reportText" :key="paragraph">{{ paragraph }}</p>
        </div>
      </section>

      <aside class="cyber-panel cabinet">
        <div class="cabinet-title">收纳柜</div>
        <div v-if="insightStore.reportCabinet.length === 0" class="empty-box">
          <n-empty description="暂无报告记录" />
        </div>
        <div
          v-for="report in insightStore.reportCabinet"
          :key="report.id"
          class="cabinet-item"
          :class="{ active: activeReport?.id === report.id }"
          @click="insightStore.selectReport(report)"
        >
          <span>
            {{ report.start_date }}<template v-if="report.end_date !== report.start_date"> - {{ report.end_date }}</template>
          </span>
          <strong>{{ report.title || reportKindLabel("report", report.period_type) }}</strong>
          <small>{{ report.summary }}</small>
          <n-popconfirm @positive-click="deleteReport(report)">
            <template #trigger>
              <n-button size="tiny" quaternary type="error" @click.stop>删除</n-button>
            </template>
            删除这条报告记录？
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
.reports-page {
  display: grid;
  gap: 14px;
}

.page-head,
.head-actions,
.sheet-head {
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

.period-tabs {
  max-width: 420px;
}

.period-tabs :deep(.n-tabs-nav) {
  margin-bottom: 0;
}

.period-tabs :deep(.n-tabs-pane-wrapper) {
  display: none;
}

.reports-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 216px;
  gap: 14px;
  align-items: start;
}

.report-sheet,
.cabinet {
  padding: 14px;
}

.report-sheet {
  min-width: 0;
  display: grid;
  gap: 14px;
  min-height: calc(100vh - 248px);
}

.report-kind,
.cabinet-title,
.cabinet-item span,
.cabinet-item small {
  color: var(--cyber-text-muted);
  font-size: 11px;
  letter-spacing: 1.6px;
}

.report-kind {
  color: var(--cyber-success);
  font-weight: 800;
}

.sheet-heading {
  flex: 1;
  min-width: 0;
}

.sheet-heading h2 {
  margin: 6px 0 0;
  color: var(--cyber-text-primary);
  font-size: 24px;
  line-height: 1.2;
}

.sheet-content {
  display: grid;
  gap: 10px;
}

.sheet-content p {
  margin: 0;
  color: var(--cyber-text-secondary);
  line-height: 1.56;
  font-size: 13px;
}

.dense .sheet-content p {
  font-size: 12px;
  line-height: 1.46;
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
  text-align: left;
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
  .reports-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .page-head,
  .head-actions,
  .sheet-head {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
