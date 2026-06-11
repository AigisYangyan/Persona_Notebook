<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  NInput,
  useMessage,
} from "naive-ui";
import { exportDataJson, importCsv, importJson } from "@/api/client/tauriCommands";
import { useSettingStore } from "@/stores/settingStore";

const settingStore = useSettingStore();
const message = useMessage();

const apiBaseUrl = ref("");
const apiModel = ref("");
const apiKey = ref("");
const savingApiConfig = ref(false);
const savingApiKey = ref(false);

onMounted(async () => {
  await settingStore.loadSettings();
  apiBaseUrl.value = settingStore.settings.apiBaseUrl;
  apiModel.value = settingStore.settings.apiModel;
});

async function handleSaveApiConfig() {
  savingApiConfig.value = true;
  settingStore.setEngine("rules_api");
  settingStore.updateApiConfig(apiBaseUrl.value, apiModel.value);
  try {
    await settingStore.persistGeneralSettings();
    message.success("API 配置已保存");
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

async function handleExport() {
  try {
    const json = await exportDataJson();
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `pgrn-export-${new Date().toISOString().slice(0, 10)}.json`;
    link.click();
    URL.revokeObjectURL(url);
    message.success("导出成功");
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
      message.error("不支持的文件格式，请上传 .csv 或 .json");
      return;
    }

    if (result.errors.length > 0) {
      message.warning(`导入完成: ${result.imported} 条成功, ${result.errors.length} 条失败`);
    } else {
      message.success(`成功导入 ${result.imported} 条记录`);
    }
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
      <!-- Scoring Engine -->
      <div class="settings-section">
        <div class="cyber-section-title">
          SCORING ENGINE<span class="sub">评分引擎</span>
        </div>
        <div class="info-panel cyber-panel">
          <p class="info-text">
            This app uses Rules + API scoring. The Rules generate a preliminary score based on configured weights,
            and then send it to the selected AI model via API for final scoring.
            If the API is unavailable, the app will fall back to Rules-only scoring.
          </p>
          <p class="info-text info-cn">
            当前配置使用 <strong>Rules + API 评分引擎</strong>。本地 Rules 先生成结构化分类与分值建议，再把这些约束一并提交给 API 复核，
            不再提供独立离线评分，也不使用无约束的纯 API 打分。当所选模型接口不可用时，将仅使用本地 Rules 评分。
          </p>
        </div>
      </div>

      <!-- API Configuration -->
      <div class="settings-section">
        <div class="cyber-section-title">
          API CONFIGURATION<span class="sub">API 配置</span>
        </div>
        <div class="config-panel cyber-panel">
          <div class="config-status">
            <span class="status-label">API KEY STATUS</span>
            <span
              class="status-badge"
              :class="settingStore.settings.apiKeyConfigured ? 'configured' : 'unconfigured'"
            >
              {{ settingStore.settings.apiKeyConfigured ? '已配置' : '未配置' }}
            </span>
          </div>

          <div class="config-form">
            <div class="form-group">
              <label class="form-label">
                <span class="label-en">BASE URL</span>
                <span class="label-cn">基础地址</span>
              </label>
              <n-input v-model:value="apiBaseUrl" placeholder="https://api.openai.com/v1" />
            </div>
            <div class="form-group">
              <label class="form-label">
                <span class="label-en">MODEL</span>
                <span class="label-cn">模型</span>
              </label>
              <n-input v-model:value="apiModel" placeholder="gpt-4o-mini" />
            </div>
            <div class="form-actions">
              <button class="cyber-btn primary" :disabled="savingApiConfig" @click="handleSaveApiConfig">
                <span class="btn-icon">✓</span>
                <div class="btn-text">
                  <span class="btn-label">SAVE CONFIG</span>
                  <span class="btn-sub">保存 API 配置</span>
                </div>
              </button>
            </div>
          </div>

          <div class="config-divider"></div>

          <div class="config-form">
            <div class="form-group">
              <label class="form-label">
                <span class="label-en">API KEY</span>
                <span class="label-cn">API 密钥</span>
              </label>
              <n-input
                v-model:value="apiKey"
                type="password"
                placeholder="输入新的 API Key"
                show-password-on="click"
              />
            </div>
            <div class="form-actions">
              <button class="cyber-btn primary" :disabled="savingApiKey" @click="handleSaveApiKey">
                <span class="btn-icon">↻</span>
                <div class="btn-text">
                  <span class="btn-label">UPDATE API KEY</span>
                  <span class="btn-sub">更新 API Key</span>
                </div>
              </button>
              <button class="cyber-btn danger" :disabled="savingApiKey" @click="handleClearApiKey">
                <span class="btn-icon">✕</span>
                <div class="btn-text">
                  <span class="btn-label">CLEAR API KEY</span>
                  <span class="btn-sub">清除 API Key</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Data Management -->
      <div class="settings-section">
        <div class="cyber-section-title">
          DATA MANAGEMENT<span class="sub">数据管理</span>
        </div>
        <div class="data-panel cyber-panel">
          <div class="data-item">
            <div class="data-icon">↓</div>
            <div class="data-info">
              <div class="data-title">EXPORT DATA AS JSON</div>
              <div class="data-desc">导出记录为 JSON</div>
              <div class="data-hint">将所有成长记录导出为 JSON 文件。</div>
            </div>
            <button class="cyber-btn primary" @click="handleExport">
              <span class="btn-label">EXPORT JSON</span>
              <span class="btn-sub">导出 JSON</span>
              <span class="btn-arrow">▶</span>
            </button>
          </div>

          <div class="data-divider"></div>

          <div class="data-item">
            <div class="data-icon">↑</div>
            <div class="data-info">
              <div class="data-title">IMPORT CLOCKLOG FILE</div>
              <div class="data-desc">导入 Clocklog 文件</div>
              <div class="data-hint">从 Clocklog 的 CSV 文件导入数据。</div>
            </div>
            <label class="cyber-btn primary file-btn">
              <span class="btn-label">IMPORT FILE</span>
              <span class="btn-sub">选择文件</span>
              <span class="btn-arrow">▶</span>
              <input type="file" accept=".csv,.json" style="display: none" @change="handleImportFile" />
            </label>
          </div>
        </div>
      </div>
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

/* Info Panel */
.info-panel {
  padding: 20px 24px;
}

.info-text {
  font-size: 13px;
  color: var(--cyber-text-secondary);
  line-height: 1.7;
  margin: 0 0 10px 0;
}

.info-text:last-child {
  margin-bottom: 0;
}

.info-text strong {
  color: var(--cyber-cyan);
  font-weight: 600;
}

.info-cn {
  color: var(--cyber-text-muted);
  font-size: 12px;
}

/* Config Panel */
.config-panel {
  padding: 20px 24px;
}

.config-status {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--cyber-border);
}

.status-label {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 2px;
  color: var(--cyber-cyan);
}

.status-badge {
  padding: 3px 12px;
  border-radius: 3px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
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

.config-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.label-en {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 2px;
  color: var(--cyber-text-secondary);
}

.label-cn {
  font-size: 11px;
  color: var(--cyber-text-dim);
}

.form-actions {
  display: flex;
  gap: 10px;
  margin-top: 4px;
}

.config-divider {
  height: 1px;
  background: linear-gradient(90deg, var(--cyber-border), transparent);
  margin: 20px 0;
}

/* Buttons */
.cyber-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
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

.cyber-btn.danger:hover {
  background: rgba(255, 51, 102, 0.15);
  border-color: var(--cyber-danger);
  box-shadow: 0 0 10px rgba(255, 51, 102, 0.15);
}

.cyber-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-icon {
  font-size: 14px;
}

.btn-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
}

.btn-label {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 1px;
}

.btn-sub {
  font-size: 10px;
  color: var(--cyber-text-muted);
  letter-spacing: 1px;
}

.btn-arrow {
  margin-left: 4px;
  font-size: 10px;
}

/* Data Panel */
.data-panel {
  padding: 16px 24px;
}

.data-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 0;
}

.data-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 60, 120, 0.15);
  border: 1px solid var(--cyber-border);
  border-radius: 4px;
  font-size: 18px;
  color: var(--cyber-cyan);
}

.data-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.data-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--cyber-text-primary);
}

.data-desc {
  font-size: 12px;
  color: var(--cyber-text-secondary);
}

.data-hint {
  font-size: 11px;
  color: var(--cyber-text-dim);
}

.data-divider {
  height: 1px;
  background: linear-gradient(90deg, var(--cyber-border), transparent);
}

.file-btn {
  cursor: pointer;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  min-width: 120px;
}

.file-btn .btn-label {
  font-size: 12px;
}

.file-btn .btn-sub {
  font-size: 10px;
}

.file-btn .btn-arrow {
  font-size: 10px;
  margin-left: 0;
}
</style>
