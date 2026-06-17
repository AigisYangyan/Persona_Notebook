<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NButton, NEmpty, NModal, NPopconfirm, NTabPane, NTabs, useMessage } from "naive-ui";
import type { InsightPeriodType, InsightReport } from "@/api/client/tauriCommands";
import { coerceInsightList, coerceInsightText } from "@/features/insights/display";
import { normalizeInsightPeriod, reportKindLabel } from "@/features/insights/periods";
import { useInsightStore } from "@/stores/insightStore";
import { getTodayStr } from "@/utils/date";

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
    summary: activeReport.value?.summary || "生成后，这里会出现一份更像心理小助手的陪伴式报告。",
    emotionalReflection: readText(reportPayload.value.emotional_reflection),
    comfortMessage: readText(reportPayload.value.comfort_message),
    pressureSources: readText(reportPayload.value.pressure_sources),
    innerPattern: readText(reportPayload.value.inner_pattern),
    selfCompassion: readText(reportPayload.value.self_compassion),
    gentleQuestions: readList(reportPayload.value.gentle_questions),
    smallNextSteps: readList(reportPayload.value.small_next_steps),
    planProgress: readText(reportPayload.value.plan_progress),
    leveragePoints: readText(reportPayload.value.leverage_points),
    notEnoughData: readText(reportPayload.value.not_enough_data),
    completed: readText(reportPayload.value.completed),
    unfinished: readText(reportPayload.value.unfinished),
    legacyBond: readText(reportPayload.value.journal_and_bond_observations),
    legacyRoot: readText(reportPayload.value.root_causes),
    legacyRemedies: readList(reportPayload.value.concrete_remedies),
    legacyNext: readList(reportPayload.value.next_actions),
  })
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
    message.success("这条报告记录已删除");
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
  return coerceInsightList(value);
}

function readText(value: unknown): string {
  return coerceInsightText(value);
}

function buildReportText(input: {
  summary: string;
  emotionalReflection: string;
  comfortMessage: string;
  pressureSources: string;
  innerPattern: string;
  selfCompassion: string;
  gentleQuestions: string[];
  smallNextSteps: string[];
  planProgress: string;
  leveragePoints: string;
  notEnoughData: string;
  completed: string;
  unfinished: string;
  legacyBond: string;
  legacyRoot: string;
  legacyRemedies: string[];
  legacyNext: string[];
}): string[] {
  const paragraphs = [
    input.emotionalReflection || input.summary,
    input.comfortMessage,
    joinSentences([
      input.pressureSources ? `今晚真正压着你的，不只是任务本身，还有这些压力线索：${input.pressureSources}` : "",
      input.innerPattern ? `再往里看一点，你今天反复出现的内在模式可能是：${input.innerPattern}` : "",
      input.planProgress ? `计划层面的卡顿，也和这些推进状态有关：${input.planProgress}` : "",
    ]),
    joinSentences([
      input.selfCompassion,
      input.leveragePoints ? `不过你依然留下了一些可用的支点：${input.leveragePoints}` : "",
      input.legacyBond,
      input.legacyRoot,
    ]),
    input.gentleQuestions.length > 0 ? `如果今晚愿意慢一点，可以轻轻问自己：${input.gentleQuestions.join("；")}` : "",
    joinSentences([
      input.smallNextSteps.length > 0 ? `明天不用一下子做很多，先从这几步开始就够了：${input.smallNextSteps.join("；")}` : "",
      input.legacyRemedies.length > 0 ? `也可以试试这些补救动作：${input.legacyRemedies.join("；")}` : "",
      input.legacyNext.length > 0 ? `下一步提醒：${input.legacyNext.join("；")}` : "",
    ]),
    input.notEnoughData ? `这份判断里仍然有一些数据缺口：${input.notEnoughData}` : "",
    !input.emotionalReflection && input.completed
      ? `如果只从记录面看，今天你已经做过这些：${input.completed}${input.unfinished ? `；还有这些还没收尾：${input.unfinished}` : ""}`
      : "",
  ].filter(Boolean);

  return paragraphs.length > 0 ? paragraphs : ["生成后，这里会出现一份完整的单框报告。"];
}

function joinSentences(parts: string[]): string {
  return parts.filter(Boolean).join(" ");
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
}

.report-kind {
  font-size: 12px;
  letter-spacing: 2px;
}

.sheet-heading {
  display: grid;
  gap: 4px;
}

.sheet-heading h2 {
  margin: 0;
  font-size: 30px;
  line-height: 1.1;
}

.sheet-content {
  display: grid;
  gap: 14px;
  align-content: start;
}

.sheet-content p {
  margin: 0;
  line-height: 1.85;
  font-size: 15px;
  color: var(--cyber-text-primary);
}

.report-sheet.dense .sheet-content p {
  font-size: 14px;
  line-height: 1.7;
}

.cabinet {
  display: grid;
  gap: 10px;
}

.cabinet-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
}

.cabinet-item {
  display: grid;
  gap: 6px;
  padding: 10px;
  border: 1px solid rgba(0, 180, 255, 0.16);
  cursor: pointer;
  background: rgba(0, 18, 45, 0.42);
}

.cabinet-item.active {
  border-color: rgba(0, 212, 255, 0.5);
}

.cabinet-item strong {
  color: var(--cyber-text-primary);
  font-size: 13px;
}

.empty-box {
  min-height: 120px;
  display: grid;
  place-items: center;
}

.context-modal :deep(pre) {
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  max-height: 70vh;
  overflow: auto;
}

@media (max-width: 1100px) {
  .reports-layout {
    grid-template-columns: 1fr;
  }

  .cabinet {
    order: -1;
  }
}

@media (max-width: 720px) {
  .page-head {
    flex-direction: column;
    align-items: flex-start;
  }

  .head-actions {
    width: 100%;
  }
}
</style>
