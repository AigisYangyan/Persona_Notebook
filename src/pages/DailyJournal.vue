<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { NButton, NEmpty, NInput, useMessage } from "naive-ui";
import { useJournalStore } from "@/stores/journalStore";
import { getTodayStr } from "@/utils/date";

const journalStore = useJournalStore();
const message = useMessage();

const pageError = ref("");
const title = ref("");
const content = ref("");
const mood = ref("");

watch(
  () => journalStore.currentJournal,
  (journal) => {
    title.value = journal?.title ?? "";
    content.value = journal?.content ?? "";
    mood.value = journal?.mood ?? "";
  },
  { immediate: true }
);

onMounted(async () => {
  try {
    await journalStore.loadJournal(getTodayStr());
  } catch (error) {
    pageError.value = readError(error, "load journal failed");
  }
});

async function handleDateChange(entryDate: string) {
  try {
    await journalStore.loadJournal(entryDate);
  } catch (error) {
    pageError.value = readError(error, "switch journal date failed");
  }
}

async function saveJournal() {
  try {
    await journalStore.saveCurrentJournal(title.value.trim(), content.value, mood.value.trim());
    message.success("日记已保存");
  } catch (error) {
    message.error(readError(error, "save journal failed"));
  }
}

async function deleteJournal() {
  try {
    await journalStore.removeCurrentJournal();
    title.value = "";
    content.value = "";
    mood.value = "";
    message.success("日记已删除");
  } catch (error) {
    message.error(readError(error, "delete journal failed"));
  }
}

function readError(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}
</script>

<template>
  <div class="cyber-page journal-page">
    <h1 class="cyber-page-title">
      DAILY JOURNAL<span class="sub">每日日记</span>
    </h1>

    <div v-if="pageError" class="error-banner">{{ pageError }}</div>

    <div class="journal-layout">
      <section class="journal-main">
        <div class="cyber-section-title">
          TODAY NOTE<span class="sub">当天记录</span>
        </div>
        <div class="cyber-panel journal-panel">
          <div class="date-row">
            <div>
              <div class="tool-label">记录日期</div>
              <input
                :value="journalStore.currentDate"
                class="cyber-date"
                type="date"
                @input="handleDateChange(($event.target as HTMLInputElement).value)"
              />
            </div>
            <div class="journal-status">
              {{ journalStore.currentJournal ? "已存在记录" : "新建当天日记" }}
            </div>
          </div>

          <n-input v-model:value="title" placeholder="今天想给这篇日记起什么标题" />
          <n-input v-model:value="mood" placeholder="心情 / 标签" />
          <n-input
            v-model:value="content"
            type="textarea"
            placeholder="记录当天的所感所想。"
            :autosize="{ minRows: 16, maxRows: 24 }"
          />

          <div class="action-row">
            <n-button type="primary" :loading="journalStore.saving" @click="saveJournal">保存日记</n-button>
            <n-button :disabled="!journalStore.currentJournal" :loading="journalStore.saving" @click="deleteJournal">
              删除当天日记
            </n-button>
          </div>
        </div>
      </section>

      <section class="journal-side">
        <div class="cyber-section-title">
          RECENT<span class="sub">最近日记</span>
        </div>
        <div class="cyber-panel recent-panel">
          <div v-if="journalStore.recentJournals.length === 0" class="empty-block">
            <n-empty description="还没有日记" />
          </div>
          <button
            v-for="journal in journalStore.recentJournals"
            :key="journal.id"
            class="recent-card"
            @click="handleDateChange(journal.entry_date)"
          >
            <div class="recent-date">{{ journal.entry_date }}</div>
            <div class="recent-title">{{ journal.title || "未命名日记" }}</div>
            <div class="recent-mood">{{ journal.mood || "无标签" }}</div>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.journal-page {
  display: grid;
  gap: 18px;
}

.journal-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.45fr) minmax(280px, 0.75fr);
  gap: 18px;
}

.journal-main,
.journal-side {
  display: grid;
  gap: 12px;
  min-width: 0;
}

.journal-panel,
.recent-panel {
  padding: 18px;
  display: grid;
  gap: 14px;
}

.date-row,
.action-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.journal-status,
.tool-label,
.recent-mood {
  color: var(--cyber-text-muted);
  font-size: 13px;
}

.cyber-date {
  width: 180px;
  height: 38px;
  padding: 0 12px;
  border: 1px solid var(--cyber-border);
  background: var(--cyber-bg-input);
  color: var(--cyber-text-primary);
}

.recent-card {
  border: 1px solid var(--cyber-border);
  background: rgba(8, 18, 38, 0.82);
  color: var(--cyber-text-primary);
  text-align: left;
  padding: 12px;
  cursor: pointer;
  transition: 0.2s ease;
}

.recent-card:hover {
  border-color: var(--cyber-cyan);
  box-shadow: 0 0 14px rgba(0, 212, 255, 0.12);
}

.recent-date,
.recent-title {
  font-weight: 700;
}

.recent-title {
  margin-top: 6px;
}

.empty-block {
  min-height: 200px;
  display: grid;
  place-items: center;
}

.error-banner {
  padding: 12px 14px;
  border: 1px solid rgba(255, 51, 102, 0.35);
  background: rgba(255, 51, 102, 0.08);
  color: #ffd3de;
}

@media (max-width: 980px) {
  .journal-layout {
    grid-template-columns: 1fr;
  }

  .date-row,
  .action-row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
