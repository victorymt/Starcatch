# 🌟 Starcatch TUI — 终端用户界面计划

> **在终端中捕捉星光 ✨** — 用 Ratatui 为 Starcatch 打造沉浸式终端体验

---

## 1. 概述

### 1.1 目标

为 Starcatch 开发一个 **Ratatui 驱动的 TUI（终端用户界面）**，让用户在不开启 GUI 或敲击复杂 CLI 命令的情况下，在终端中完成所有捕获和管理操作。

### 1.2 定位

| 维度 | 说明 |
|------|------|
| **用户场景** | SSH 远程、tmux 会话、不喜欢开 GUI 的终端党、无 Wayland 环境 |
| **相比 CLI** | 视觉化、实时刷新、可交互浏览和操作，不需要记命令和 ID |
| **相比 GUI (egui)** | 轻量、依赖少（只需终端）、可在任何环境运行、可通过 SSH 使用 |
| **共存方式** | 与 CLI 和 Qt GUI 共享同一 SQLite 数据库，三套界面操作同一份数据 |

### 1.3 技术选型

| 组件 | 选择 | 理由 |
|------|------|------|
| **TUI 框架** | [Ratatui](https://github.com/ratatui/ratatui) v0.30+ | Rust TUI 生态最活跃的库，社区好，文档全 |
| **终端后端** | [crossterm](https://github.com/crossterm-rs/crossterm) | 跨平台，与 Ratatui 深度集成，支持 Wayland 终端 |
| **事件处理** | `crossterm::event` + 自定义事件循环 | 支持键盘事件、鼠标事件、终端 resize |
| **输入框** | `ratatui::widgets::Paragraph` + 自定义 | 简单输入不需要额外依赖，复杂编辑器再考虑 |
| **数据库** | 复用现有 `db.rs` | 同一 SQLite 数据库，零重复 |
| **数据模型** | 复用现有 `models/` | 共享 Todo/Idea/Log 类型定义 |

### 1.4 当前架构

```
┌─────────────────────────────────────────────────────────┐
│                     Starcatch 星捕                        │
├─────────────┬─────────────────┬─────────────────────────┤
│   CLI       │    GUI (egui)   │    TUI (Ratatui) ★      │
│  (clap)     │   (eframe)      │   (crossterm)           │
├─────────────┴─────────────────┴─────────────────────────┤
│             共享 SQLite 数据库 (db.rs)                    │
│             共享数据模型 (models/)                        │
├─────────────────────────────────────────────────────────┤
│                     数据存储                              │
│          ~/.local/share/starcatch/starcatch.db           │
└─────────────────────────────────────────────────────────┘
```

---

## 2. 功能需求

### 2.1 主界面布局

```
┌─────────────────────────────────────────────────────────────┐
│  ⭐ Starcatch 星捕                          📊 🔍 📤 ✕    │  ← 状态栏
├──────────────────────┬──────────────────────────────────────┤
│                      │                                      │
│  [●] 📋 Todo (12)   │    (内容区 — 根据左侧选择显示)       │
│  [ ] 💭 Idea  (5)   │                                      │
│  [ ] 📓 Log   (3)   │   ┌──────────────────────────────┐   │
│                      │   │ 待办列表 / 灵感流 / 日志    │   │
│  ─── 快捷操作 ───    │   │                              │   │
│  [>] 快速输入        │   │  🔴 ⬜ 完成项目演示          │   │
│                      │   │  🟡 ⬜ 学习 Emacs Lisp     │   │
│  ─── 筛选 ───        │   │  🟢 ⬜ 买牛奶              │   │
│  [ ] 全部            │   │                              │   │
│  [●] 待办            │   └──────────────────────────────┘   │
│  [ ] 已完成          │                                      │
│  [ ] 已归档          │                                      │
├──────────────────────┴──────────────────────────────────────┤
│  ⚡ 输入新内容... (P1 #tag due:后天)           [Enter 提交] │  ← 快速输入栏
│  Ctrl+T:Todo  Ctrl+I:Idea  Ctrl+L:Log  Tab:切换面板         │  ← 提示栏
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心视图

| 视图 | 内容 | 交互 |
|------|------|------|
| **Todo 列表** | 按优先级排序的待办列表，显示图标、标题、标签、截止日期 | Enter 勾选/取消，d 删除，a 归档，e 编辑，/ 筛选 |
| **Idea 列表** | 时间倒序的灵感流，显示标题、来源、标签 | Enter 查看详情，d 删除，e 编辑 |
| **Log 列表** | 时间倒序的日志，显示内容、心情、标签 | Enter 查看详情，d 删除，e 编辑 |
| **快速输入栏** | 底部固定输入框，支持自然语言解析 | 输入后 Enter 提交，Ctrl+T/I/L 切换类型 |
| **搜索模式** | 全局搜索覆盖层 | 输入关键词，实时展示跨类型的搜索结果 |
| **统计面板** | 数据仪表盘 | 显示待办数、今日完成数、近7天灵感/日志数 |
| **详情弹出层** | 选中项目的完整信息 | 分页展示完整内容，支持编辑字段 |

### 2.3 快捷键一览

| 快捷键 | 全局 | Todo 视图 | Idea 视图 | Log 视图 |
|--------|:----:|:---------:|:---------:|:--------:|
| **Tab / →** | 切换焦点面板 | - | - | - |
| **↑ / ↓ / j / k** | 列表导航 | - | - | - |
| **Enter** | 提交输入 / 选中操作 | 切换 done | 查看详情 | 查看详情 |
| **Ctrl+T** | 切换输入类型 → Todo | - | - | - |
| **Ctrl+I** | 切换输入类型 → Idea | - | - | - |
| **Ctrl+L** | 切换输入类型 → Log | - | - | - |
| **1 / 2 / 3** | 切换 Tab | - | - | - |
| **d** | - | 删除 Todo | 删除 Idea | 删除 Log |
| **e** | - | 编辑 Todo | 编辑 Idea | 编辑 Log |
| **a** | - | 归档/取消归档 | - | - |
| **r** | - | 重新打开 | - | - |
| **/** | 打开搜索 | - | - | - |
| **Esc** | 关闭弹层/退出搜索 | - | - | - |
| **q / Ctrl+C** | 退出 TUI | - | - | - |
| **?** | 显示帮助 | - | - | - |
| **Ctrl+R** | 手动刷新 | - | - | - |

---

## 3. 分阶段实现

### Phase 1: 最小可用 TUI 🚀

**目标**: 让用户能在终端中浏览和操作数据，实现核心 CRUD 闭环。

| 任务 | 说明 | 预估 |
|------|------|:----:|
| **P1-1** 项目骨架 | 创建 `tui/` 目录、`Cargo.toml` 添加 ratatui + crossterm 依赖、模块结构 | 0.5h |
| **P1-2** 主循环 + 布局 | 终端初始化、事件循环、左侧导航 + 右侧内容区 + 底部输入栏的 layout | 1h |
| **P1-3** Todo 视图 | 从 DB 加载 Todo、按优先级排序显示、Enter 勾选、d 删除、a 归档 | 1.5h |
| **P1-4** Idea 视图 | 从 DB 加载 Idea、时间倒序显示、d 删除 | 1h |
| **P1-5** Log 视图 | 从 DB 加载 Log、时间倒序显示、d 删除 | 1h |
| **P1-6** 快速输入栏 | 底部输入框，支持类型切换(Ctrl+T/I/L)，Enter 提交，自然语言解析 | 1.5h |
| **P1-7** 状态栏 | 顶部显示版本、当前视图、统计数据 | 0.5h |
| **P1-8** 集成测试+打磨 | 端到端测试，修复体验问题 | 1h |

**合计**: 约 7-8 小时

**Phase 1 产出**: 一个可用的 TUI 程序，能完成 Todo/Idea/Log 的浏览、添加、勾选、删除、归档。

---

### Phase 2: 增强功能 🛠️

**目标**: 增加搜索、详情查看、筛选、数据统计等提升效率的功能。

| 任务 | 说明 | 预估 |
|------|------|:----:|
| **P2-1** 全局搜索 | `/` 打开搜索弹层，实时显示搜索结果 | 1.5h |
| **P2-2** 详情弹出层 | Enter 选中项目时，弹出完整详情窗口，显示所有字段 | 1h |
| **P2-3** 视图内筛选 | 按状态(pending/done/archived)标签、项目名筛选列表 | 1h |
| **P2-4** 编辑模式 | e 键进入编辑模式，支持修改字段值 | 1.5h |
| **P2-5** 统计面板 | 按需显示统计面板（待办数、今日完成等） | 0.5h |
| **P2-6** 数据导出 | 在 TUI 中触发 JSON/CSV 导出 | 0.5h |

**合计**: 约 6 小时

---

### Phase 3: 锦上添花 ✨

**目标**: 让 TUI 变得好看又顺手。

| 任务 | 说明 | 预估 |
|------|------|:----:|
| **P3-1** 颜色主题 | 支持亮色/暗色主题，颜色配置化 | 1h |
| **P3-2** 鼠标支持 | crossterm 鼠标事件，点击选择、滚动 | 1h |
| **P3-3** 动画与反馈 | 操作成功/失败的视觉反馈（如 toast 消息） | 1h |
| **P3-4** 帮助视图 | `?` 打开完整的快捷键帮助页 | 0.5h |
| **P3-5** Vim 模式 | j/k 导航、dd 删除、/ 搜索等 vim 风格快捷键 | 0.5h |
| **P3-6** 配置文件 | `~/.config/starcatch/tui.toml` 自定义颜色和快捷键 | 1h |

**合计**: 约 5 小时

---

## 4. 目录结构

```
tui/
├── Cargo.toml             # 独立二进制 crate，依赖 starcatch 的 lib
├── src/
│   ├── main.rs            # 入口：初始化终端、启动事件循环
│   ├── app.rs             # App 状态机：视图管理、数据持有
│   ├── event.rs           # 事件循环：键盘、鼠标、resize 处理
│   ├── ui.rs              # 主渲染函数：组合所有组件
│   ├── layout.rs          # 布局定义：区域划分、尺寸计算
│   ├── components/
│   │   ├── mod.rs
│   │   ├── sidebar.rs     # 左侧导航面板
│   │   ├── todo_list.rs   # Todo 列表组件
│   │   ├── idea_list.rs   # Idea 列表组件
│   │   ├── log_list.rs    # Log 列表组件
│   │   ├── quick_input.rs # 底部快速输入栏
│   │   ├── search.rs      # 搜索弹层
│   │   ├── detail.rs      # 详情弹出层
│   │   ├── status_bar.rs  # 顶部状态栏
│   │   └── help.rs        # 帮助视图
│   ├── handler.rs         # 按键事件分发到各组件的处理函数
│   ├── styles.rs          # 颜色和样式定义
│   └── input_parser.rs    # 自然语言输入解析（共用或重写 CLI 的 parser）
```

---

## 5. 关键设计决策

### 5.1 独立二进制 vs Feature Flag

**决策：独立二进制 crate。**

```
/Starcatch/
├── Cargo.toml          # workspace root
├── src/                # CLI 二进制 (现有)
├── qt/                 # Qt GUI (现有)
├── tui/                # ★ TUI 二进制 (新增)
│   ├── Cargo.toml      # 依赖 ratatui + crossterm + starcatch-core
│   └── src/
└── starcatch-core/     # 共享库 (新增)
    ├── Cargo.toml      # 从现有 src/ 提取的 db + models
    └── src/
```

**理由：**
- TUI 的依赖（ratatui, crossterm）是重量级的，加到 CLI 的 feature flag 里会增加非 TUI 用户的编译时间
- 独立二进制可以独立发布、独立版本号
- 通过提取 `starcatch-core` 共享 lib，CLI、GUI、TUI 都依赖同一份 model + db
- 渐进式重构：现有 CLI 先小改，把核心逻辑提成 lib

### 5.2 与其他组件共享数据

TUI 直接复用 `db.rs` 的 CRUD 函数，操作同一份 SQLite 文件。CLI/GUI/TUI 修改的数据立即可见（WAL 模式下可并发读）。

### 5.3 输入解析复用

快速输入栏的自然语言解析（P1, #tag, due:明天, project:名称）复用现有 `parse_pipe_*` 函数或提取为共享模块。

### 5.4 刷新策略

- 每个操作（添加、删除、标记）后自动刷新列表
- 周期性自动刷新（可选，每 30 秒）
- 手动 Ctrl+R 强制刷新
- 不实现实时监听（避免复杂性）

---

## 6. Cargo.toml 草案

```toml
[package]
name = "starcatch-tui"
version = "0.1.0"
edition = "2024"
description = "Terminal UI for Starcatch — catch your starlight ideas in the terminal"

[dependencies]
# TUI
ratatui = "0.30"
crossterm = "0.28"

# Database (reuse from core lib)
starcatch-core = { path = "../starcatch-core" }

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Serialization (for input parsing)
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# UUID (for creating new entries)
uuid = { version = "1", features = ["v4"] }
```

---

## 7. 未来展望

- **Unix Socket 守护进程模式**: TUI 连接到后台守护进程，实现常驻通知
- **Tmux 集成**: 检测 tmux 状态，自动适配窗口大小
- **异步后端**: 使用 tokio 实现非阻塞 DB 操作（当前 SQLite 操作同步，大表可能卡 UI）
- **Pipe 模式集成**: TUI 也可以作为 pipe 的目标，`echo "灵感" | starcatch-tui --pipe`
- **HTML/ANSI 导出**: 将列表导出为彩色 HTML 或 ANSI 文本，方便粘贴到笔记中

---

## 8. 实施笔记

### 开始实施前的准备

1. 先创建 `starcatch-core` 共享库，把 `models/`、`db.rs` 从 CLI 提取出来
2. 创建 `tui/` 目录和 `Cargo.toml`
3. 确保 `cargo build` 能在 tui 目录正常工作
4. 然后按 Phase 1-3 的顺序逐步实现

### 测试方式

```bash
# 在开发目录运行
cd tui
# 使用测试数据库（不污染真实数据）
cargo run -- --db /tmp/starcatch-test.db
```

### 参考项目

- [bottom](https://github.com/ClementTsang/bottom) — Rust TUI 系统监控，Ratatui 实践参考
- [gitui](https://github.com/extrawurst/gitui) — Rust TUI Git 客户端，布局可参考
- [yazi](https://github.com/sxyazi/yazi) — Rust TUI 文件管理器，风格可参考
