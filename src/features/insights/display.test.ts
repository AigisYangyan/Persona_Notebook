import { describe, expect, it } from "vitest";
import { coerceInsightList, coerceInsightText } from "./display";

describe("insight display normalization", () => {
  it("converts report object arrays into readable list items", () => {
    const items = coerceInsightList([
      { description: "完成高数复习", evidence_ids: ["record:1"] },
      { item: "背单词", detail: "没有打卡", evidence_ids: ["plan_item:1"] },
    ]);

    expect(items).toEqual(["完成高数复习", "背单词: 没有打卡"]);
  });

  it("extracts readable text from structured single objects", () => {
    const text = coerceInsightText({
      description: "时间集中在复变函数和认知天性",
      evidence_ids: ["insight_report:1"],
    });

    expect(text).toBe("时间集中在复变函数和认知天性");
  });
});
