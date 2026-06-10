import { DailyAnalysisResponseSchema, type DailyAnalysisResponse } from "@/schemas/score";

export function cleanScoreResponse(rawResponse: string): string {
  let cleaned = rawResponse.trim();
  if (cleaned.startsWith("```")) {
    cleaned = cleaned.replace(/^```[\w]*\n?/, "").replace(/\n?```$/, "");
  }
  return cleaned.replace(/^\uFEFF/, "").trim();
}

export function parseScoreResponse(rawResponse: string): DailyAnalysisResponse {
  const cleaned = cleanScoreResponse(rawResponse);

  let parsed: unknown;
  try {
    parsed = JSON.parse(cleaned);
  } catch (error) {
    throw new Error(`API 返回非合法 JSON: ${error}`);
  }

  const result = DailyAnalysisResponseSchema.safeParse(parsed);
  if (!result.success) {
    throw new Error(`Schema 校验失败: ${result.error.message}`);
  }

  return result.data;
}
