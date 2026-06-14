import { ref } from "vue";
import { defineStore } from "pinia";
import {
  clearApiKey,
  getSettings,
  saveApiKey,
  saveGeneralSettings,
} from "@/api/client/tauriCommands";
import { mapSettingsFromBackend, toGeneralSettingsPayload } from "@/stores/settingMapper";

export type ScoringEngine = "rules_api";

export interface AppSettings {
  scoringEngine: ScoringEngine;
  deepseekBaseUrl: string;
  deepseekFlashModel: string;
  deepseekProModel: string;
  apiKeyConfigured: boolean;
}

export const useSettingStore = defineStore("setting", () => {
  const settings = ref<AppSettings>({
    scoringEngine: "rules_api",
    deepseekBaseUrl: "https://api.deepseek.com/v1",
    deepseekFlashModel: "",
    deepseekProModel: "",
    apiKeyConfigured: false,
  });

  async function loadSettings() {
    try {
      settings.value = mapSettingsFromBackend(await getSettings());
    } catch (error) {
      console.error("加载设置失败:", error);
    }
  }

  async function persistGeneralSettings() {
    try {
      await saveGeneralSettings(toGeneralSettingsPayload(settings.value));
    } catch (error) {
      console.error("保存设置失败:", error);
      throw error;
    }
  }

  async function persistApiKey(apiKey: string) {
    try {
      await saveApiKey(apiKey);
      settings.value = {
        ...settings.value,
        apiKeyConfigured: apiKey.trim().length > 0,
      };
    } catch (error) {
      console.error("保存 API Key 失败:", error);
      throw error;
    }
  }

  async function removeApiKey() {
    try {
      await clearApiKey();
      settings.value = {
        ...settings.value,
        apiKeyConfigured: false,
      };
    } catch (error) {
      console.error("清除 API Key 失败:", error);
      throw error;
    }
  }

  function setEngine(engine: ScoringEngine) {
    settings.value = {
      ...settings.value,
      scoringEngine: engine,
    };
  }

  function updateApiConfig(baseUrl: string, flashModel: string, proModel: string) {
    settings.value = {
      ...settings.value,
      deepseekBaseUrl: baseUrl,
      deepseekFlashModel: flashModel,
      deepseekProModel: proModel,
    };
  }

  return {
    settings,
    loadSettings,
    persistGeneralSettings,
    persistApiKey,
    removeApiKey,
    setEngine,
    updateApiConfig,
  };
});
