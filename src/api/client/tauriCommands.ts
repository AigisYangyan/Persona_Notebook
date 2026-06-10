import { invoke } from "@tauri-apps/api/core";

export interface RecordItem {
  id: number;
  date: string;
  title: string;
  minutes: number;
  difficulty_star: number;
}

export interface DimensionTotal {
  key: string;
  name: string;
  total: number;
}

export interface ScoreRecordInput {
  id: number;
  title: string;
  minutes: number;
  difficulty_star: number;
}

export interface ScorePreviewChange {
  dimension_key: string;
  change_value: number;
}

export interface ScorePreviewItem {
  record_id: number | null;
  title: string;
  category: string;
  changes: ScorePreviewChange[];
  difficulty_star: number;
  confidence: number;
  reason: string;
  engine: string;
}

export interface LedgerEntry {
  id: number;
  date: string;
  dimension_key: string;
  dimension_name: string;
  change_value: number;
  source_title: string;
  reason: string;
  engine: string;
}

export interface LedgerByDateEntry {
  id: number;
  date: string;
  dimension_key: string;
  change_value: number;
  source_title: string;
  reason: string;
}

export interface ImportResult {
  imported: number;
  errors: string[];
}

export interface StreakInfo {
  current_streak: number;
  longest_streak: number;
}

export interface AppSettings {
  scoring_engine: string;
  api_base_url: string;
  api_model: string;
  api_key_configured: boolean;
}

export interface GeneralSettingsPayload {
  scoring_engine: string;
  api_base_url: string;
  api_model: string;
}

export interface CalendarOverviewItem {
  date: string;
  record_count: number;
  is_analyzed: boolean;
}

export async function getRecordsByDate(date: string): Promise<RecordItem[]> {
  return invoke("get_records_by_date", { date });
}

export async function createRecord(
  date: string,
  title: string,
  minutes: number,
  difficultyStar: number
): Promise<RecordItem> {
  return invoke("create_record", {
    date,
    title,
    minutes,
    difficultyStar,
  });
}

export async function deleteRecord(id: number): Promise<void> {
  return invoke("delete_record", { id });
}

export async function getDimensionTotals(): Promise<DimensionTotal[]> {
  return invoke("get_dimension_totals");
}

export async function previewScoreWithLocalRules(
  records: ScoreRecordInput[]
): Promise<ScorePreviewItem[]> {
  return invoke("preview_score_with_local_rules", { records });
}

export async function callScoringApi(requestJson: string): Promise<string> {
  return invoke("call_scoring_api", { requestJson });
}

export async function confirmScorePreview(
  date: string,
  items: ScorePreviewItem[],
  summary: string | null
): Promise<void> {
  return invoke("confirm_score_preview", { date, items, summary });
}

export async function getAllLedger(limit: number): Promise<LedgerEntry[]> {
  return invoke("get_all_ledger", { limit });
}

export async function getLedgerByDate(date: string): Promise<LedgerByDateEntry[]> {
  return invoke("get_ledger_by_date", { date });
}

export async function rollbackLedger(ledgerId: number): Promise<void> {
  return invoke("rollback_ledger", { ledgerId });
}

export async function getStreakInfo(): Promise<StreakInfo> {
  return invoke("get_streak_info", {});
}

export async function getCalendarOverview(
  year: number,
  month: number
): Promise<CalendarOverviewItem[]> {
  return invoke("get_calendar_overview", { year, month });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings", {});
}

export async function saveGeneralSettings(
  settings: GeneralSettingsPayload
): Promise<void> {
  return invoke("save_general_settings", {
    scoringEngine: settings.scoring_engine,
    apiBaseUrl: settings.api_base_url,
    apiModel: settings.api_model,
  });
}

export async function saveApiKey(apiKey: string): Promise<void> {
  return invoke("save_api_key", { apiKey });
}

export async function clearApiKey(): Promise<void> {
  return invoke("clear_api_key", {});
}

export async function importCsv(content: string): Promise<ImportResult> {
  return invoke("import_csv", { content });
}

export async function importJson(content: string): Promise<ImportResult> {
  return invoke("import_json", { content });
}

export async function exportDataJson(): Promise<string> {
  return invoke("export_data", {});
}
