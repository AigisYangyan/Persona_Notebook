import { z } from "zod";

export const RecordSchema = z.object({
  id: z.number().optional(),
  date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
  title: z.string().min(1).max(200),
  minutes: z.number().int().min(0).max(1440),
  difficulty_star: z.number().int().min(0).max(3),
  note: z.string().optional(),
  clocklog_id: z.string().optional(),
});

export type Record = z.infer<typeof RecordSchema>;
