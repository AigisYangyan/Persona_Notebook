use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- 1. 五维定义表
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

        -- 2. 原始任务记录表
        CREATE TABLE IF NOT EXISTS records (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            date            TEXT NOT NULL,
            title           TEXT NOT NULL,
            minutes         INTEGER NOT NULL DEFAULT 0,
            difficulty_star INTEGER NOT NULL DEFAULT 0 CHECK (difficulty_star BETWEEN 0 AND 3),
            note            TEXT,
            clocklog_id     TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_records_date ON records(date);

        -- 3. 成长账本
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

        -- 4. 每日汇总
        CREATE TABLE IF NOT EXISTS daily_reviews (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            date            TEXT NOT NULL UNIQUE,
            total_knowledge INTEGER NOT NULL DEFAULT 0,
            total_willpower INTEGER NOT NULL DEFAULT 0,
            total_expression INTEGER NOT NULL DEFAULT 0,
            total_physique  INTEGER NOT NULL DEFAULT 0,
            total_bond      INTEGER NOT NULL DEFAULT 0,
            summary_text    TEXT,
            is_analyzed     INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        -- 5. API 请求日志
        CREATE TABLE IF NOT EXISTS api_runs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            date            TEXT NOT NULL,
            request_json    TEXT NOT NULL,
            response_json   TEXT,
            status          TEXT NOT NULL,
            error_message   TEXT,
            latency_ms      INTEGER,
            engine_name     TEXT NOT NULL DEFAULT 'openai',
            created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        -- 6. 本地分类规则表
        CREATE TABLE IF NOT EXISTS category_rules (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            keywords        TEXT NOT NULL,
            primary_dim     TEXT NOT NULL,
            secondary_dim   TEXT,
            priority        INTEGER NOT NULL DEFAULT 0,
            is_regex        INTEGER NOT NULL DEFAULT 0,
            is_active       INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        -- 7. 用户设置表
        CREATE TABLE IF NOT EXISTS settings (
            key         TEXT PRIMARY KEY,
            value       TEXT NOT NULL,
            value_type  TEXT NOT NULL DEFAULT 'string',
            description TEXT,
            updated_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        ",
    )?;

    init_default_data(conn)?;
    Ok(())
}

fn init_default_data(conn: &Connection) -> Result<()> {
    // 五维
    conn.execute(
        "INSERT OR IGNORE INTO stat_dimensions (key, name, description, daily_cap, sort_order) VALUES
        ('knowledge', '学识', '学习、知识、课程、考试、阅读、技术理解', 10, 1),
        ('willpower', '觉悟', '自律、执行、抗压、计划完成、长期坚持', 8, 2),
        ('expression', '表达', '写作、输出、沟通、摄影、剪辑、自媒体、作品呈现', 8, 3),
        ('physique', '体魄', '健身、跑步、睡眠、饮食、健康维护', 8, 4),
        ('bond', '羁绊', '社交、合作、人际连接、团队协作、关系维护', 6, 5)",
        [],
    )?;

    // 分类规则
    conn.execute(
        "INSERT OR IGNORE INTO category_rules (keywords, primary_dim, secondary_dim, priority) VALUES
        ('作业,复习,刷题,听课,课程,考试,预习', 'knowledge', 'willpower', 100),
        ('背单词,听力,阅读理解,翻译,作文,语法', 'knowledge', 'willpower', 100),
        ('健身,跑步,力量训练,拉伸,瑜伽,游泳', 'physique', 'willpower', 100),
        ('阅读,看书,摘抄,读书笔记,文献', 'knowledge', 'expression', 90),
        ('项目,代码,开发,调试,架构,设计,编程,算法', 'knowledge', 'willpower', 90),
        ('拍摄,剪辑,自媒体,写稿,文案,博客,视频', 'expression', 'willpower', 90),
        ('沟通,合作,社交,会议,约拍,聚会,团建', 'bond', 'expression', 90),
        ('整理房间,洗衣服,生活维护,做饭,洗碗,扫地', 'willpower', 'physique', 80)",
        [],
    )?;

    // 默认设置
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value, value_type, description) VALUES
        ('scoring_engine', 'rules_api', 'string', '评分引擎: rules_api'),
        ('api_base_url', '', 'string', 'API 基础地址'),
        ('api_model', 'gpt-4o-mini', 'string', '使用的模型名称'),
        ('api_key', '', 'string', 'API Key（本地存储）'),
        ('level_formula_denominator', '50', 'number', '等级公式分母'),
        ('theme', 'dark', 'string', '界面主题')",
        [],
    )?;

    Ok(())
}
