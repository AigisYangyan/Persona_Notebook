const TITLE_KEYS = ["item", "title", "name", "label", "topic", "card_name", "archetype"];
const BODY_KEYS = ["description", "detail", "summary", "text", "content", "message", "reason", "value"];
const FALLBACK_KEYS = [...BODY_KEYS, ...TITLE_KEYS];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function normalizeText(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function pickObjectText(record: Record<string, unknown>): string {
  const title = TITLE_KEYS.map((key) => coerceInsightText(record[key])).find(Boolean) || "";
  const body = BODY_KEYS.map((key) => coerceInsightText(record[key])).find(Boolean) || "";
  if (title && body) {
    return body.includes(title) ? body : `${title}: ${body}`;
  }
  if (body) {
    return body;
  }
  if (title) {
    return title;
  }
  for (const key of FALLBACK_KEYS) {
    const text = coerceInsightText(record[key]);
    if (text) {
      return text;
    }
  }
  for (const [key, value] of Object.entries(record)) {
    if (key === "evidence_ids" || key === "evidence_id") {
      continue;
    }
    const text = coerceInsightText(value);
    if (text) {
      return text;
    }
  }
  return "";
}

export function coerceInsightList(value: unknown): string[] {
  if (Array.isArray(value)) {
    return value
      .flatMap((item) => {
        const text = coerceInsightText(item);
        return text ? [text] : [];
      })
      .filter(Boolean);
  }
  const text = coerceInsightText(value);
  return text ? [text] : [];
}

export function coerceInsightText(value: unknown, fallback = ""): string {
  if (typeof value === "string") {
    return normalizeText(value) || fallback;
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (Array.isArray(value)) {
    const list = coerceInsightList(value);
    return list.length > 0 ? list.join("; ") : fallback;
  }
  if (isRecord(value)) {
    return pickObjectText(value) || fallback;
  }
  return fallback;
}
