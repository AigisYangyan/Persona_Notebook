<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  NButton,
  NCard,
  NForm,
  NFormItem,
  NInput,
  NSpace,
  NTag,
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
      console.log("Import errors:", result.errors);
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
  <div>
    <h1 style="margin-top: 0">设置</h1>
    <n-space vertical :size="16">
      <n-card title="评分引擎">
        <p style="color: #666; font-size: 13px; margin-top: 8px">
          当前固定使用 <strong>Rules + API 反馈引擎</strong>。
          本地 Rules 先生成结构化分类与分值建议，再把这些约束一并提交给 API 复核，
          不再提供独立离线评分，也不使用无约束的纯 API 打分。
        </p>
      </n-card>

      <n-card title="API 配置">
        <n-space vertical :size="12">
          <n-tag :type="settingStore.settings.apiKeyConfigured ? 'success' : 'warning'">
            {{ settingStore.settings.apiKeyConfigured ? "API Key 已配置" : "API Key 未配置" }}
          </n-tag>

          <n-form label-placement="left" label-width="100">
            <n-form-item label="Base URL">
              <n-input v-model:value="apiBaseUrl" placeholder="https://api.openai.com/v1" />
            </n-form-item>
            <n-form-item label="Model">
              <n-input v-model:value="apiModel" placeholder="gpt-4o-mini" />
            </n-form-item>
            <n-button type="primary" :loading="savingApiConfig" @click="handleSaveApiConfig">
              保存 API 配置
            </n-button>
          </n-form>

          <n-form label-placement="left" label-width="100">
            <n-form-item label="API Key">
              <n-input
                v-model:value="apiKey"
                type="password"
                placeholder="输入新的 API Key"
                show-password-on="click"
              />
            </n-form-item>
            <n-space>
              <n-button type="primary" :loading="savingApiKey" @click="handleSaveApiKey">
                更新 API Key
              </n-button>
              <n-button ghost type="error" :loading="savingApiKey" @click="handleClearApiKey">
                清除 API Key
              </n-button>
            </n-space>
          </n-form>
        </n-space>
      </n-card>

      <n-card title="数据管理">
        <n-space vertical>
          <n-button @click="handleExport">导出记录为 JSON</n-button>
          <div>
            <label style="display: block; margin-bottom: 8px; font-size: 14px">
              导入 Clocklog 文件
            </label>
            <input type="file" accept=".csv,.json" @change="handleImportFile" />
            <p style="color: #999; font-size: 12px; margin-top: 4px">
              支持 .csv 和 .json 格式
            </p>
          </div>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>
