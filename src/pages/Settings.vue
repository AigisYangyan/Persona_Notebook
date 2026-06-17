<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { NInput, useMessage } from "naive-ui";
import type { ApiRunDiagnostic, PersonalProfile } from "@/api/client/tauriCommands";
import { exportDataJson, getRecentApiRuns, importCsv, importJson } from "@/api/client/tauriCommands";
import { usePersonalMemoryStore } from "@/stores/personalMemoryStore";
import { useSettingStore } from "@/stores/settingStore";

const message = useMessage();
const settingStore = useSettingStore();
const personalMemoryStore = usePersonalMemoryStore();

const deepseekBaseUrl = ref("");
const deepseekFlashModel = ref("");
const deepseekProModel = ref("");
const apiKey = ref("");
const savingApiConfig = ref(false);
const savingApiKey = ref(false);
const savingProfile = ref(false);
const memorySearchQuery = ref("");
const contextDate = ref(new Date().toISOString().slice(0, 10));
const contextMode = ref("general");
const patchJson = ref(
  JSON.stringify(
    {
      schema_version: "1.0",
      profile_updates: null,
      memory_operations: [],
    },
    null,
    2
  )
);
const sourceContextId = ref("settings-manual");
const diagnosticOutput = ref("");
const recentApiRuns = ref<ApiRunDiagnostic[]>([]);
const profileForm = ref<PersonalProfile>({
  birthday: "",
  personality: "",
  experiences: "",
  personal_notes: "",
  updated_at: null,
});

const topMemoryItems = computed(() => personalMemoryStore.overview?.top_items ?? []);
const memoryStats = computed(() => {
  const overview = personalMemoryStore.overview;
  return [
    { label: "Total", sub: "all memory items", value: overview?.total_items ?? 0 },
    { label: "Active", sub: "usable now", value: overview?.active_items ?? 0 },
    { label: "Pending", sub: "needs review", value: overview?.pending_items ?? 0 },
    { label: "Rejected", sub: "not applied", value: overview?.rejected_items ?? 0 },
  ];
});

onMounted(async () => {
  await Promise.all([settingStore.loadSettings(), personalMemoryStore.loadFoundation()]);
  deepseekBaseUrl.value = settingStore.settings.deepseekBaseUrl;
  deepseekFlashModel.value = settingStore.settings.deepseekFlashModel;
  deepseekProModel.value = settingStore.settings.deepseekProModel;
  syncProfileForm();
  void handleLoadRecentApiRuns();
});

function syncProfileForm() {
  profileForm.value = {
    ...personalMemoryStore.profile,
  };
}

function updateProfileForm<K extends keyof PersonalProfile>(key: K, value: PersonalProfile[K]) {
  profileForm.value = {
    ...profileForm.value,
    [key]: value,
  };
}

function downloadJson(filename: string, content: string) {
  const blob = new Blob([content], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function formatCacheHitRate(run: ApiRunDiagnostic): string {
  const hit = run.prompt_cache_hit_tokens ?? 0;
  const miss = run.prompt_cache_miss_tokens ?? 0;
  const total = hit + miss;
  if (total <= 0) {
    return "n/a";
  }
  return `${Math.round((hit / total) * 100)}%`;
}

function formatTokenStats(run: ApiRunDiagnostic): string {
  const prompt = run.prompt_tokens ?? 0;
  const completion = run.completion_tokens ?? 0;
  return `${prompt} / ${completion}`;
}

async function handleSaveApiConfig() {
  savingApiConfig.value = true;
  settingStore.setEngine("rules_api");
  settingStore.updateApiConfig(deepseekBaseUrl.value, deepseekFlashModel.value, deepseekProModel.value);
  try {
    await settingStore.persistGeneralSettings();
    message.success("DeepSeek config saved");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Save failed: ${messageText}`);
  } finally {
    savingApiConfig.value = false;
  }
}

async function handleSaveApiKey() {
  if (!apiKey.value.trim()) {
    message.warning("Please enter a new API key");
    return;
  }

  savingApiKey.value = true;
  try {
    await settingStore.persistApiKey(apiKey.value.trim());
    apiKey.value = "";
    message.success("API key updated");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`API key save failed: ${messageText}`);
  } finally {
    savingApiKey.value = false;
  }
}

async function handleClearApiKey() {
  savingApiKey.value = true;
  try {
    await settingStore.removeApiKey();
    apiKey.value = "";
    message.success("API key cleared");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Clear API key failed: ${messageText}`);
  } finally {
    savingApiKey.value = false;
  }
}

async function handleSaveProfile() {
  savingProfile.value = true;
  try {
    await personalMemoryStore.saveProfile(profileForm.value);
    syncProfileForm();
    message.success("Profile and RAG mirror synced");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Save profile failed: ${messageText}`);
  } finally {
    savingProfile.value = false;
  }
}

async function handleSearchMemory() {
  try {
    await personalMemoryStore.searchMemory(memorySearchQuery.value, [], 20);
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Search failed: ${messageText}`);
  }
}

async function handleBuildContextPack() {
  try {
    const contextPackResult = await personalMemoryStore.generateContextPack(contextDate.value, contextMode.value);
    diagnosticOutput.value = JSON.stringify(contextPackResult, null, 2);
    message.success("Context pack generated");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Build context pack failed: ${messageText}`);
  }
}

async function handleRebuildMirror() {
  try {
    diagnosticOutput.value = await personalMemoryStore.rebuildMirrorFiles();
    message.success("rag_memory mirror rebuilt");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Rebuild mirror failed: ${messageText}`);
  }
}

async function handleExportRagSnapshot() {
  try {
    const snapshot = await personalMemoryStore.exportSnapshot();
    diagnosticOutput.value = snapshot;
    downloadJson(`pgrn-rag-memory-${new Date().toISOString().slice(0, 10)}.json`, snapshot);
    message.success("RAG snapshot exported");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Export RAG snapshot failed: ${messageText}`);
  }
}

async function handleApplyPatch() {
  try {
    await personalMemoryStore.applyPatch(patchJson.value, sourceContextId.value.trim() || "settings-manual");
    syncProfileForm();
    diagnosticOutput.value = JSON.stringify(personalMemoryStore.lastPatchResult, null, 2);
    message.success("Memory patch applied");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Apply patch failed: ${messageText}`);
  }
}

async function handleLoadRecentApiRuns() {
  try {
    const runs = await getRecentApiRuns(12);
    recentApiRuns.value = runs;
    diagnosticOutput.value = JSON.stringify(runs, null, 2);
    message.success("Recent API diagnostics loaded");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Load API diagnostics failed: ${messageText}`);
  }
}

async function handleExport() {
  try {
    const json = await exportDataJson();
    downloadJson(`pgrn-export-${new Date().toISOString().slice(0, 10)}.json`, json);
    message.success("Full data export complete");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Export failed: ${messageText}`);
  }
}

async function handleImportFile(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) {
    return;
  }

  const text = await file.text();
  try {
    const result = file.name.endsWith(".csv")
      ? await importCsv(text)
      : file.name.endsWith(".json")
        ? await importJson(text)
        : null;

    if (!result) {
      message.error("Only .csv and .json are supported");
      return;
    }

    if (result.errors.length > 0) {
      message.warning(`Import finished: ${result.imported} ok / ${result.errors.length} failed`);
    } else {
      message.success(`Imported ${result.imported} records`);
    }

    await personalMemoryStore.loadFoundation();
    syncProfileForm();
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`Import failed: ${messageText}`);
  } finally {
    target.value = "";
  }
}
</script>

<template>
  <div class="cyber-page">
    <h1 class="cyber-page-title">SETTINGS<span class="sub">system config</span></h1>

    <div class="settings-grid">
      <section class="settings-section">
        <div class="cyber-section-title">DEEPSEEK CONFIGURATION<span class="sub">dual-tier routing</span></div>
        <div class="config-panel cyber-panel">
          <div class="config-status">
            <span class="status-label">API KEY STATUS</span>
            <span class="status-badge" :class="settingStore.settings.apiKeyConfigured ? 'configured' : 'unconfigured'">
              {{ settingStore.settings.apiKeyConfigured ? "configured" : "not configured" }}
            </span>
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label class="form-label">DeepSeek Base URL</label>
              <n-input v-model:value="deepseekBaseUrl" placeholder="https://api.deepseek.com/v1" />
            </div>
            <div class="form-group">
              <label class="form-label">Flash Model</label>
              <n-input v-model:value="deepseekFlashModel" placeholder="deepseek-chat" />
            </div>
            <div class="form-group">
              <label class="form-label">Pro Model</label>
              <n-input v-model:value="deepseekProModel" placeholder="deepseek-reasoner" />
            </div>
          </div>

          <div class="hint-panel">
            <div class="hint-line"><strong>flash</strong> = structured, fast, cheaper. Used for scoring and plan refresh.</div>
            <div class="hint-line"><strong>pro</strong> = longer writing and steadier insight. Used for tarot and reports.</div>
            <div class="hint-line">Current route: scoring / plan / clarification use flash, tarot and reports use pro.</div>
          </div>

          <div class="form-actions">
            <button class="cyber-btn primary" :disabled="savingApiConfig" @click="handleSaveApiConfig">
              Save DeepSeek config
            </button>
          </div>

          <div class="config-divider"></div>

          <div class="form-grid">
            <div class="form-group">
              <label class="form-label">API Key</label>
              <n-input
                v-model:value="apiKey"
                type="password"
                placeholder="enter a new API key"
                show-password-on="click"
              />
            </div>
          </div>

          <div class="form-actions">
            <button class="cyber-btn primary" :disabled="savingApiKey" @click="handleSaveApiKey">Update API key</button>
            <button class="cyber-btn danger" :disabled="savingApiKey" @click="handleClearApiKey">Clear API key</button>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">PERSONAL PROFILE<span class="sub">long-term context</span></div>
        <div class="profile-panel cyber-panel">
          <div class="profile-grid">
            <div class="form-group">
              <label class="form-label">Birthday</label>
              <input
                class="native-field"
                type="date"
                :value="profileForm.birthday"
                @input="updateProfileForm('birthday', ($event.target as HTMLInputElement).value)"
              />
            </div>

            <div class="form-group form-span-2">
              <label class="form-label">Personality</label>
              <n-input
                :value="profileForm.personality"
                type="textarea"
                :autosize="{ minRows: 3, maxRows: 5 }"
                placeholder="temperament, pressure response, stable preferences"
                @update:value="(value) => updateProfileForm('personality', value)"
              />
            </div>

            <div class="form-group form-span-2">
              <label class="form-label">Experiences</label>
              <n-input
                :value="profileForm.experiences"
                type="textarea"
                :autosize="{ minRows: 4, maxRows: 8 }"
                placeholder="important experiences, study background, recurring life themes"
                @update:value="(value) => updateProfileForm('experiences', value)"
              />
            </div>

            <div class="form-group form-span-2">
              <label class="form-label">Personal Notes</label>
              <n-input
                :value="profileForm.personal_notes"
                type="textarea"
                :autosize="{ minRows: 4, maxRows: 10 }"
                placeholder="taboos, preferred support style, recurring struggles, direction"
                @update:value="(value) => updateProfileForm('personal_notes', value)"
              />
            </div>
          </div>

          <div class="form-actions">
            <button class="cyber-btn primary" :disabled="savingProfile" @click="handleSaveProfile">Save profile</button>
            <span class="updated-text">Last sync: {{ personalMemoryStore.profile.updated_at || "not yet synced" }}</span>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">RAG MEMORY FOUNDATION<span class="sub">memory base</span></div>
        <div class="memory-panel cyber-panel">
          <div class="stats-grid">
            <div v-for="item in memoryStats" :key="item.label" class="stat-tile">
              <div class="stat-label">{{ item.label }}</div>
              <div class="stat-sub">{{ item.sub }}</div>
              <div class="stat-value">{{ item.value }}</div>
            </div>
          </div>

          <div class="memory-actions">
            <div class="form-group compact-grow">
              <label class="form-label">Search Memory</label>
              <n-input v-model:value="memorySearchQuery" placeholder="habit / preference / relationship / goal" />
            </div>
            <button class="cyber-btn primary" :disabled="personalMemoryStore.searching" @click="handleSearchMemory">
              Search
            </button>
          </div>

          <div class="memory-list">
            <div class="list-title">Top Memory Items</div>
            <div v-if="topMemoryItems.length === 0" class="empty-state">No long-term memory items yet.</div>
            <div v-else class="memory-cards">
              <article v-for="item in topMemoryItems" :key="item.id" class="memory-card">
                <div class="memory-card-head">
                  <div>
                    <div class="memory-title">{{ item.title }}</div>
                    <div class="memory-meta">
                      {{ item.memory_type }} / {{ item.status }} / evidence {{ item.source_count }}
                    </div>
                  </div>
                  <div class="memory-score">{{ item.importance }}</div>
                </div>
                <div class="memory-summary">{{ item.summary || item.detail || "no summary" }}</div>
                <div class="memory-tags">
                  <span v-for="tag in item.tags" :key="tag" class="tag-chip">{{ tag }}</span>
                </div>
              </article>
            </div>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">API DIAGNOSTICS<span class="sub">recent model runs</span></div>
        <div class="debug-panel cyber-panel">
          <div class="form-actions wrap diagnostic-actions">
            <button class="cyber-btn primary" @click="handleLoadRecentApiRuns">Refresh diagnostics</button>
          </div>

          <div class="diagnostic-list">
            <article v-for="run in recentApiRuns" :key="run.id" class="diagnostic-card">
              <div class="diagnostic-head">
                <div>
                  <div class="diagnostic-title">{{ run.task_kind }} / {{ run.model_tier }}</div>
                  <div class="diagnostic-meta">{{ run.engine_name }} / {{ run.created_at }}</div>
                </div>
                <div class="diagnostic-status" :class="run.status">{{ run.status }}</div>
              </div>
              <div class="diagnostic-grid">
                <div>cache hit: {{ formatCacheHitRate(run) }}</div>
                <div>tokens p/c: {{ formatTokenStats(run) }}</div>
                <div>finish: {{ run.finish_reason || "n/a" }}</div>
                <div>fallback: {{ run.fallback_used ? "yes" : "no" }}</div>
              </div>
              <div v-if="run.error_message" class="diagnostic-error">{{ run.error_message }}</div>
            </article>
            <div v-if="recentApiRuns.length === 0" class="empty-state">No recent API diagnostics yet.</div>
          </div>

          <div class="debug-grid">
            <div class="form-group">
              <label class="form-label">Context Date</label>
              <input class="native-field" type="date" v-model="contextDate" />
            </div>
            <div class="form-group">
              <label class="form-label">Context Mode</label>
              <select class="native-field" v-model="contextMode">
                <option value="general">general</option>
                <option value="morning">morning</option>
                <option value="evening">evening</option>
                <option value="planning">planning</option>
              </select>
            </div>
            <div class="form-group form-span-2">
              <label class="form-label">Patch Source Context ID</label>
              <n-input v-model:value="sourceContextId" placeholder="settings-manual / ai-run-2026-06-13" />
            </div>
          </div>

          <div class="form-actions wrap">
            <button class="cyber-btn primary" @click="handleBuildContextPack">Build Context Pack</button>
            <button class="cyber-btn primary" @click="handleRebuildMirror">Rebuild rag_memory mirror</button>
            <button class="cyber-btn primary" @click="handleExportRagSnapshot">Export RAG snapshot</button>
          </div>

          <div class="form-group">
            <label class="form-label">PersonalMemoryPatch v1</label>
            <n-input
              v-model:value="patchJson"
              type="textarea"
              :autosize="{ minRows: 10, maxRows: 18 }"
              placeholder="paste a structured AI memory patch here"
            />
          </div>

          <div class="form-actions wrap">
            <button class="cyber-btn primary" :disabled="personalMemoryStore.patching" @click="handleApplyPatch">
              Apply Patch
            </button>
            <div v-if="personalMemoryStore.lastPatchResult" class="patch-status">
              {{ personalMemoryStore.lastPatchResult.apply_status }}
              / applied {{ personalMemoryStore.lastPatchResult.applied_operations }}
              / rejected {{ personalMemoryStore.lastPatchResult.rejected_operations }}
            </div>
          </div>

          <div class="form-group">
            <label class="form-label">Diagnostic Output</label>
            <n-input
              :value="diagnosticOutput"
              type="textarea"
              readonly
              :autosize="{ minRows: 10, maxRows: 18 }"
              placeholder="context pack, API runs, patch result"
            />
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">DATA MANAGEMENT<span class="sub">import and export</span></div>
        <div class="data-panel cyber-panel">
          <div class="data-item">
            <div class="data-info">
              <div class="data-title">EXPORT DATA AS JSON</div>
              <div class="data-desc">export the full data object</div>
              <div class="data-hint">includes records, ledger, plans, bonds, journals, personal memory and public settings</div>
            </div>
            <button class="cyber-btn primary" @click="handleExport">Export JSON</button>
          </div>

          <div class="data-divider"></div>

          <div class="data-item">
            <div class="data-info">
              <div class="data-title">IMPORT CSV OR JSON</div>
              <div class="data-desc">supports legacy record arrays and object exports</div>
              <div class="data-hint">JSON import restores records first and may sync personal_profile</div>
            </div>
            <label class="cyber-btn primary file-btn">
              Choose File
              <input type="file" accept=".csv,.json" hidden @change="handleImportFile" />
            </label>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.config-panel,
.profile-panel,
.memory-panel,
.debug-panel,
.data-panel {
  padding: 20px 24px;
}

.config-status {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 18px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--cyber-border);
}

.status-label,
.form-label,
.list-title,
.data-title,
.stat-label {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--cyber-text-secondary);
}

.status-badge {
  padding: 3px 12px;
  border-radius: 3px;
  font-size: 12px;
  font-weight: 600;
}

.status-badge.configured {
  color: var(--cyber-success);
  background: rgba(0, 255, 170, 0.1);
  border: 1px solid rgba(0, 255, 170, 0.3);
}

.status-badge.unconfigured {
  color: var(--cyber-warning);
  background: rgba(255, 204, 0, 0.1);
  border: 1px solid rgba(255, 204, 0, 0.3);
}

.form-grid,
.profile-grid,
.debug-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-span-2 {
  grid-column: span 2;
}

.form-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 16px;
}

.form-actions.wrap {
  flex-wrap: wrap;
}

.compact-grow {
  flex: 1;
}

.config-divider,
.data-divider {
  height: 1px;
  background: linear-gradient(90deg, var(--cyber-border), transparent);
  margin: 20px 0;
}

.hint-panel {
  display: grid;
  gap: 8px;
  margin-top: 16px;
  padding: 12px 14px;
  border: 1px solid rgba(0, 212, 255, 0.18);
  background: rgba(6, 15, 38, 0.72);
}

.hint-line {
  color: var(--cyber-text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.hint-line strong {
  color: var(--cyber-cyan);
}

.native-field {
  min-height: 40px;
  padding: 0 12px;
  background: rgba(6, 15, 38, 0.95);
  border: 1px solid var(--cyber-border);
  border-radius: 4px;
  color: var(--cyber-text-primary);
  font-family: inherit;
}

.native-field:focus {
  outline: none;
  border-color: var(--cyber-cyan);
  box-shadow: 0 0 0 1px rgba(0, 212, 255, 0.18);
}

.cyber-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 40px;
  padding: 10px 16px;
  background: rgba(0, 60, 120, 0.2);
  border: 1px solid var(--cyber-border);
  border-radius: 4px;
  color: var(--cyber-text-secondary);
  cursor: pointer;
  transition: all 0.25s;
  font-family: inherit;
}

.cyber-btn:hover {
  border-color: var(--cyber-border-hover);
  background: rgba(0, 180, 255, 0.1);
  color: var(--cyber-text-primary);
}

.cyber-btn.primary {
  background: linear-gradient(135deg, rgba(0, 100, 200, 0.25), rgba(0, 180, 255, 0.15));
  border-color: var(--cyber-cyan);
  box-shadow: 0 0 10px rgba(0, 180, 255, 0.1);
}

.cyber-btn.primary:hover {
  box-shadow: 0 0 16px rgba(0, 212, 255, 0.25);
}

.cyber-btn.danger {
  background: rgba(255, 51, 102, 0.08);
  border-color: rgba(255, 51, 102, 0.3);
  color: var(--cyber-danger);
}

.cyber-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.updated-text,
.patch-status,
.data-desc,
.stat-sub,
.memory-meta,
.data-hint,
.diagnostic-meta,
.diagnostic-grid,
.diagnostic-error {
  font-size: 13px;
  color: var(--cyber-text-dim);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.stat-tile,
.diagnostic-card,
.memory-card {
  padding: 14px;
  background: rgba(8, 18, 42, 0.88);
  border: 1px solid rgba(0, 212, 255, 0.16);
  border-radius: 6px;
}

.stat-value,
.memory-score {
  font-size: 24px;
  font-weight: 700;
  color: var(--cyber-cyan);
}

.memory-actions {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  margin-top: 18px;
}

.memory-list,
.diagnostic-list {
  margin-top: 18px;
}

.memory-cards {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 10px;
}

.memory-card.compact {
  padding: 12px;
}

.memory-card-head,
.diagnostic-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.memory-title,
.data-title,
.diagnostic-title {
  color: var(--cyber-text-primary);
}

.memory-summary {
  margin-top: 10px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--cyber-text-secondary);
}

.memory-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.tag-chip {
  padding: 3px 10px;
  border-radius: 999px;
  background: rgba(0, 212, 255, 0.08);
  border: 1px solid rgba(0, 212, 255, 0.16);
  font-size: 12px;
  color: var(--cyber-text-secondary);
}

.diagnostic-list {
  display: grid;
  gap: 12px;
}

.diagnostic-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 16px;
  margin-top: 12px;
}

.diagnostic-status {
  padding: 2px 10px;
  border-radius: 999px;
  border: 1px solid rgba(0, 212, 255, 0.2);
  text-transform: uppercase;
  font-size: 12px;
  color: var(--cyber-text-secondary);
}

.diagnostic-status.success {
  color: var(--cyber-success);
  border-color: rgba(0, 255, 170, 0.3);
}

.diagnostic-status.error {
  color: var(--cyber-danger);
  border-color: rgba(255, 51, 102, 0.3);
}

.diagnostic-error {
  margin-top: 10px;
  color: #ff8aa5;
  line-height: 1.5;
}

.empty-state {
  margin-top: 10px;
  font-size: 13px;
  color: var(--cyber-text-dim);
}

.data-item {
  display: flex;
  align-items: center;
  gap: 16px;
  justify-content: space-between;
}

.data-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.file-btn {
  position: relative;
  overflow: hidden;
}

@media (max-width: 980px) {
  .form-grid,
  .profile-grid,
  .debug-grid,
  .stats-grid,
  .memory-cards,
  .diagnostic-grid {
    grid-template-columns: 1fr;
  }

  .form-span-2 {
    grid-column: span 1;
  }

  .memory-actions,
  .data-item,
  .form-actions {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
