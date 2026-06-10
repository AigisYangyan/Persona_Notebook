import { describe, expect, it } from "vitest";
import {
  mapSettingsFromBackend,
  toGeneralSettingsPayload,
} from "@/stores/settingMapper";

describe("settingMapper", () => {
  it("normalizes backend settings to the rules+api mode without exposing api key", () => {
    const mapped = mapSettingsFromBackend({
      scoring_engine: "local",
      api_base_url: "https://api.example.com/v1",
      api_model: "gpt-4o-mini",
      api_key_configured: true,
    });

    expect(mapped).toEqual({
      scoringEngine: "rules_api",
      apiBaseUrl: "https://api.example.com/v1",
      apiModel: "gpt-4o-mini",
      apiKeyConfigured: true,
    });
  });

  it("creates the general settings payload expected by Rust", () => {
    expect(
      toGeneralSettingsPayload({
        scoringEngine: "rules_api",
        apiBaseUrl: "",
        apiModel: "gpt-4o-mini",
        apiKeyConfigured: false,
      })
    ).toEqual({
      scoring_engine: "rules_api",
      api_base_url: "",
      api_model: "gpt-4o-mini",
    });
  });
});
