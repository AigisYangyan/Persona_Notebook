import type { DailyAnalysisRequest, DailyAnalysisResponse } from "@/schemas/score";

export interface ValidationError {
  rule: string;
  message: string;
  field?: string;
}

export function validateDailyResponse(
  request: DailyAnalysisRequest,
  response: DailyAnalysisResponse
): { valid: boolean; errors: ValidationError[] } {
  const errors: ValidationError[] = [];

  // Rule 2: date consistency
  if (response.date !== request.date) {
    errors.push({
      rule: "RULE_02_DATE",
      message: `日期不匹配: ${response.date} ≠ ${request.date}`,
    });
  }

  // Rule 3: dimension completeness
  const requestKeys = new Set(request.stat_dimensions.map((d) => d.key));
  const responseKeys = Object.keys(response.total_changes);
  for (const key of requestKeys) {
    if (!responseKeys.includes(key)) {
      errors.push({ rule: "RULE_03_DIMS", message: `缺少维度: ${key}`, field: key });
    }
  }

  // Rule 4: record count match
  if (response.record_results.length !== request.records.length) {
    errors.push({
      rule: "RULE_04_COUNT",
      message: `记录数量不匹配: ${response.record_results.length} ≠ ${request.records.length}`,
    });
  }

  // Rule 5 & 6: invalid dimension + max 3 dims per record
  for (let i = 0; i < response.record_results.length; i++) {
    const rr = response.record_results[i];
    const changeKeys = Object.keys(rr.changes);
    for (const k of changeKeys) {
      if (!requestKeys.has(k)) {
        errors.push({
          rule: "RULE_05_INVALID_DIM",
          message: `非法维度: ${k}`,
          field: `record[${i}]`,
        });
      }
    }
    if (changeKeys.length > 3) {
      errors.push({
        rule: "RULE_06_MAX_DIM",
        message: `影响维度过多: ${changeKeys.length} > 3`,
        field: `record[${i}]`,
      });
    }
  }

  // Rule 7: total consistency (before cap)
  const computedTotals: Record<string, number> = {};
  for (const dim of request.stat_dimensions) computedTotals[dim.key] = 0;
  for (const rr of response.record_results) {
    for (const [k, v] of Object.entries(rr.changes)) {
      computedTotals[k] = (computedTotals[k] || 0) + v;
    }
  }
  const caps = Object.fromEntries(
    request.stat_dimensions.map((d) => [d.key, d.daily_cap])
  );
  for (const key of requestKeys) {
    const capped = Math.min(computedTotals[key] || 0, caps[key] || Infinity);
    if (response.total_changes[key] !== capped) {
      errors.push({
        rule: "RULE_07_TOTAL",
        message: `总分不一致: ${key}=${response.total_changes[key]} ≠ 计算值(${capped})`,
        field: key,
      });
    }
  }

  // Rule 8: daily cap check
  for (const dim of request.stat_dimensions) {
    if ((response.total_changes[dim.key] || 0) > dim.daily_cap) {
      errors.push({
        rule: "RULE_08_CAP",
        message: `超出单日上限: ${dim.name}=${response.total_changes[dim.key]} > ${dim.daily_cap}`,
        field: dim.key,
      });
    }
  }

  // Rule 9: reason completeness
  for (let i = 0; i < response.record_results.length; i++) {
    if (!response.record_results[i].reason?.trim()) {
      errors.push({
        rule: "RULE_09_REASON",
        message: `缺少 reason`,
        field: `record[${i}]`,
      });
    }
  }

  // Rule 10: confidence range
  for (let i = 0; i < response.record_results.length; i++) {
    const c = response.record_results[i].confidence;
    if (c < 0 || c > 1 || Number.isNaN(c)) {
      errors.push({
        rule: "RULE_10_CONFIDENCE",
        message: `置信度非法: ${c}`,
        field: `record[${i}]`,
      });
    }
  }

  return { valid: errors.length === 0, errors };
}
