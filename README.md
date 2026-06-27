# ⭐ Starcatch (星捕)

Catch your starlight ideas — Wayland 原生的 idea/todo/log 快速捕获工具。

| 界面 | 描述 |
|------|------|
| 💻 **CLI** | Rust 命令行，支持子命令 + pipe 模式 |
| 🖥️ **TUI** | 终端界面，ratatui + crossterm，支持键盘导航和编辑 |
| 🗄️ **Core** | 共享 SQLite 数据库 (`rusqlite` + WAL) |

## 安装

### 依赖

- Rust 1.85+（edition 2024）

### 一键安装

```bash
./install.sh                            # 只装 CLI
INSTALL_TUI=1 ./install.sh              # CLI + TUI
INSTALL_COMPLETIONS=1 ./install.sh      # CLI + bash 补全
```

### 手动安装

```bash
cargo build --release
cp target/release/starcatch ~/.local/bin/
# TUI（可选）
cargo build --release -p starcatch-tui
cp target/release/starcatch-tui ~/.local/bin/
```

## 使用

### CLI

```bash
# 📋 Todo
starcatch todo add "买牛奶" --due 明天 -p P1 --tag 购物
starcatch todo list                    # pending + done
starcatch todo done <id>               # 标记完成
starcatch todo archive <id>            # 归档

# 💭 Idea
starcatch idea add "做个AI助手" --source 洗澡 --tag AI

# 📓 Log
starcatch log add "今天写了代码" --mood happy

# 🚰 Pipe 模式
echo "买牛奶和面包" | starcatch pipe todo
echo "今天搞定了 TUI 实现" | starcatch pipe log

# 🔍 搜索 / 📊 统计 / 📤 导出
starcatch search "关键词"
starcatch stats
starcatch export --format json
starcatch export --format csv
```

### TUI

```bash
starcatch-tui
```

| 快捷键 | 功能 |
|--------|------|
| `/` | 进入输入模式 |
| `Tab` / `←` `→` | 切换视图 (Todo/Idea/Log) |
| `↑` `↓` / `j` `k` | 导航条目 |
| `Enter` | 标记 todo 完成 / 查看详情 |
| `e` | 编辑当前条目 |
| `d` (`d` 两次) | 删除（需确认） |
| `a` | 归档 todo |
| `1` `2` `3` | 快速切换视图 |
| `?` | 显示帮助 |
| `q` / `Ctrl+C` | 退出 |

编辑模式下：`Ctrl+A` 行首 / `Ctrl+E` 行尾 / `Ctrl+K` 删至行末 / `Ctrl+T/I/L` 切换输入类型。

### 快速输入语法

| 语法 | 说明 |
|------|------|
| `P0` `P1` `P2` `P3` | 优先级 |
| `due:明天` | 截止日期（自然语言） |
| `#标签` | 标签 |
| `project:项目名` | 所属项目 |
| `mood:happy` | 心情（仅 Log） |
| `source:来源` | 来源（仅 Idea） |
| `标题 \| P1 #tag due:明天` | 编辑模式：`\|` 左侧为原样标题 |

## 数据

存储在 `~/.local/share/starcatch/starcatch.db`，CLI 与 TUI 共享（通过 `-D <path>` 覆盖）。

## 项目结构

```
├── Cargo.toml                     # 工作区根
├── src/                           # CLI 入口
│   ├── main.rs
│   └── cli.rs
├── starcatch-core/                # 共享核心库
│   └── src/
│       ├── lib.rs
│       ├── db.rs                  # SQLite CRUD
│       ├── parser.rs              # 管道输入解析
│       └── models/                # Todo, Idea, Log
├── tui/                           # 终端界面
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── event.rs
│       ├── handler.rs
│       ├── ui.rs
│       ├── styles.rs
│       └── components/
└── qt/                            # Qt GUI（历史）
```
