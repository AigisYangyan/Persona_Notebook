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
    apiBaseUrl: settings.api_base_url || "",
    apiModel: settings.api_model || "gpt-4o-mini",
    apiKeyConfigured: settings.api_key_configured,
  };
}

export function toGeneralSettingsPayload(
  settings: StoreAppSettings
): GeneralSettingsPayload {
  return {
    scoring_engine: settings.scoringEngine,
    api_base_url: settings.apiBaseUrl,
    api_model: settings.apiModel,
  };
}
