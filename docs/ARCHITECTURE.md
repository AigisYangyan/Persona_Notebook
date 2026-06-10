# PGRN Architecture

## 概览

Personal Growth RPG Notebook 是一个本地优先的桌面应用。  
用户记录 `title + minutes + difficulty_star`，系统生成成长预览，确认后写入成长账本，再由 Dashboard、Calendar、Ledger 消费。

当前实现遵循两条核心约束：

- API 只提供评分建议，不直接改数据库
- 所有成长值都必须落到 `stat_ledger`，并可回滚
- 评分模式固定为 `Rules + API feedback`

## 当前模块

### 前端

- `src/pages/`
  - `Dashboard.vue`：五维总览、等级、连续记录、徽章
  - `Today.vue`：任务录入、评分预览、确认写入
  - `Calendar.vue`：按月查看有记录/已评分状态
  - `Ledger.vue`：账本浏览与回滚
  - `Settings.vue`：评分引擎、API 配置、API Key、导入导出
- `src/stores/`
  - `recordStore.ts`：今日记录与待确认评分预览
  - `statStore.ts`：五维统计与等级数据
  - `settingStore.ts`：设置加载、普通配置保存、API Key 管理
- `src/api/adapter/`
  - `openaiAdapter.ts`：LLM 评分适配器
  - `parseScoreResponse.ts`：API 返回清洗与 Schema 解析
- `src/features/scoring/requestBuilder.ts`
  - 组装 `Rules` 缓存和 API 请求
- `src/features/scoring/preview.ts`
  - `Rules` 预览摘要和 API 评分结果归一化
  - 单日上限裁剪

### Rust / Tauri

- `src-tauri/src/commands/`
  - `record.rs`：任务 CRUD
  - `score.rs`：Rules 缓存预览、确认写入
  - `api_proxy.rs`：LLM 调用与 `api_runs` 记录
  - `stat.rs`：总分、账本、连续记录、月历概览、回滚
  - `settings.rs`：普通设置和 API Key 管理
  - `import_export.rs`：CSV/JSON 导入导出
- `src-tauri/src/db/repositories/`
  - `record_repo.rs`
  - `ledger_repo.rs`
  - `daily_review_repo.rs`
  - `api_run_repo.rs`
  - `dimension_repo.rs`
  - `rule_repo.rs`
  - `setting_repo.rs`

## 数据流

1. 用户在 Today 录入任务，写入 `records`
2. Rust 规则引擎生成 `Rules` 缓存预览
3. 前端把 `rule_hints` 和原始记录一起发送给 API
4. API 返回受约束的最终预览
5. 前端展示预览，用户确认
6. Rust 侧统一写入 `stat_ledger`
7. 同步重算 `daily_reviews`
8. Dashboard / Calendar / Ledger 刷新展示

## 数据模型

### `records`

保存原始任务输入：

- `date`
- `title`
- `minutes`
- `difficulty_star`

### `stat_ledger`

保存每一次有效成长变化：

- `date`
- `record_id`
- `dimension_key`
- `change_value`
- `source_title`
- `reason`
- `confidence`
- `engine`
- `is_rollback`

### `daily_reviews`

按天保存聚合结果：

- 五维总分列
- `summary_text`
- `is_analyzed`

### `api_runs`

记录 API 请求链路：

- `request_json`
- `response_json`
- `status`
- `error_message`
- `latency_ms`
- `engine_name`

## 已实现的安全边界

- API Key 由 Rust 侧持久化和读取
- 前端只知道 `api_key_configured`，不回显明文
- API 返回先经过 Zod + 业务校验，再进入预览流程

## 当前发布方式

推荐使用：

- `src-tauri\target\release\pgrn.exe`
- NSIS / MSI 安装包

不再把根目录手工复制的 `pgrn.exe` 作为标准发布路径。

## 后续阶段

当前仓库已经覆盖 Kimi 蓝图中的核心闭环，但以下方向仍可继续扩展：

- 更完整的 `daily_reviews` 文案生成
- 更强的导入规则与 Clocklog 对接
- 更细的日历筛选和已评分重跑能力
- Stronghold 或系统级安全存储
- 更完整的 E2E 测试与覆盖率门槛
