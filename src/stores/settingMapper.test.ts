import { describe, expect, it } from "vitest";
import {
  mapSettingsFromBackend,
  toGeneralSettingsPayload,
} from "@/stores/settingMapper";

describe("settingMapper", () => {
  it("normalizes backend settings to the rules+api mode without exposing api key", () => {
    const mapped = mapSettingsFromBackend({
      scoring_engine: "local",
      deepseek_base_url: "https://api.deepseek.com/v1",
      deepseek_flash_model: "deepseek-chat",
      deepseek_pro_model: "deepseek-reasoner",
      api_key_configured: true,
    });

    expect(mapped).toEqual({
      scoringEngine: "rules_api",
      deepseekBaseUrl: "https://api.deepseek.com/v1",
      deepseekFlashModel: "deepseek-chat",
      deepseekProModel: "deepseek-reasoner",
      apiKeyConfigured: true,
    });
  });

  it("falls back across DeepSeek tiers when one side is empty", () => {
    const mapped = mapSettingsFromBackend({
      scoring_engine: "rules_api",
      deepseek_base_url: "",
      deepseek_flash_model: "",
      deepseek_pro_model: "deepseek-reasoner",
      api_key_configured: false,
    });

    expect(mapped.deepseekBaseUrl).toBe("https://api.deepseek.com/v1");
    expect(mapped.deepseekFlashModel).toBe("deepseek-reasoner");
    expect(mapped.deepseekProModel).toBe("deepseek-reasoner");
  });

  it("creates the general settings payload expected by Rust", () => {
    expect(
      toGeneralSettingsPayload({
        scoringEngine: "rules_api",
        deepseekBaseUrl: "https://api.deepseek.com/v1",
        deepseekFlashModel: "deepseek-chat",
        deepseekProModel: "deepseek-reasoner",
        apiKeyConfigured: false,
      })
    ).toEqual({
      scoring_engine: "rules_api",
      deepseek_base_url: "https://api.deepseek.com/v1",
      deepseek_flash_model: "deepseek-chat",
      deepseek_pro_model: "deepseek-reasoner",
    });
  });
});
