import { invoke } from "@tauri-apps/api/core";

export type TimerMode = "stopwatch" | "countdown";

export interface RecordItem {
  id: number;
  date: string;
  title: string;
  minutes: number;
  difficulty_star: number;
  parent_id?: number | null;
  is_completed?: boolean;
  completed_at?: string | null;
  elapsed_seconds?: number;
  timer_mode?: TimerMode;
  countdown_target_seconds?: number | null;
  timer_started_at?: string | null;
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
  dimension_name: string;
  change_value: number;
  source_title: string;
  reason: string;
  engine: string;
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
  deepseek_base_url: string;
  deepseek_flash_model: string;
  deepseek_pro_model: string;
  api_key_configured: boolean;
}

export interface GeneralSettingsPayload {
  scoring_engine: string;
  deepseek_base_url: string;
  deepseek_flash_model: string;
  deepseek_pro_model: string;
}

export interface CalendarOverviewItem {
  date: string;
  record_count: number;
  is_analyzed: boolean;
  has_week_plan_update: boolean;
  has_month_plan_update: boolean;
}

export interface DailyCloseoutStatus {
  date: string;
  has_ledger: boolean;
  ledger_count: number;
}

export interface BondPerson {
  id: number;
  name: string;
  relation_label: string;
  score: number;
  note: string;
  latest_entry_date: string | null;
  entry_count: number;
}

export interface BondEntry {
  id: number;
  person_id: number;
  entry_date: string;
  title: string;
  content: string;
}

export interface DailyJournal {
  id: number;
  entry_date: string;
  title: string;
  content: string;
  mood: string;
}

export interface PersonalProfile {
  birthday: string;
  personality: string;
  experiences: string;
  personal_notes: string;
  updated_at: string | null;
}

export interface PersonalMemoryViewItem {
  id: number;
  memory_type: string;
  title: string;
  summary: string;
  detail: string;
  tags: string[];
  importance: number;
  confidence: number;
  first_seen_date: string | null;
  last_seen_date: string | null;
  status: string;
  supersedes_id: number | null;
  created_by: string;
  source_count: number;
  evidence_ids: string[];
}

export interface PersonalMemoryOverview {
  total_items: number;
  active_items: number;
  pending_items: number;
  rejected_items: number;
  top_items: PersonalMemoryViewItem[];
}

export interface PersonalContextPack {
  schema_version: string;
  profile: PersonalProfile;
  high_priority_memories: PersonalMemoryViewItem[];
  relevant_memories: PersonalMemoryViewItem[];
  recent_memories: PersonalMemoryViewItem[];
  query_relevant_memories: PersonalMemoryViewItem[];
  overview: PersonalMemoryOverview;
  mode: string;
  date: string;
}

export interface PersonalMemoryPatchApplyResult {
  patch_run_id: number;
  validation_status: string;
  apply_status: string;
  applied_operations: number;
  rejected_operations: number;
  message: string;
}

export type InsightReportKind = "tarot" | "report";
export type InsightPeriodType = "day" | "week" | "month";

export interface InsightReport {
  id: number;
  report_kind: InsightReportKind;
  period_type: InsightPeriodType;
  start_date: string;
  end_date: string;
  title: string;
  summary: string;
  content_json: unknown;
  context_snapshot_id: number | null;
  status: string;
  error_message: string | null;
  memory_patch_apply_status: string | null;
  memory_patch_apply_message: string | null;
  created_at: string;
}

export interface InsightContextSnapshot {
  id: number;
  report_kind: InsightReportKind;
  period_type: InsightPeriodType;
  start_date: string;
  end_date: string;
  context_json: string;
  created_at: string;
}

export interface CalendarInsightHistory {
  date: string;
  tarot: InsightReport | null;
  daily_report: InsightReport | null;
  week_report: InsightReport | null;
  month_report: InsightReport | null;
}

export type PlanPeriodType = "week" | "month";

export interface PlanCycle {
  id: number;
  period_type: PlanPeriodType;
  start_date: string;
  end_date: string;
  title: string;
  summary: string;
  ai_summary: string;
  last_ai_run_at: string | null;
}

export interface PlanItem {
  id: number;
  cycle_id: number;
  title: string;
  description: string;
  dimension_key: string | null;
  progress_percent: number;
  ai_comment: string;
  sort_order: number;
  is_completed: boolean;
}

export interface PlanGrowthDimension {
  key: string;
  name: string;
  total: number;
  daily_cap: number;
  max_total: number;
  progress_percent: number;
  headline: string;
}

export interface PlanGrowthSnapshot {
  start_date: string;
  end_date: string;
  total_days: number;
  active_days: number;
  analyzed_days: number;
  record_count: number;
  total_change: number;
  dimensions: PlanGrowthDimension[];
}

export interface GoalProgressSummary {
  total_items: number;
  completed_items: number;
  active_goal_count: number;
  average_progress_percent: number;
}

export interface RelatedWeekPlan {
  cycle_id: number;
  start_date: string;
  end_date: string;
  title: string;
  summary: string;
  average_progress: number;
  total_items: number;
  completed_items: number;
}

export interface PlanSnapshot {
  cycle: PlanCycle;
  items: PlanItem[];
  goal_progress: GoalProgressSummary;
  growth: PlanGrowthSnapshot;
  related_weeks: RelatedWeekPlan[];
}

export interface CalendarPlanHistoryItem {
  cycle: PlanCycle;
  goal_progress: GoalProgressSummary;
  items: PlanItem[];
  is_historical: boolean;
}

export interface CalendarPlanHistory {
  date: string;
  week_plan: CalendarPlanHistoryItem | null;
  month_plan: CalendarPlanHistoryItem | null;
}

export interface PlanAiProposalItem {
  title: string;
  description: string;
  dimension_key: string | null;
  progress_percent: number;
  ai_comment: string;
  sort_order: number;
  is_completed: boolean;
}

export interface PlanAiProposal {
  title: string;
  summary: string;
  ai_summary: string;
  items: PlanAiProposalItem[];
}

export interface PlanAiOutcome {
  session_id: number;
  status: string;
  requires_clarification: boolean;
  questions: string[];
  proposal: PlanAiProposal | null;
}

export type CloseoutStepStatus = "pending" | "skipped" | "success" | "needs_clarification" | "error";

export interface CloseoutStep {
  status: CloseoutStepStatus;
  message: string;
  report_id: number | null;
  session_id: number | null;
  questions: string[];
}

export interface GlobalCloseoutResult {
  date: string;
  scope: "day" | "week" | "month" | "all" | string;
  score: CloseoutStep;
  report: CloseoutStep;
  week_plan: CloseoutStep;
  month_plan: CloseoutStep;
  closeout_run_id: number;
}

export interface ApiRunDiagnostic {
  id: number;
  date: string;
  status: string;
  error_message: string | null;
  latency_ms: number | null;
  engine_name: string;
  task_kind: string;
  model_tier: string;
  fallback_used: boolean;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  prompt_cache_hit_tokens: number | null;
  prompt_cache_miss_tokens: number | null;
  finish_reason: string | null;
  created_at: string;
}

export async function getRecordsByDate(date: string): Promise<RecordItem[]> {
  return invoke("get_records_by_date", { date });
}

export async function createRecord(
  date: string,
  title: string,
  difficultyStar: number,
  timerMode: TimerMode,
  countdownTargetSeconds: number | null
): Promise<RecordItem> {
  return invoke("create_record", {
    date,
    title,
    difficultyStar,
    timerMode,
    countdownTargetSeconds,
  });
}

export async function createSubRecord(
  parentId: number,
  title: string,
  difficultyStar: number,
  timerMode: TimerMode,
  countdownTargetSeconds: number | null
): Promise<RecordItem> {
  return invoke("create_sub_record", {
    parentId,
    title,
    difficultyStar,
    timerMode,
    countdownTargetSeconds,
  });
}

export async function createManualRecordEntry(payload: {
  date: string;
  title: string;
  minutes: number;
  difficultyStar: number;
  parentId?: number | null;
}): Promise<RecordItem> {
  return invoke("create_manual_record_entry", {
    date: payload.date,
    title: payload.title,
    minutes: payload.minutes,
    difficultyStar: payload.difficultyStar,
    parentId: payload.parentId ?? null,
  });
}

export async function deleteRecord(id: number): Promise<void> {
  return invoke("delete_record", { id });
}

export async function startRecordTimer(id: number): Promise<RecordItem> {
  return invoke("start_record_timer", { id });
}

export async function pauseRecordTimer(id: number): Promise<RecordItem> {
  return invoke("pause_record_timer", { id });
}

export async function resetRecordTimer(id: number): Promise<RecordItem> {
  return invoke("reset_record_timer", { id });
}

export async function toggleRecordCompleted(id: number, isCompleted: boolean): Promise<RecordItem> {
  return invoke("toggle_record_completed", { id, isCompleted });
}

export async function updateRecordTimerMode(
  id: number,
  timerMode: TimerMode,
  countdownTargetSeconds: number | null
): Promise<RecordItem> {
  return invoke("update_record_timer_mode", { id, timerMode, countdownTargetSeconds });
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

export async function getRecentApiRuns(limit = 12): Promise<ApiRunDiagnostic[]> {
  return invoke("get_recent_api_runs", { limit });
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

export async function getDailyCloseoutStatus(date: string): Promise<DailyCloseoutStatus> {
  return invoke("get_daily_closeout_status", { date });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings", {});
}

export async function getPersonalProfile(): Promise<PersonalProfile> {
  return invoke("get_personal_profile", {});
}

export async function savePersonalProfile(profile: {
  birthday: string;
  personality: string;
  experiences: string;
  personal_notes: string;
}): Promise<PersonalProfile> {
  return invoke("save_personal_profile", {
    birthday: profile.birthday,
    personality: profile.personality,
    experiences: profile.experiences,
    personalNotes: profile.personal_notes,
  });
}

export async function getPersonalMemoryOverview(): Promise<PersonalMemoryOverview> {
  return invoke("get_personal_memory_overview", {});
}

export async function searchPersonalMemory(
  query?: string | null,
  tags?: string[] | null,
  limit?: number | null
): Promise<PersonalMemoryViewItem[]> {
  return invoke("search_personal_memory", {
    query: query ?? null,
    tags: tags ?? null,
    limit: limit ?? null,
  });
}

export async function buildPersonalContextPack(
  date?: string | null,
  mode?: string | null
): Promise<PersonalContextPack> {
  return invoke("build_personal_context_pack", {
    date: date ?? null,
    mode: mode ?? null,
  });
}

export async function applyPersonalMemoryPatch(
  patchJson: string,
  sourceContextId: string
): Promise<PersonalMemoryPatchApplyResult> {
  return invoke("apply_personal_memory_patch", {
    patchJson,
    sourceContextId,
  });
}

export async function exportRagMemory(): Promise<string> {
  return invoke("export_rag_memory", {});
}

export async function rebuildRagMemoryFiles(): Promise<string> {
  return invoke("rebuild_rag_memory_files", {});
}

export async function generateTarotInsight(date: string): Promise<InsightReport> {
  return invoke<InsightReport>("generate_tarot_insight", { date });
}

export async function generatePeriodReport(
  periodType: InsightPeriodType,
  anchorDate: string
): Promise<InsightReport> {
  return invoke("generate_period_report", { periodType, anchorDate });
}

export async function listInsightReports(
  reportKind?: InsightReportKind | null,
  periodType?: InsightPeriodType | null,
  limit?: number | null
): Promise<InsightReport[]> {
  return invoke("list_insight_reports", {
    reportKind: reportKind ?? null,
    periodType: periodType ?? null,
    limit: limit ?? null,
  });
}

export async function getInsightContextSnapshot(snapshotId: number): Promise<InsightContextSnapshot> {
  return invoke("get_insight_context_snapshot", { snapshotId });
}

export async function getCalendarInsightHistory(date: string): Promise<CalendarInsightHistory> {
  return invoke("get_calendar_insight_history", { date });
}

export async function deleteInsightReport(reportId: number): Promise<void> {
  return invoke("delete_insight_report", { reportId });
}

export async function getBondPeople(): Promise<BondPerson[]> {
  return invoke("get_bond_people", {});
}

export async function saveBondPerson(payload: {
  personId?: number | null;
  name: string;
  relationLabel?: string | null;
  score: number;
  note?: string | null;
}): Promise<BondPerson> {
  return invoke("save_bond_person", {
    personId: payload.personId ?? null,
    name: payload.name,
    relationLabel: payload.relationLabel ?? null,
    score: payload.score,
    note: payload.note ?? null,
  });
}

export async function deleteBondPerson(personId: number): Promise<void> {
  return invoke("delete_bond_person", { personId });
}

export async function getBondEntries(personId: number): Promise<BondEntry[]> {
  return invoke("get_bond_entries", { personId });
}

export async function saveBondEntry(
  personId: number,
  entryDate: string,
  title: string,
  content: string
): Promise<BondEntry> {
  return invoke("save_bond_entry", {
    personId,
    entryDate,
    title: title || null,
    content,
  });
}

export async function deleteBondEntry(entryId: number): Promise<void> {
  return invoke("delete_bond_entry", { entryId });
}

export async function getDailyJournalByDate(entryDate: string): Promise<DailyJournal | null> {
  return invoke("get_daily_journal_by_date", { entryDate });
}

export async function getRecentDailyJournals(limit = 20): Promise<DailyJournal[]> {
  return invoke("get_recent_daily_journals", { limit });
}

export async function saveDailyJournal(
  entryDate: string,
  title: string,
  content: string,
  mood: string
): Promise<DailyJournal> {
  return invoke("save_daily_journal", {
    entryDate,
    title: title || null,
    content,
    mood: mood || null,
  });
}

export async function deleteDailyJournal(journalId: number): Promise<void> {
  return invoke("delete_daily_journal", { journalId });
}

export async function getWeekPlan(anchorDate?: string): Promise<PlanSnapshot> {
  return invoke("get_week_plan", { anchorDate });
}

export async function getMonthPlan(anchorDate?: string): Promise<PlanSnapshot> {
  return invoke("get_month_plan", { anchorDate });
}

export async function getCalendarPlanHistory(date: string): Promise<CalendarPlanHistory> {
  return invoke("get_calendar_plan_history", { date });
}

export async function savePlanCycle(
  periodType: PlanPeriodType,
  anchorDate: string,
  title: string,
  summary: string
): Promise<PlanSnapshot> {
  return invoke("save_plan_cycle", { periodType, anchorDate, title, summary });
}

export async function savePlanItem(payload: {
  cycleId: number;
  itemId?: number | null;
  title: string;
  description: string;
  dimensionKey?: string | null;
  progressPercent?: number | null;
  aiComment?: string | null;
  sortOrder?: number | null;
  isCompleted: boolean;
}): Promise<PlanSnapshot> {
  return invoke("save_plan_item", {
    cycleId: payload.cycleId,
    itemId: payload.itemId ?? null,
    title: payload.title,
    description: payload.description,
    dimensionKey: payload.dimensionKey ?? null,
    progressPercent: payload.progressPercent ?? null,
    aiComment: payload.aiComment ?? null,
    sortOrder: payload.sortOrder ?? null,
    isCompleted: payload.isCompleted,
  });
}

export async function deletePlanItem(cycleId: number, itemId: number): Promise<PlanSnapshot> {
  return invoke("delete_plan_item", { cycleId, itemId });
}

export async function refreshPlanProgress(
  periodType: PlanPeriodType,
  anchorDate: string
): Promise<PlanAiOutcome> {
  return invoke("refresh_plan_progress", { periodType, anchorDate });
}

export async function getPlanAiOutcome(sessionId: number): Promise<PlanAiOutcome> {
  return invoke("get_plan_ai_outcome", { sessionId });
}

export async function getLatestPlanAiOutcome(
  periodType: PlanPeriodType,
  anchorDate: string
): Promise<PlanAiOutcome | null> {
  return invoke("get_latest_plan_ai_outcome", { periodType, anchorDate });
}

export async function submitPlanAiAnswers(
  sessionId: number,
  answers: string[]
): Promise<PlanAiOutcome> {
  return invoke("submit_plan_ai_answers", { sessionId, answers });
}

export async function applyPlanAiUpdate(sessionId: number): Promise<PlanSnapshot> {
  return invoke("apply_plan_ai_update", { sessionId });
}

export async function runGlobalCloseout(
  date: string,
  scope: "day" | "week" | "month" | "all" = "all"
): Promise<GlobalCloseoutResult> {
  return invoke("run_global_closeout", { date, scope });
}

export async function getLatestCloseoutRun(date: string): Promise<GlobalCloseoutResult | null> {
  return invoke("get_latest_closeout_run", { date });
}

export async function saveGeneralSettings(
  settings: GeneralSettingsPayload
): Promise<void> {
  return invoke("save_general_settings", {
    scoringEngine: settings.scoring_engine,
    deepseekBaseUrl: settings.deepseek_base_url,
    deepseekFlashModel: settings.deepseek_flash_model,
    deepseekProModel: settings.deepseek_pro_model,
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
