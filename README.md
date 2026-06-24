# Starcatch 星捕

Catch your starlight ideas — Wayland 原生的 idea/todo/log 快速捕获工具。

## 安装

### 依赖

- **CLI**：Rust 1.96+
- **GUI**：Qt 6 (Core, Gui, Widgets, Sql), CMake 3.16+, gcc/clang

Arch Linux：
```bash
pacman -S qt6-base cmake gcc
```

### CLI

```bash
cargo install --path .
```

安装到 `~/.cargo/bin/starcatch`

### GUI

```bash
cd qt
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
cp build/starcatch-qt ~/.local/bin/
```

安装到 `~/.local/bin/starcatch-qt`

## 使用

```bash
# CLI
starcatch todo add "买牛奶" --due 明天 --priority P1
starcatch todo list
starcatch idea add "做个AI助手" --tag tech,ai
starcatch log add "今天写了代码" --mood happy

# GUI
starcatch-qt
```

### 快速输入语法（GUI）

| 语法 | 说明 |
|------|------|
| `P0` `P1` `P2` `P3` | 优先级 |
| `due:明天` | 截止日期 |
| `#标签` | 标签 |
| `/t` `/i` `/l` | 切换输入类型 |
| `/help` | 查看命令 |

### 命令

| 命令 | 说明 |
|------|------|
| `/help` | 可用命令 |
| `/theme` | 切换主题 |
| `/search` | 搜索 |
| `/stats` | 统计 |
| `/export` | 导出 Markdown |

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+1-4` | 切换标签 |
| `Ctrl+N` | 聚焦输入 |
| `Ctrl+Shift+T` | 切换主题 |
| `j/k` `↓/↑` | 导航条目 |
| `Enter` | 勾选 todo |
| `Tab` | 命令补全 |
| `Esc` | 关闭 |

### 数据

存储在 `~/.local/share/starcatch/starcatch.db`，CLI 与 GUI 共享。
