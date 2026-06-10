import { describe, expect, it } from "vitest";
import { cleanScoreResponse, parseScoreResponse } from "@/api/adapter/parseScoreResponse";

describe("parseScoreResponse", () => {
  it("removes markdown fences before parsing", () => {
    const cleaned = cleanScoreResponse(
      "```json\n{\"version\":\"1.0\",\"date\":\"2026-06-10\",\"total_changes\":{\"knowledge\":1},\"record_results\":[],\"summary\":\"ok\"}\n```"
    );

    expect(cleaned.startsWith("{")).toBe(true);
  });

  it("parses a valid response", () => {
    const response = parseScoreResponse(
      "{\"version\":\"1.0\",\"date\":\"2026-06-10\",\"total_changes\":{\"knowledge\":1},\"record_results\":[],\"summary\":\"ok\"}"
    );

    expect(response.summary).toBe("ok");
  });
});
