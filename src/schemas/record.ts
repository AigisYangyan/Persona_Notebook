import { z } from "zod";

export const RecordSchema = z.object({
  id: z.number().optional(),
  date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
  title: z.string().min(1).max(200),
  minutes: z.number().int().min(0).max(1440),
  difficulty_star: z.number().int().min(0).max(3),
  parent_id: z.number().nullable().optional(),
  is_completed: z.boolean().optional(),
  completed_at: z.string().nullable().optional(),
  elapsed_seconds: z.number().int().min(0).optional(),
  timer_mode: z.enum(["stopwatch", "countdown"]).optional(),
  countdown_target_seconds: z.number().int().positive().nullable().optional(),
  timer_started_at: z.string().nullable().optional(),
  note: z.string().optional(),
  clocklog_id: z.string().optional(),
});

export type Record = z.infer<typeof RecordSchema>;
