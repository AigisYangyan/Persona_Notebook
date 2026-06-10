import type { DailyAnalysisRequest } from "@/schemas/score";

export function buildUserPrompt(request: DailyAnalysisRequest): string {
  return `请分析以下当日任务记录，并返回 JSON 评分结果。

【日期】
${request.date}

【任务记录】
${JSON.stringify(request.records, null, 2)}

【五维定义与单日上限】
${JSON.stringify(request.stat_dimensions, null, 2)}

【评分规则】
${JSON.stringify(request.score_rules, null, 2)}

【Rules 缓存】
${JSON.stringify(request.rule_hints, null, 2)}

【计算说明】
1. 当前模式为 ${request.feedback_mode}，Rules 缓存是强约束参考，不能无视。
2. 先根据时间区间查基础分，再乘以难度系数，四舍五入得单条原始分。
3. 主维度获得约 70% 原始分，次维度获得约 30%。
4. 累加所有任务的各维度得分后，用单日上限裁剪，得到最终 total_changes。
5. record_results 中每条任务的 changes 为裁剪前的原始分配值。

请严格返回 JSON，不要输出 Markdown 或任何解释文字。`;
}
