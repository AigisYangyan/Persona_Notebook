use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS stat_dimensions (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            key         TEXT NOT NULL UNIQUE,
            name        TEXT NOT NULL,
            description TEXT,
            daily_cap   INTEGER NOT NULL DEFAULT 10,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS records (
            id                        INTEGER PRIMARY KEY AUTOINCREMENT,
            date                      TEXT NOT NULL,
            title                     TEXT NOT NULL,
            minutes                   INTEGER NOT NULL DEFAULT 0,
            difficulty_star           INTEGER NOT NULL DEFAULT 0 CHECK (difficulty_star BETWEEN 0 AND 3),
            parent_id                 INTEGER,
            is_completed              INTEGER NOT NULL DEFAULT 0,
            completed_at              TEXT,
            elapsed_seconds           INTEGER NOT NULL DEFAULT 0,
            timer_mode                TEXT NOT NULL DEFAULT 'stopwatch',
            countdown_target_seconds  INTEGER,
            timer_started_at          TEXT,
            note                      TEXT,
            clocklog_id               TEXT,
            created_at                TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at                TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_records_date ON records(date);
        CREATE TABLE IF NOT EXISTS record_timer_sessions (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            record_id        INTEGER NOT NULL,
            started_at       TEXT NOT NULL,
            ended_at         TEXT,
            duration_seconds INTEGER,
            created_at       TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_record_timer_sessions_record ON record_timer_sessions(record_id);
        CREATE INDEX IF NOT EXISTS idx_record_timer_sessions_open ON record_timer_sessions(record_id, ended_at);

        CREATE TABLE IF NOT EXISTS stat_ledger (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            date            TEXT NOT NULL,
            record_id       INTEGER,
            dimension_key   TEXT NOT NULL,
            change_value    INTEGER NOT NULL,
            source_title    TEXT,
            reason          TEXT,
            confidence      REAL,
            engine          TEXT NOT NULL DEFAULT 'rules_api',
            is_rollback     INTEGER NOT NULL DEFAULT 0,
            rollback_ref    INTEGER,
            created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE SET NULL,
            FOREIGN KEY (dimension_key) REFERENCES stat_dimensions(key)
        );
        CREATE INDEX IF NOT EXISTS idx_ledger_date ON stat_ledger(date);
        CREATE INDEX IF NOT EXISTS idx_ledger_record ON stat_ledger(record_id);
        CREATE INDEX IF NOT EXISTS idx_ledger_dimension ON stat_ledger(dimension_key);

        CREATE TABLE IF NOT EXISTS daily_reviews (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            date             TEXT NOT NULL UNIQUE,
            total_knowledge  INTEGER NOT NULL DEFAULT 0,
            total_willpower  INTEGER NOT NULL DEFAULT 0,
            total_expression INTEGER NOT NULL DEFAULT 0,
            total_physique   INTEGER NOT NULL DEFAULT 0,
            total_bond       INTEGER NOT NULL DEFAULT 0,
            summary_text     TEXT,
            is_analyzed      INTEGER NOT NULL DEFAULT 0,
            created_at       TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at       TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS api_runs (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            date          TEXT NOT NULL,
            request_json  TEXT NOT NULL,
            response_json TEXT,
            status        TEXT NOT NULL,
            error_message TEXT,
            latency_ms    INTEGER,
            engine_name   TEXT NOT NULL DEFAULT 'openai',
            task_kind     TEXT NOT NULL DEFAULT 'unknown',
            model_tier    TEXT NOT NULL DEFAULT 'legacy',
            fallback_used INTEGER NOT NULL DEFAULT 0,
            prompt_tokens INTEGER,
            completion_tokens INTEGER,
            prompt_cache_hit_tokens INTEGER,
            prompt_cache_miss_tokens INTEGER,
            finish_reason TEXT,
            created_at    TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS plan_cycles (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            period_type     TEXT NOT NULL,
            start_date      TEXT NOT NULL,
            end_date        TEXT NOT NULL,
            title           TEXT NOT NULL DEFAULT '',
            summary         TEXT NOT NULL DEFAULT '',
            ai_summary      TEXT NOT NULL DEFAULT '',
            last_ai_run_at  TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE(period_type, start_date, end_date)
        );
        CREATE INDEX IF NOT EXISTS idx_plan_cycles_period ON plan_cycles(period_type, start_date, end_date);

        CREATE TABLE IF NOT EXISTS plan_items (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            cycle_id          INTEGER NOT NULL,
            title             TEXT NOT NULL,
            description       TEXT NOT NULL DEFAULT '',
            dimension_key     TEXT,
            progress_percent  INTEGER NOT NULL DEFAULT 0,
            ai_comment        TEXT NOT NULL DEFAULT '',
            sort_order        INTEGER NOT NULL DEFAULT 0,
            is_completed      INTEGER NOT NULL DEFAULT 0,
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (cycle_id) REFERENCES plan_cycles(id) ON DELETE CASCADE,
            FOREIGN KEY (dimension_key) REFERENCES stat_dimensions(key) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_plan_items_cycle ON plan_items(cycle_id, sort_order);

        CREATE TABLE IF NOT EXISTS plan_ai_sessions (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            cycle_id          INTEGER NOT NULL,
            status            TEXT NOT NULL DEFAULT 'pending',
            request_payload   TEXT NOT NULL DEFAULT '',
            response_payload  TEXT,
            questions_json    TEXT NOT NULL DEFAULT '[]',
            answers_json      TEXT NOT NULL DEFAULT '[]',
            proposal_json     TEXT,
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (cycle_id) REFERENCES plan_cycles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_plan_ai_sessions_cycle ON plan_ai_sessions(cycle_id, created_at);

        CREATE TABLE IF NOT EXISTS bond_people (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            name              TEXT NOT NULL,
            relation_label    TEXT NOT NULL DEFAULT '',
            score             INTEGER NOT NULL DEFAULT 5 CHECK (score BETWEEN 0 AND 10),
            note              TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_bond_people_updated ON bond_people(updated_at DESC);

        CREATE TABLE IF NOT EXISTS bond_entries (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            person_id         INTEGER NOT NULL,
            entry_date        TEXT NOT NULL,
            title             TEXT NOT NULL DEFAULT '',
            content           TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE(person_id, entry_date),
            FOREIGN KEY (person_id) REFERENCES bond_people(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_bond_entries_person ON bond_entries(person_id, entry_date DESC);

        CREATE TABLE IF NOT EXISTS daily_journals (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_date        TEXT NOT NULL UNIQUE,
            title             TEXT NOT NULL DEFAULT '',
            content           TEXT NOT NULL DEFAULT '',
            mood              TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_daily_journals_date ON daily_journals(entry_date DESC);

        CREATE TABLE IF NOT EXISTS personal_profile (
            id                INTEGER PRIMARY KEY CHECK (id = 1),
            birthday          TEXT NOT NULL DEFAULT '',
            personality       TEXT NOT NULL DEFAULT '',
            experiences       TEXT NOT NULL DEFAULT '',
            personal_notes    TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS personal_memory_items (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_type       TEXT NOT NULL,
            title             TEXT NOT NULL,
            summary           TEXT NOT NULL DEFAULT '',
            detail            TEXT NOT NULL DEFAULT '',
            tags_json         TEXT NOT NULL DEFAULT '[]',
            importance        INTEGER NOT NULL DEFAULT 0,
            confidence        REAL NOT NULL DEFAULT 0,
            first_seen_date   TEXT,
            last_seen_date    TEXT,
            status            TEXT NOT NULL DEFAULT 'active',
            supersedes_id     INTEGER,
            created_by        TEXT NOT NULL DEFAULT 'ai',
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (supersedes_id) REFERENCES personal_memory_items(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_personal_memory_items_status ON personal_memory_items(status, importance DESC);
        CREATE INDEX IF NOT EXISTS idx_personal_memory_items_last_seen ON personal_memory_items(last_seen_date DESC, importance DESC);

        CREATE TABLE IF NOT EXISTS personal_memory_sources (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_id         INTEGER NOT NULL,
            source_type       TEXT NOT NULL,
            source_id         TEXT NOT NULL DEFAULT '',
            source_date       TEXT,
            evidence_id       TEXT NOT NULL,
            excerpt           TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (memory_id) REFERENCES personal_memory_items(id) ON DELETE CASCADE,
            UNIQUE (memory_id, evidence_id)
        );
        CREATE INDEX IF NOT EXISTS idx_personal_memory_sources_memory ON personal_memory_sources(memory_id);
        CREATE INDEX IF NOT EXISTS idx_personal_memory_sources_evidence ON personal_memory_sources(evidence_id);

        CREATE TABLE IF NOT EXISTS personal_memory_events (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_id         INTEGER,
            event_type        TEXT NOT NULL,
            payload_json      TEXT NOT NULL DEFAULT '{}',
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (memory_id) REFERENCES personal_memory_items(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_personal_memory_events_memory ON personal_memory_events(memory_id, created_at DESC);

        CREATE TABLE IF NOT EXISTS personal_memory_patch_runs (
            id                 INTEGER PRIMARY KEY AUTOINCREMENT,
            source_context_id  TEXT NOT NULL DEFAULT '',
            patch_json         TEXT NOT NULL,
            validation_status  TEXT NOT NULL DEFAULT 'pending',
            apply_status       TEXT NOT NULL DEFAULT 'pending',
            rejected_reason    TEXT,
            applied_operations INTEGER NOT NULL DEFAULT 0,
            rejected_operations INTEGER NOT NULL DEFAULT 0,
            created_at         TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at         TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_personal_memory_patch_runs_created ON personal_memory_patch_runs(created_at DESC);

        CREATE TABLE IF NOT EXISTS insight_context_snapshots (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            report_kind       TEXT NOT NULL,
            period_type       TEXT NOT NULL,
            start_date        TEXT NOT NULL,
            end_date          TEXT NOT NULL,
            context_json      TEXT NOT NULL,
            created_at        TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_insight_context_period ON insight_context_snapshots(report_kind, period_type, start_date, end_date);

        CREATE TABLE IF NOT EXISTS insight_reports (
            id                         INTEGER PRIMARY KEY AUTOINCREMENT,
            report_kind                TEXT NOT NULL,
            period_type                TEXT NOT NULL,
            start_date                 TEXT NOT NULL,
            end_date                   TEXT NOT NULL,
            title                      TEXT NOT NULL DEFAULT '',
            summary                    TEXT NOT NULL DEFAULT '',
            content_json               TEXT NOT NULL DEFAULT '{}',
            raw_response               TEXT NOT NULL DEFAULT '',
            context_snapshot_id        INTEGER,
            status                     TEXT NOT NULL DEFAULT 'success',
            error_message              TEXT,
            memory_patch_json          TEXT,
            memory_patch_apply_status  TEXT,
            memory_patch_apply_message TEXT,
            created_at                 TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (context_snapshot_id) REFERENCES insight_context_snapshots(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_insight_reports_period ON insight_reports(report_kind, period_type, start_date, end_date, created_at DESC);

        CREATE TABLE IF NOT EXISTS category_rules (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            keywords      TEXT NOT NULL,
            primary_dim   TEXT NOT NULL,
            secondary_dim TEXT,
            priority      INTEGER NOT NULL DEFAULT 0,
            is_regex      INTEGER NOT NULL DEFAULT 0,
            is_active     INTEGER NOT NULL DEFAULT 1,
            created_at    TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS settings (
            key         TEXT PRIMARY KEY,
            value       TEXT NOT NULL,
            value_type  TEXT NOT NULL DEFAULT 'string',
            description TEXT,
            updated_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        ",
    )?;

    ensure_record_columns(conn)?;
    ensure_api_run_columns(conn)?;
    ensure_daily_memory_digest(conn)?;
    init_default_data(conn)?;
    Ok(())
}

fn ensure_daily_memory_digest(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS daily_memory_digest (
            date         TEXT NOT NULL PRIMARY KEY,
            profile_json TEXT NOT NULL,
            digest_json  TEXT NOT NULL,
            created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        DELETE FROM daily_memory_digest
        WHERE date < date('now', 'localtime', '-7 days');
        ",
    )?;
    Ok(())
}

fn ensure_record_columns(conn: &Connection) -> Result<()> {
    ensure_column(conn, "records", "parent_id", "parent_id INTEGER")?;
    ensure_column(
        conn,
        "records",
        "is_completed",
        "is_completed INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(conn, "records", "completed_at", "completed_at TEXT")?;
    ensure_column(
        conn,
        "records",
        "elapsed_seconds",
        "elapsed_seconds INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "records",
        "timer_mode",
        "timer_mode TEXT NOT NULL DEFAULT 'stopwatch'",
    )?;
    ensure_column(
        conn,
        "records",
        "countdown_target_seconds",
        "countdown_target_seconds INTEGER",
    )?;
    ensure_column(conn, "records", "timer_started_at", "timer_started_at TEXT")?;

    conn.execute_batch(
        "
        UPDATE records
        SET elapsed_seconds = CASE
            WHEN elapsed_seconds = 0 AND minutes > 0 THEN minutes * 60
            ELSE elapsed_seconds
        END;
        UPDATE records
        SET timer_mode = 'stopwatch'
        WHERE timer_mode IS NULL OR timer_mode = '';
        CREATE INDEX IF NOT EXISTS idx_records_parent ON records(parent_id);
        CREATE INDEX IF NOT EXISTS idx_records_timer_started ON records(timer_started_at);
        ",
    )?;
    Ok(())
}

fn ensure_api_run_columns(conn: &Connection) -> Result<()> {
    ensure_column(
        conn,
        "api_runs",
        "task_kind",
        "task_kind TEXT NOT NULL DEFAULT 'unknown'",
    )?;
    ensure_column(
        conn,
        "api_runs",
        "model_tier",
        "model_tier TEXT NOT NULL DEFAULT 'legacy'",
    )?;
    ensure_column(
        conn,
        "api_runs",
        "fallback_used",
        "fallback_used INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(conn, "api_runs", "prompt_tokens", "prompt_tokens INTEGER")?;
    ensure_column(
        conn,
        "api_runs",
        "completion_tokens",
        "completion_tokens INTEGER",
    )?;
    ensure_column(
        conn,
        "api_runs",
        "prompt_cache_hit_tokens",
        "prompt_cache_hit_tokens INTEGER",
    )?;
    ensure_column(
        conn,
        "api_runs",
        "prompt_cache_miss_tokens",
        "prompt_cache_miss_tokens INTEGER",
    )?;
    ensure_column(conn, "api_runs", "finish_reason", "finish_reason TEXT")?;
    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let existing = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if existing.iter().any(|name| name == column) {
        return Ok(());
    }

    conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {definition}"), [])?;
    Ok(())
}

fn init_default_data(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO stat_dimensions (key, name, description, daily_cap, sort_order) VALUES
        ('knowledge', '知识', '学习、知识、课程、考试、阅读、技术理解', 10, 1),
        ('willpower', '觉悟', '自律、执行、抗压、计划完成、长期坚持', 8, 2),
        ('expression', '表达', '写作、输出、沟通、摄影、剪辑、自媒体、作品呈现', 8, 3),
        ('physique', '体魄', '健身、跑步、力量训练、睡眠、饮食、健康维护', 8, 4),
        ('bond', '羁绊', '社交、合作、人际连接、团队协作、关系维护', 6, 5)",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO category_rules (keywords, primary_dim, secondary_dim, priority) VALUES
        ('作业,复习,刷题,听课,课程,考试,预习', 'knowledge', 'willpower', 100),
        ('背单词,听力,阅读理解,翻译,作文,语法', 'knowledge', 'willpower', 100),
        ('健身,跑步,力量训练,拉伸,瑜伽,游泳', 'physique', 'willpower', 100),
        ('阅读,看书,摘抄,读书笔记,文献', 'knowledge', 'expression', 90),
        ('项目,代码,开发,调试,架构,设计,编程,算法', 'knowledge', 'willpower', 90),
        ('拍摄,剪辑,自媒体,文案,博客,视频', 'expression', 'willpower', 90),
        ('沟通,合作,社交,会议,约拍,聚会,团建', 'bond', 'expression', 90),
        ('整理房间,洗衣服,生活维护,做饭,洗碗,扫地', 'willpower', 'physique', 80)",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value, value_type, description) VALUES
        ('scoring_engine', 'rules_api', 'string', '评分引擎: rules_api'),
        ('api_base_url', '', 'string', 'API 基础地址'),
        ('api_model', 'gpt-4o-mini', 'string', '使用的模型名称'),
        ('deepseek_base_url', 'https://api.deepseek.com/v1', 'string', 'DeepSeek 基础地址'),
        ('deepseek_flash_model', '', 'string', 'DeepSeek flash 模型名称'),
        ('deepseek_pro_model', '', 'string', 'DeepSeek pro 模型名称'),
        ('api_key', '', 'string', 'API Key（本地存储）'),
        ('level_formula_denominator', '50', 'number', '等级公式分母'),
        ('theme', 'dark', 'string', '界面主题')",
        [],
    )?;

    Ok(())
}
