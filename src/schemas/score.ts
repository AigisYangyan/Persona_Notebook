import { z } from "zod";

export const DailyAnalysisRequestSchema = z.object({
  version: z.string().default("1.0"),
  feedback_mode: z.literal("rules_api").default("rules_api"),
  date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
  records: z
    .array(
      z.object({
        title: z.string().min(1).max(200),
        minutes: z.number().int().min(0).max(1440),
        difficulty_star: z.number().int().min(0).max(3),
      })
    )
    .min(1)
    .max(50),
  stat_dimensions: z.array(
    z.object({
      key: z.string(),
      name: z.string(),
      daily_cap: z.number().int().positive(),
    })
  ),
  score_rules: z.object({
    time_base: z.record(z.string(), z.number().int()),
    difficulty_multiplier: z.record(z.string(), z.number()),
    max_dims_per_record: z.number().int().positive(),
    allocation_ratio: z.object({
      primary: z.number(),
      secondary: z.number(),
    }),
  }),
  rule_hints: z.object({
    source: z.literal("deterministic_rules_cache"),
    summary: z.string().min(1),
    suggested_totals: z.record(z.string(), z.number().int().min(0)),
    record_hints: z
      .array(
        z.object({
          record_index: z.number().int().min(0),
          title: z.string().min(1).max(200),
          category: z.string().min(1),
          suggested_dimensions: z.array(z.string()).max(3),
          suggested_changes: z.record(z.string(), z.number().int().min(0)),
          confidence: z.number().min(0).max(1),
          reason: z.string().min(1),
        })
      )
      .min(1)
      .max(50),
  }),
});

export const DailyAnalysisResponseSchema = z.object({
  version: z.string().default("1.0"),
  date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
  total_changes: z.record(z.string(), z.number().int().min(0)),
  record_results: z.array(
    z.object({
      title: z.string(),
      category: z.string(),
      changes: z.record(z.string(), z.number().int().min(0)),
      difficulty_star: z.number().int().min(0).max(3),
      confidence: z.number().min(0).max(1),
      reason: z.string().min(1),
    })
  ),
  summary: z.string().min(1),
});

export type DailyAnalysisRequest = z.infer<typeof DailyAnalysisRequestSchema>;
export type DailyAnalysisResponse = z.infer<typeof DailyAnalysisResponseSchema>;
