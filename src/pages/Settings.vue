<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { NInput, useMessage } from "naive-ui";
import type { PersonalProfile } from "@/api/client/tauriCommands";
import { exportDataJson, importCsv, importJson } from "@/api/client/tauriCommands";
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
    { label: "Total", sub: "总记忆", value: overview?.total_items ?? 0 },
    { label: "Active", sub: "活跃", value: overview?.active_items ?? 0 },
    { label: "Pending", sub: "待确认", value: overview?.pending_items ?? 0 },
    { label: "Rejected", sub: "已拒绝", value: overview?.rejected_items ?? 0 },
  ];
});

onMounted(async () => {
  await Promise.all([settingStore.loadSettings(), personalMemoryStore.loadFoundation()]);
  deepseekBaseUrl.value = settingStore.settings.deepseekBaseUrl;
  deepseekFlashModel.value = settingStore.settings.deepseekFlashModel;
  deepseekProModel.value = settingStore.settings.deepseekProModel;
  syncProfileForm();
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

async function handleSaveApiConfig() {
  savingApiConfig.value = true;
  settingStore.setEngine("rules_api");
  settingStore.updateApiConfig(
    deepseekBaseUrl.value,
    deepseekFlashModel.value,
    deepseekProModel.value
  );
  try {
    await settingStore.persistGeneralSettings();
    message.success("DeepSeek 配置已保存");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`保存失败: ${messageText}`);
  } finally {
    savingApiConfig.value = false;
  }
}

async function handleSaveApiKey() {
  if (!apiKey.value.trim()) {
    message.warning("请输入新的 API Key");
    return;
  }

  savingApiKey.value = true;
  try {
    await settingStore.persistApiKey(apiKey.value.trim());
    apiKey.value = "";
    message.success("API Key 已更新");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`保存 API Key 失败: ${messageText}`);
  } finally {
    savingApiKey.value = false;
  }
}

async function handleClearApiKey() {
  savingApiKey.value = true;
  try {
    await settingStore.removeApiKey();
    apiKey.value = "";
    message.success("API Key 已清除");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`清除 API Key 失败: ${messageText}`);
  } finally {
    savingApiKey.value = false;
  }
}

async function handleSaveProfile() {
  savingProfile.value = true;
  try {
    await personalMemoryStore.saveProfile(profileForm.value);
    syncProfileForm();
    message.success("个人资料与 RAG 镜像已同步");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`保存个人资料失败: ${messageText}`);
  } finally {
    savingProfile.value = false;
  }
}

async function handleSearchMemory() {
  try {
    await personalMemoryStore.searchMemory(memorySearchQuery.value, [], 20);
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`搜索记忆失败: ${messageText}`);
  }
}

async function handleBuildContextPack() {
  try {
    const contextPackResult = await personalMemoryStore.generateContextPack(contextDate.value, contextMode.value);
    diagnosticOutput.value = JSON.stringify(contextPackResult, null, 2);
    message.success("个人上下文包已生成");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`生成上下文包失败: ${messageText}`);
  }
}

async function handleRebuildMirror() {
  try {
    diagnosticOutput.value = await personalMemoryStore.rebuildMirrorFiles();
    message.success("rag_memory 文件镜像已重建");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`重建镜像失败: ${messageText}`);
  }
}

async function handleExportRagSnapshot() {
  try {
    const snapshot = await personalMemoryStore.exportSnapshot();
    diagnosticOutput.value = snapshot;
    downloadJson(`pgrn-rag-memory-${new Date().toISOString().slice(0, 10)}.json`, snapshot);
    message.success("RAG 快照已导出");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`导出 RAG 快照失败: ${messageText}`);
  }
}

async function handleApplyPatch() {
  try {
    await personalMemoryStore.applyPatch(patchJson.value, sourceContextId.value.trim() || "settings-manual");
    syncProfileForm();
    diagnosticOutput.value = JSON.stringify(personalMemoryStore.lastPatchResult, null, 2);
    message.success("Memory patch 已应用");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`应用 patch 失败: ${messageText}`);
  }
}

async function handleExport() {
  try {
    const json = await exportDataJson();
    downloadJson(`pgrn-export-${new Date().toISOString().slice(0, 10)}.json`, json);
    message.success("完整数据导出成功");
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`导出失败: ${messageText}`);
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
      message.error("仅支持 .csv 或 .json 文件");
      return;
    }

    if (result.errors.length > 0) {
      message.warning(`导入完成: 成功 ${result.imported} 条, 失败 ${result.errors.length} 条`);
    } else {
      message.success(`成功导入 ${result.imported} 条记录`);
    }

    await personalMemoryStore.loadFoundation();
    syncProfileForm();
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    message.error(`导入失败: ${messageText}`);
  } finally {
    target.value = "";
  }
}
</script>

<template>
  <div class="cyber-page">
    <h1 class="cyber-page-title">
      SETTINGS<span class="sub">设置</span>
    </h1>

    <div class="settings-grid">
      <section class="settings-section">
        <div class="cyber-section-title">
          DEEPSEEK CONFIGURATION<span class="sub">DeepSeek 双档路由</span>
        </div>
        <div class="config-panel cyber-panel">
          <div class="config-status">
            <span class="status-label">API KEY STATUS</span>
            <span
              class="status-badge"
              :class="settingStore.settings.apiKeyConfigured ? 'configured' : 'unconfigured'"
            >
              {{ settingStore.settings.apiKeyConfigured ? "已配置" : "未配置" }}
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
            <div class="hint-line"><strong>flash</strong> = 结构化、快、便宜，用于评分与计划刷新</div>
            <div class="hint-line"><strong>pro</strong> = 长文、洞察更稳，用于塔罗、日报、周报、月报</div>
            <div class="hint-line">当前固定路由：评分/计划/澄清走 flash，塔罗与各类报告走 pro</div>
          </div>

          <div class="form-actions">
            <button class="cyber-btn primary" :disabled="savingApiConfig" @click="handleSaveApiConfig">
              保存 DeepSeek 配置
            </button>
          </div>

          <div class="config-divider"></div>

          <div class="form-grid">
            <div class="form-group">
              <label class="form-label">API Key</label>
              <n-input
                v-model:value="apiKey"
                type="password"
                placeholder="输入新的 API Key"
                show-password-on="click"
              />
            </div>
          </div>

          <div class="form-actions">
            <button class="cyber-btn primary" :disabled="savingApiKey" @click="handleSaveApiKey">
              更新 API Key
            </button>
            <button class="cyber-btn danger" :disabled="savingApiKey" @click="handleClearApiKey">
              清除 API Key
            </button>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">
          PERSONAL PROFILE<span class="sub">个人资料与长期上下文</span>
        </div>
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
                placeholder="例如：理性、慢热、容易在高压时追求秩序感"
                @update:value="(value) => updateProfileForm('personality', value)"
              />
            </div>

            <div class="form-group form-span-2">
              <label class="form-label">Experiences</label>
              <n-input
                :value="profileForm.experiences"
                type="textarea"
                :autosize="{ minRows: 4, maxRows: 8 }"
                placeholder="填写重要经历、学习背景、反复出现的生活主题"
                @update:value="(value) => updateProfileForm('experiences', value)"
              />
            </div>

            <div class="form-group form-span-2">
              <label class="form-label">Personal Notes</label>
              <n-input
                :value="profileForm.personal_notes"
                type="textarea"
                :autosize="{ minRows: 4, maxRows: 10 }"
                placeholder="补充偏好、禁忌、常见困境、目标倾向等"
                @update:value="(value) => updateProfileForm('personal_notes', value)"
              />
            </div>
          </div>

          <div class="form-actions">
            <button class="cyber-btn primary" :disabled="savingProfile" @click="handleSaveProfile">
              保存个人资料
            </button>
            <span class="updated-text">
              最近同步：
              {{ personalMemoryStore.profile.updated_at || "尚未同步" }}
            </span>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">
          RAG MEMORY FOUNDATION<span class="sub">浓缩记忆底座</span>
        </div>
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
              <n-input
                v-model:value="memorySearchQuery"
                placeholder="搜索 habit / preference / relationship 等记忆"
              />
            </div>
            <button class="cyber-btn primary" :disabled="personalMemoryStore.searching" @click="handleSearchMemory">
              搜索记忆
            </button>
          </div>

          <div class="memory-list">
            <div class="list-title">Top Memory Items</div>
            <div v-if="topMemoryItems.length === 0" class="empty-state">
              还没有进入长期记忆库的条目。
            </div>
            <div v-else class="memory-cards">
              <article v-for="item in topMemoryItems" :key="item.id" class="memory-card">
                <div class="memory-card-head">
                  <div>
                    <div class="memory-title">{{ item.title }}</div>
                    <div class="memory-meta">
                      {{ item.memory_type }} · {{ item.status }} · evidence {{ item.source_count }}
                    </div>
                  </div>
                  <div class="memory-score">{{ item.importance }}</div>
                </div>
                <div class="memory-summary">{{ item.summary || item.detail || "暂无摘要" }}</div>
                <div class="memory-tags">
                  <span v-for="tag in item.tags" :key="tag" class="tag-chip">{{ tag }}</span>
                </div>
              </article>
            </div>
          </div>

          <div v-if="personalMemoryStore.searchResults.length > 0" class="memory-list">
            <div class="list-title">Search Results</div>
            <div class="memory-cards">
              <article
                v-for="item in personalMemoryStore.searchResults"
                :key="`search-${item.id}`"
                class="memory-card compact"
              >
                <div class="memory-card-head">
                  <div>
                    <div class="memory-title">{{ item.title }}</div>
                    <div class="memory-meta">{{ item.memory_type }} · {{ item.last_seen_date || "no date" }}</div>
                  </div>
                  <div class="memory-score">{{ item.importance }}</div>
                </div>
                <div class="memory-summary">{{ item.summary }}</div>
              </article>
            </div>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">
          DEBUG WORKBENCH<span class="sub">上下文包 / 镜像 / Patch</span>
        </div>
        <div class="debug-panel cyber-panel">
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
            <button class="cyber-btn primary" @click="handleBuildContextPack">
              构建 Context Pack
            </button>
            <button class="cyber-btn primary" @click="handleRebuildMirror">
              重建 rag_memory 镜像
            </button>
            <button class="cyber-btn primary" @click="handleExportRagSnapshot">
              导出 RAG 快照
            </button>
          </div>

          <div class="form-group">
            <label class="form-label">PersonalMemoryPatch v1</label>
            <n-input
              v-model:value="patchJson"
              type="textarea"
              :autosize="{ minRows: 10, maxRows: 18 }"
              placeholder="在这里粘贴 AI 输出的结构化 patch"
            />
          </div>

          <div class="form-actions wrap">
            <button class="cyber-btn primary" :disabled="personalMemoryStore.patching" @click="handleApplyPatch">
              应用 Patch
            </button>
            <div v-if="personalMemoryStore.lastPatchResult" class="patch-status">
              {{ personalMemoryStore.lastPatchResult.apply_status }}
              · applied {{ personalMemoryStore.lastPatchResult.applied_operations }}
              · rejected {{ personalMemoryStore.lastPatchResult.rejected_operations }}
            </div>
          </div>

          <div class="form-group">
            <label class="form-label">Diagnostic Output</label>
            <n-input
              :value="diagnosticOutput"
              type="textarea"
              readonly
              :autosize="{ minRows: 10, maxRows: 18 }"
              placeholder="这里会显示 context pack、manifest 或 patch 结果"
            />
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="cyber-section-title">
          DATA MANAGEMENT<span class="sub">完整导出与导入</span>
        </div>
        <div class="data-panel cyber-panel">
          <div class="data-item">
            <div class="data-info">
              <div class="data-title">EXPORT DATA AS JSON</div>
              <div class="data-desc">导出完整数据库对象</div>
              <div class="data-hint">
                包含 records、ledger、plans、bond、journals、personal memory 和公开设置。
              </div>
            </div>
            <button class="cyber-btn primary" @click="handleExport">
              导出 JSON
            </button>
          </div>

          <div class="data-divider"></div>

          <div class="data-item">
            <div class="data-info">
              <div class="data-title">IMPORT CSV OR JSON</div>
              <div class="data-desc">兼容旧记录数组和新的对象导出</div>
              <div class="data-hint">
                JSON 导入目前优先恢复 records，并可同步 personal_profile。
              </div>
            </div>
            <label class="cyber-btn primary file-btn">
              选择文件
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

.settings-section {
  position: relative;
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
.data-hint {
  font-size: 13px;
  color: var(--cyber-text-dim);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.stat-tile {
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

.memory-list {
  margin-top: 18px;
}

.memory-cards {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 10px;
}

.memory-card {
  padding: 14px;
  background: rgba(8, 18, 42, 0.88);
  border: 1px solid rgba(0, 212, 255, 0.14);
  border-radius: 6px;
}

.memory-card.compact {
  padding: 12px;
}

.memory-card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.memory-title,
.data-title {
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
  .memory-cards {
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
