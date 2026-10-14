# Capsule — A Tiny Desktop Companion for Tasks & Healthy Breaks

[简体中文](#简体中文) · [English](#english)

![Capsule 胶囊、任务列表、健康提醒及历史记录的概念示意图 / Concept illustration of Capsule's pill, tasks, reminders, and history](docs/capsule-overview.svg)

*项目界面概念示意图 / Concept illustration of the desktop interface.*

## 简体中文

### Capsule 是什么

Capsule 是一款常驻屏幕边缘的轻量级桌面工具，将每日待办与护眼、喝水、起身活动提醒放在同一个悬浮胶囊中。数据保存在本机，无需账号、服务器或云同步，也不包含遥测与行为追踪。

### 功能

- 无边框、透明背景、置顶的悬浮胶囊，没有传统主窗口；收起时约为 110 × 32 逻辑像素。
- 点击展开，点击外部或收起按钮即可收起；支持拖动胶囊及展开面板的标题区域。
- 在设置中选择原色、炭黑、浅灰、柔蓝或柔黄五种清淡配色，选择会保存在本机。
- 添加、完成、取消完成、删除和排序当天任务。任务保存在 SQLite 中，可在历史记录中查看。
- 手动将未完成任务顺延到次日，或从过去的日期逐条带入今天（也可一键全部带入）。新日期的任务副本会标记原始创建日期，旧记录仍保留。
- Rust 后台分别调度护眼、喝水和起身提醒。胶囊收起时计时仍继续；完成和稍后提醒会记录为事件。
- 护眼与起身相差不超过 10 分钟时，在较晚的到期时间合并成一张卡片；喝水仍独立提醒。合并卡片可一次完成两项，或按两者较长的稍后时长一起推迟。
- 提醒到期时，在胶囊所在屏幕中央显示小卡片并播放 Windows 系统提示音。卡片 10 秒后收起，并可在 90 秒后再次提示；可在设置中关闭声音和重复提示，免打扰时段也会抑制它们。
- 修改提醒间隔、稍后提醒时长和免打扰时段，无需重启应用。睡眠唤醒或免打扰结束后，不会集中补发错过的提醒。
- 查看今天、最近 7 天或最近 30 天的任务与健康提醒完成情况，以及每日详情和完成率。

### 技术栈

Tauri 2、React 19、TypeScript、Vite、Rust、rusqlite、SQLite、chrono 和 Zustand。打包后的应用不依赖网页后端、账号或云数据库。

### 开发运行

在 Windows 上，先安装 [Rust（MSVC 工具链）](https://v2.tauri.app/start/prerequisites/)、Microsoft C++ Build Tools（勾选 **Desktop development with C++**），并确保 WebView2 可用。然后运行：

```powershell
npm install
npm run tauri dev
```

如果普通终端找不到链接器，请在 Visual Studio Developer PowerShell 或 Developer Command Prompt 中运行上述命令。

### 构建与测试

`npm run build` 检查 TypeScript 并构建前端。安装原生编译环境后，运行 `npm run tauri build` 构建桌面安装包。在 `src-tauri/` 目录中运行 `cargo test` 可执行 Rust 单元测试。

### 项目结构

```text
src/
  components/Capsule/     悬浮胶囊的界面状态与交互
  components/Todo/        行内添加任务与任务列表
  components/Reminder/    倒计时与提醒卡片
  components/Settings/    提醒和免打扰设置
  components/History/     历史日期列表与详情
  stores/                 Zustand 临时界面状态与后台数据快照
  lib/                    类型化 Tauri 命令及窗口交互
  types/                  前端数据类型
src-tauri/
  src/database.rs         SQLite 初始化与任务访问
  src/reminder.rs         提醒规则与事件
  src/scheduler.rs        提醒计时、免打扰与睡眠唤醒处理
  src/attention.rs        提醒卡片呈现与可选提示音
  src/history.rs          根据原始数据计算每日统计
  src/commands.rs         Tauri 命令
  src/window.rs           窗口位置与尺寸
  src/models.rs           序列化数据结构
  src/lib.rs              应用初始化
```

### 架构

Rust 负责原生窗口、SQLite、提醒调度及屏幕中央提醒卡片的触发时间。调度器保存真实的下次提醒时间，并在提醒触发、设置变化或日期切换时发送 Tauri 事件。React 负责界面呈现，根据 Rust 提供的时间戳显示倒计时；前端每秒更新只影响显示，不决定提醒何时触发或再次提示。Zustand 仅保存临时界面状态和后台数据快照，持久化以数据库为准。

### 数据库

数据库文件 `capsule.sqlite3` 位于 Tauri 标准应用数据目录。版本化迁移创建 `tasks`、`reminder_rules`、`reminder_events` 和键值表 `app_settings`。任务以本地 `YYYY-MM-DD` 日期归属；手动顺延会为新日期创建任务副本，同时保留原日期的未完成记录和最初创建时间。提醒事件保存计划与操作时间、稍后提醒截止时间，以及 `triggered`、`completed`、`snoozed`、`skipped` 状态。历史统计直接由任务和提醒事件计算，不建立冗余统计表。

### 路线图

- **v0.1：** 悬浮胶囊、待办、手动顺延、健康提醒、护眼与起身合并、历史记录、配色主题。
- **v0.2：** 空闲检测、热力图、键盘快捷键。
- **v0.3：** 全屏检测、顺延体验完善。

窗口位置保存、边缘吸附和开机启动仍属于桌面体验完善工作。空闲检测和热力图尚未包含在 v0.1 中。

## English

### What is Capsule

Capsule is a small, local desktop companion that lives near the edge of the screen. It combines today's tasks with gentle eye, water, and movement reminders. Data stays on the device; no account, server, telemetry, or cloud sync is used.

### Features

- One transparent, borderless, always-on-top capsule window; no traditional main window.
- A compact 110 × 32 logical-pixel collapsed view and small expanded panels.
- Five calm capsule color themes in Settings: original sage, charcoal, light gray, soft blue, and soft yellow. The choice is saved locally and restored on launch.
- Click to expand, click outside or use the collapse button to tuck it away.
- Drag the capsule or its expanded header. The initial location is near the right edge of the primary display.
- Add, complete, uncomplete, delete, and reorder today's tasks. Tasks are stored in SQLite and remain available in history.
- Manually continue an unfinished task to tomorrow, or bring individual unfinished tasks from earlier days into today (with a Bring all shortcut). Each new daily copy shows the original creation date; earlier daily records stay in history.
- Rust schedules independent Eye, Water, and Stand reminders even while the capsule is collapsed. Done and Snooze are recorded as events.
- Eye and Stand reminders due within ten minutes appear together at the later deadline; Water stays independent. A paired card can complete both or snooze them together for the longer configured delay.
- When a reminder is due, a compact card appears at the center of the capsule's monitor with a Windows system chime. It retracts after ten seconds and can appear once more after ninety seconds. Sound and repeat can be disabled in Settings; quiet hours suppress both.
- Change intervals, snooze lengths, and quiet hours without restarting. Missed reminders are not replayed after sleep or quiet hours.
- Browse today, seven days, or thirty days of task and reminder completion, with daily details and rates.

### Tech stack

Tauri 2, React 19, TypeScript, Vite, Rust, rusqlite, SQLite, chrono, and Zustand. The packaged app has no web backend, account, cloud database, telemetry, or tracking.

### Development

On Windows, install [Rust with the MSVC toolchain](https://v2.tauri.app/start/prerequisites/) and Microsoft C++ Build Tools with **Desktop development with C++**. WebView2 is also required. Then run:

```powershell
npm install
npm run tauri dev
```

If the linker is not found in a normal terminal, run these commands from a Visual Studio Developer PowerShell or Developer Command Prompt.

### Build

`npm run build` checks TypeScript and builds the frontend. `npm run tauri build` builds a desktop bundle after the native prerequisites are installed.
Run Rust unit tests with `cargo test` from `src-tauri/`.

### Project structure

```text
src/
  components/Capsule/     Floating window states and interactions
  components/Todo/        Inline task entry and list
  components/Reminder/    Countdown and alert
  components/Settings/    Reminder and quiet-hours controls
  components/History/     Date list and detail
  stores/                 Zustand UI state and backend snapshots
  lib/                    Typed Tauri commands and window bridge
  types/                  Frontend data contracts
src-tauri/
  src/database.rs         SQLite initialization and task access
  src/reminder.rs         Reminder rules and events
  src/scheduler.rs        Rust reminder clock, quiet hours, wake handling
  src/attention.rs        Reminder presentation and optional system chime
  src/history.rs          Daily statistics derived from source tables
  src/commands.rs         Typed Tauri commands
  src/window.rs           Window position and sizing
  src/models.rs           Serialized data structures
  src/lib.rs              Application setup
```

### Architecture

Rust owns the native window, SQLite, reminder schedule, and the timing of the center-screen reminder card. The scheduler keeps the true next-trigger times and emits Tauri events when reminders occur, settings change, or the date rolls over. React renders the UI and displays a countdown from Rust timestamps; its one-second interval never decides when a reminder fires or repeats. Zustand holds temporary UI state and backend snapshots, not persistent storage.

### Database schema

The database is `capsule.sqlite3` in Tauri's standard app data directory. Versioned migrations create four tables: `tasks`, `reminder_rules`, `reminder_events`, and key-value `app_settings`. Tasks store their local `YYYY-MM-DD` date. A manual carry creates a new daily task while preserving the earlier row, its incomplete history, and the original creation timestamp. Reminder events store scheduled and action timestamps, a persisted snooze deadline, and `triggered`, `completed`, `snoozed`, or `skipped` status. History is calculated from tasks and reminder events; there is no redundant statistics table.

### Roadmap

- **v0.1:** Capsule, Todo, manual task carry, Reminder, Eye/Stand merge, History, color themes.
- **v0.2:** Idle detection, heatmap, keyboard shortcut.
- **v0.3:** Fullscreen detection and carry-over polish.

Window position persistence, edge snapping, and launch at startup remain desktop-polish work. Idle detection and the heatmap remain outside v0.1.
