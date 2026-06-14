# Personal Growth RPG Notebook (个人成长 RPG 笔记本)

这是一个本地优先的桌面笔记本，旨在帮助用户进行结构化自我追踪、每日复盘与五维成长评分。

---

## 🌟 项目概述

**Personal Growth RPG Notebook (`PGRN`)** 是一款基于 Tauri 开发的桌面端应用程序。系统围绕以下五维成长模型构建：

*   **学识 (`knowledge`)** / 学识积累与认知提升
*   **觉悟 (`willpower`)** / 自律、反思与习惯养成
*   **表达 (`expression`)** / 写作、沟通与观点输出
*   **体魄 (`physique`)** / 运动、健康与精力管理
*   **羁绊 (`bond`)** / 社交联系、亲情与人际互动

---

## ⚙️ 核心设计与评分机制

与普通的无约束 AI 评分或纯本地静态评分不同，PGRN 采用创新的**混合评分管线**：

1.  **本地规则初步评估**：本地规则引擎（`Rules`）根据录入任务生成初步评分缓存与评分提示（`rule_hints`）。
2.  **API 约束评分预览**：评分引擎将本地提示作为结构化约束发送给大语言模型 API，生成受约束的评分与评语预览。
3.  **确认写入机制**：所有评分和评语仅作为“预览”展示，必须经过用户手动“确认”后，才会正式写入本地数据库账本（`stat_ledger`）。
4.  **安全可靠**：API 评分不直接修改数据库，支持历史日期的回滚与重新计算。

---

## 🗺️ 功能模块与页面导航

| 页面 / 模块 | 用途与核心功能 |
| :--- | :--- |
| **仪表盘 (Dashboard)** | 展示用户等级、经验值（EXP）、五维雷达图、连续记录天数及已解锁的成长徽章。 |
| **今日复盘 (Today)** | 记录任务（包含标题、时长、困难星级），自动生成本地与 API 评分预览，用户确认后写入账本。 |
| **日历视图 (Calendar)** | 按月份浏览有记录、已评分或未评分的天数，方便历史轨迹追踪。 |
| **成长账本 (Ledger)** | 详细审计每一次成长值的变化，并支持对单条或某天的记录进行回滚与重算。 |
| **系统设置 (Settings)** | 配置本地/在线评分引擎、API Key 管理（仅 Rust 侧明文读取）、任务数据 CSV/JSON 导入与导出。 |

---

## 🔒 隐私与安全边界

*   **本地优先**：所有任务数据、成长账本和评语均存储在本地 SQLite 数据库中。
*   **安全存储**：API Key 保存在本地，且仅由 Rust 侧读取并向大模型 API 发起请求，前端仅能获取 `api_key_configured: boolean`，从根本上杜绝了 Key 在前端被明文泄露或劫持的风险。
*   **格式校验**：API 返回的数据进入预览流程前，必须通过严格的 Zod Schema 与业务逻辑双重校验。

---

## 🚀 统一启动与开发指南

> [!IMPORTANT]
> 根据项目规范，**`启动入口\`** 文件夹是本项目唯一的标准启动与构建目录。
> 根目录下的 `run-dev.bat`、`build-release.bat` 等仅用于兼容转发，请不要直接修改或将其作为主要开发入口。

### 📦 安装依赖

```bash
npm install
```

### 🛠️ 启动模式

你可以通过以下几种方式运行和构建项目：

1.  **开发调试模式**
    *   双击运行 **`启动入口\开发模式.cmd`**
    *   或者在终端运行：
        ```bash
        npm run tauri dev
        ```

2.  **打开完整版（推荐日常使用）**
    *   双击运行 **`启动入口\打开完整版.cmd`**
    *   *特点*：脚本会自动比对源码和本地已构建的 `pgrn.exe` 文件的修改时间。如果源码比 exe 更亲，会自动重新构建最新版并启动（该过程只构建 exe，不进行 installer 打包，构建速度极快）。若已经有 PGRN 进程在运行，则会直接打开，避免卡死。

3.  **手动重新构建最新桌面 EXE**
    *   双击运行 **`启动入口\构建完整版.cmd`**
    *   或者在终端运行：
        ```bash
        npm run tauri build
        ```

### 📂 构建产物

构建成功后的安装包与可执行文件输出在以下路径：
*   **免安装可执行程序**：`src-tauri\target\release\pgrn.exe`
*   **Windows NSIS 安装程序**：`src-tauri\target\release\bundle\nsis\Personal Growth RPG Notebook_X.Y.Z_x64-setup.exe`
*   **Windows MSI 安装包**：`src-tauri\target\release\bundle\msi\Personal Growth RPG Notebook_X.Y.Z_x64_en-US.msi`

---

## 📝 常用测试命令

```bash
# 运行前端与 TypeScript 测试
npm test

# 验证前端构建
npm run build

# 运行 Rust 单元测试
cargo test --manifest-path src-tauri/Cargo.toml

# 完整打包测试
npm run tauri build
```

---

## 💾 数据存储位置

默认的本地 SQLite 数据库文件保存在：

```text
%LOCALAPPDATA%\com.pgrn.app\data.db
```

*注意：为兼容旧版本的用户数据，Tauri 的应用标识符固定为 `com.pgrn.app`，请勿随意更改。*

---

## 🔖 发布与版本策略

*   遵循 **语义化版本 (Semantic Versioning)** 标准。
*   Git 标签格式为 `vX.Y.Z`。
*   版本更新历史记录在 `CHANGELOG.md` 中。
*   Github Releases 页面提供最新打包好的 Windows 安装包和免安装程序。
*   有关更详细的发布规范，请参阅：
    *   [docs/RELEASING.md](docs/RELEASING.md)
    *   [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

---

## ⚖️ 开源协议

本项目采用 **Apache-2.0** 协议开源。详细信息请参阅 [LICENSE](LICENSE)。
