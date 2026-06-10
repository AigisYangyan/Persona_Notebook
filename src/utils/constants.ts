export const TIME_BASE_SCORES: Record<string, number> = {
  "0-15": 1,
  "16-30": 2,
  "31-60": 3,
  "61-90": 4,
  "91-120": 5,
  "121-180": 6,
  "181+": 7,
};

export const DIFFICULTY_MULTIPLIERS: Record<number, number> = {
  0: 1.0,
  1: 0.9,
  2: 1.0,
  3: 1.2,
};

export function getTimeBaseScore(minutes: number): number {
  if (minutes <= 15) return 1;
  if (minutes <= 30) return 2;
  if (minutes <= 60) return 3;
  if (minutes <= 90) return 4;
  if (minutes <= 120) return 5;
  if (minutes <= 180) return 6;
  return 7;
}
