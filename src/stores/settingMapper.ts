import type {
  AppSettings as BackendAppSettings,
  GeneralSettingsPayload,
} from "@/api/client/tauriCommands";
import type { AppSettings as StoreAppSettings } from "@/stores/settingStore";

export function mapSettingsFromBackend(
  settings: BackendAppSettings
): StoreAppSettings {
  return {
    scoringEngine: "rules_api",
    deepseekBaseUrl: settings.deepseek_base_url || "https://api.deepseek.com/v1",
    deepseekFlashModel: settings.deepseek_flash_model || settings.deepseek_pro_model || "",
    deepseekProModel: settings.deepseek_pro_model || settings.deepseek_flash_model || "",
    apiKeyConfigured: settings.api_key_configured,
  };
}

export function toGeneralSettingsPayload(
  settings: StoreAppSettings
): GeneralSettingsPayload {
  return {
    scoring_engine: settings.scoringEngine,
    deepseek_base_url: settings.deepseekBaseUrl,
    deepseek_flash_model: settings.deepseekFlashModel,
    deepseek_pro_model: settings.deepseekProModel,
  };
}
