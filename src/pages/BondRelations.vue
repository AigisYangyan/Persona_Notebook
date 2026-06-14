<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NButton, NEmpty, NInput, NInputNumber, useMessage } from "naive-ui";
import { useBondStore } from "@/stores/bondStore";

const bondStore = useBondStore();
const message = useMessage();

const pageError = ref("");
const newPersonName = ref("");
const newPersonLabel = ref("");
const newPersonScore = ref<number | null>(5);
const newPersonNote = ref("");

const personName = ref("");
const personLabel = ref("");
const personScore = ref<number | null>(5);
const personNote = ref("");

const entryTitle = ref("");
const entryContent = ref("");

const selectedPerson = computed(() => bondStore.selectedPerson);
const selectedEntry = computed(() => bondStore.selectedEntry);

watch(
  () => bondStore.selectedPerson,
  (person) => {
    personName.value = person?.name ?? "";
    personLabel.value = person?.relation_label ?? "";
    personScore.value = person?.score ?? 5;
    personNote.value = person?.note ?? "";
  },
  { immediate: true }
);

watch(
  () => bondStore.selectedEntry,
  (entry) => {
    entryTitle.value = entry?.title ?? "";
    entryContent.value = entry?.content ?? "";
  },
  { immediate: true }
);

onMounted(async () => {
  try {
    await bondStore.loadPeople();
  } catch (error) {
    pageError.value = readError(error, "load bond relations failed");
  }
});

async function createPerson() {
  if (!newPersonName.value.trim()) {
    message.warning("先输入人物名称");
    return;
  }

  try {
    await bondStore.savePerson({
      name: newPersonName.value.trim(),
      relationLabel: newPersonLabel.value.trim(),
      score: newPersonScore.value ?? 5,
      note: newPersonNote.value.trim(),
    });
    newPersonName.value = "";
    newPersonLabel.value = "";
    newPersonScore.value = 5;
    newPersonNote.value = "";
    message.success("人物已创建");
  } catch (error) {
    message.error(readError(error, "save person failed"));
  }
}

async function savePersonProfile() {
  if (!selectedPerson.value) {
    return;
  }

  try {
    await bondStore.savePerson({
      personId: selectedPerson.value.id,
      name: personName.value.trim(),
      relationLabel: personLabel.value.trim(),
      score: personScore.value ?? 5,
      note: personNote.value.trim(),
    });
    message.success("人物信息已更新");
  } catch (error) {
    message.error(readError(error, "update person failed"));
  }
}

async function deletePerson() {
  if (!selectedPerson.value) {
    return;
  }

  try {
    await bondStore.removePerson(selectedPerson.value.id);
    message.success("人物已删除");
  } catch (error) {
    message.error(readError(error, "delete person failed"));
  }
}

async function saveEntry() {
  if (!selectedPerson.value) {
    message.warning("先选择人物");
    return;
  }

  try {
    await bondStore.saveEntry(entryTitle.value.trim(), entryContent.value);
    message.success("关系文档已保存");
  } catch (error) {
    message.error(readError(error, "save entry failed"));
  }
}

async function deleteEntry() {
  if (!selectedEntry.value) {
    return;
  }

  try {
    await bondStore.removeEntry(selectedEntry.value.id);
    entryTitle.value = "";
    entryContent.value = "";
    message.success("关系文档已删除");
  } catch (error) {
    message.error(readError(error, "delete entry failed"));
  }
}

function choosePerson(personId: number) {
  pageError.value = "";
  void bondStore.loadEntries(personId).catch((error) => {
    pageError.value = readError(error, "load entries failed");
  });
}

function readError(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}
</script>

<template>
  <div class="cyber-page bond-page">
    <h1 class="cyber-page-title">
      BOND RELATIONS<span class="sub">羁绊关系</span>
    </h1>

    <div v-if="pageError" class="error-banner">{{ pageError }}</div>

    <div class="bond-layout">
      <section class="bond-column">
        <div class="cyber-section-title">
          PEOPLE<span class="sub">人物列表</span>
        </div>
        <div class="cyber-panel bond-panel">
          <div class="person-form">
            <n-input v-model:value="newPersonName" placeholder="人物名称" />
            <div class="person-form-row">
              <n-input v-model:value="newPersonLabel" placeholder="关系标签" />
              <n-input-number v-model:value="newPersonScore" :min="0" :max="10" />
            </div>
            <n-input
              v-model:value="newPersonNote"
              type="textarea"
              placeholder="给这个人物一个简短备注"
              :autosize="{ minRows: 2, maxRows: 3 }"
            />
            <n-button type="primary" :loading="bondStore.saving" @click="createPerson">创建人物</n-button>
          </div>

          <div v-if="bondStore.people.length === 0" class="empty-block">
            <n-empty description="还没有人物" />
          </div>
          <div v-else class="person-list">
            <button
              v-for="person in bondStore.people"
              :key="person.id"
              class="person-card"
              :class="{ active: bondStore.selectedPersonId === person.id }"
              @click="choosePerson(person.id)"
            >
              <div class="person-card-top">
                <span class="person-name">{{ person.name }}</span>
                <span class="person-score">{{ person.score }}/10</span>
              </div>
              <div class="person-meta">
                <span>{{ person.relation_label || "未分类" }}</span>
                <span>{{ person.entry_count }} 篇</span>
              </div>
              <div class="person-date">{{ person.latest_entry_date || "暂无记录" }}</div>
            </button>
          </div>
        </div>
      </section>

      <section class="bond-column">
        <div class="cyber-section-title">
          PROFILE<span class="sub">人物详情</span>
        </div>
        <div class="cyber-panel bond-panel">
          <template v-if="selectedPerson">
            <div class="detail-grid">
              <n-input v-model:value="personName" placeholder="人物名称" />
              <n-input v-model:value="personLabel" placeholder="关系标签" />
              <n-input-number v-model:value="personScore" :min="0" :max="10" />
            </div>
            <n-input
              v-model:value="personNote"
              type="textarea"
              placeholder="这里可以写一些对关系的长期描述"
              :autosize="{ minRows: 3, maxRows: 4 }"
            />
            <div class="detail-actions">
              <n-button type="primary" :loading="bondStore.saving" @click="savePersonProfile">保存人物</n-button>
              <n-button type="error" :loading="bondStore.saving" @click="deletePerson">删除人物</n-button>
            </div>

            <div class="entry-toolbar">
              <div>
                <div class="tool-label">文档日期</div>
                <input
                  :value="bondStore.selectedEntryDate"
                  class="cyber-date"
                  type="date"
                  @input="bondStore.setSelectedEntryDate(($event.target as HTMLInputElement).value)"
                />
              </div>
              <div class="entry-date-list">
                <button
                  v-for="entry in bondStore.entries"
                  :key="entry.id"
                  class="entry-chip"
                  :class="{ active: bondStore.selectedEntryDate === entry.entry_date }"
                  @click="bondStore.setSelectedEntryDate(entry.entry_date)"
                >
                  {{ entry.entry_date }}
                </button>
              </div>
            </div>

            <n-input v-model:value="entryTitle" placeholder="文档标题" />
            <n-input
              v-model:value="entryContent"
              type="textarea"
              placeholder="把你对这个人的想法写在这里，留在本地，不带到现实冲突里。"
              :autosize="{ minRows: 10, maxRows: 16 }"
            />
            <div class="detail-actions">
              <n-button type="primary" :loading="bondStore.saving" @click="saveEntry">保存文档</n-button>
              <n-button :disabled="!selectedEntry" :loading="bondStore.saving" @click="deleteEntry">
                删除当前文档
              </n-button>
            </div>
          </template>
          <div v-else class="empty-block">
            <n-empty description="选择一个人物后开始记录" />
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.bond-page {
  display: grid;
  gap: 18px;
}

.bond-layout {
  display: grid;
  grid-template-columns: minmax(320px, 0.8fr) minmax(0, 1.4fr);
  gap: 18px;
}

.bond-column {
  display: grid;
  gap: 12px;
  min-width: 0;
}

.bond-panel,
.person-form,
.entry-toolbar {
  display: grid;
  gap: 16px;
}

.bond-panel {
  padding: 18px;
}

.person-form-row,
.detail-grid,
.detail-actions {
  display: grid;
  gap: 12px;
}

.person-form-row {
  grid-template-columns: minmax(0, 1fr) 140px;
}

.detail-grid {
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 140px;
}

.person-list,
.entry-date-list {
  display: grid;
  gap: 10px;
}

.person-card,
.entry-chip {
  border: 1px solid var(--cyber-border);
  background: rgba(8, 18, 38, 0.82);
  color: var(--cyber-text-primary);
  text-align: left;
  padding: 12px;
  cursor: pointer;
  transition: 0.2s ease;
}

.person-card:hover,
.entry-chip:hover,
.person-card.active,
.entry-chip.active {
  border-color: var(--cyber-cyan);
  box-shadow: 0 0 14px rgba(0, 212, 255, 0.12);
}

.person-card-top,
.person-meta {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.person-name,
.person-score {
  font-weight: 700;
}

.person-meta,
.person-date,
.tool-label {
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

@media (max-width: 1100px) {
  .bond-layout {
    grid-template-columns: 1fr;
  }

  .person-form-row,
  .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
