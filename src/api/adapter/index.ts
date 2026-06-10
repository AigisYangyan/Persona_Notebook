import type { DailyAnalysisRequest, DailyAnalysisResponse } from "@/schemas/score";

export interface ScoringAdapter {
  name: string;
  score(request: DailyAnalysisRequest): Promise<DailyAnalysisResponse>;
}
