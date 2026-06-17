import type { ScoringAdapter } from "./index";
import type { DailyAnalysisRequest, DailyAnalysisResponse } from "@/schemas/score";
import { callScoringApi } from "@/api/client/tauriCommands";
import { parseScoreResponse } from "@/api/adapter/parseScoreResponse";
import { validateDailyResponse } from "@/validators/scoreValidator";

export class OpenAIAdapter implements ScoringAdapter {
  name = "Rules + API Feedback Engine";

  async score(request: DailyAnalysisRequest): Promise<DailyAnalysisResponse> {
    const requestJson = JSON.stringify(request);
    const rawResponse = await callScoringApi(requestJson);
    const response = parseScoreResponse(rawResponse);

    // Business validation
    const validation = validateDailyResponse(request, response);
    if (!validation.valid) {
      const messages = validation.errors.map((e) => `${e.rule}: ${e.message}`).join("\n");
      throw new Error(`校验失败:\n${messages}`);
    }

    return response;
  }
}
