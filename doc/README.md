# ⭐ Starcatch (星捕) — 文档

> Catch your starlight ideas.

名称源自《彼方的她-Aliya》——溯星逆流追寻星渺，Starcatch 捕捉途中洒落的星光。

## 快速导航

- **[README.md](../README.md)** — 安装、使用、快捷键
- **[install.sh](../install.sh)** — 一键安装脚本

## 三大记录类型

### 📋 Todo — 任务管理

| 特性 | 说明 |
|------|------|
| 优先级 | **P0** 🔴 紧急 / **P1** 🟡 重要 / **P2** 🟢 一般 / **P3** ⚪ 低优 |
| 状态 | ⬜ Pending → ✅ Done → 📦 Archived |
| 截止日期 | 可选，自然语言（`明天`、`3天`、`下周一`） |
| 标签 | 逗号分隔 |
| 项目 | 所属项目 |

**生命周期：** 所有 To-do 始于 Pending → Done（标记完成，保留记录）→ Archived（隐藏，不删除）。

**列表过滤：**
| 选项 | 显示内容 |
|------|---------|
| (默认) | pending + done |
| `--pending` | 仅待办 |
| `--done` | 仅已完成 |
| `--archived` | 仅已归档 |
| `--all` | 全部（含 archived） |
| `--tag` `--project` | 按标签/项目筛选 |

### 💭 Idea — 灵感闪念

标题（必填）+ 内容（可选）+ 来源（`--source`）+ 标签 + 自动记录时间。

### 📓 Log — 随手记 / 日记

内容（必填）+ 心情（`--mood`）+ 标签。

## CLI 命令一览

```bash
starcatch todo add|list|edit|show|done|archive|reopen|delete
starcatch idea add|list|edit|show|delete
starcatch log   add|list|edit|show|delete
starcatch pipe  todo|idea|log       # stdin
starcatch search <query>
starcatch stats
starcatch export --format json|csv
starcatch completions bash|zsh|fish|elvish|powershell
```

## TUI 快捷键

| 快捷键 | 功能 |
|--------|------|
| `/` | 进入输入模式 |
| `Tab` / `←` `→` | 切换视图 |
| `↑` `↓` / `j` `k` | 导航 |
| `Enter` | 标记完成 / 查看详情 |
| `e` | 编辑 |
| `d`（两次） | 删除 |
| `a` | 归档 |
| `1` `2` `3` | 快速切视图 |
| `?` | 帮助 |
| `q` / `Ctrl+C` | 退出 |

## 技术栈

| 层面 | 选择 |
|------|------|
| CLI | Rust + clap |
| TUI | ratatui + crossterm |
| 数据库 | SQLite (rusqlite + WAL) |
| 序列化 | serde + serde_json |
| UUID | uuid v4 |
| 时间 | chrono |

## 项目结构

```
├── Cargo.toml              # 工作区
├── src/                    # CLI
│   ├── main.rs
│   └── cli.rs
├── starcatch-core/         # 共享核心（模型 + DB + 解析器）
├── tui/                    # 终端界面
└── qt/                     # Qt GUI（存档）
```
