# ⭐ Starcatch (星捕)

> **Catch your starlight ideas.** — 捕捉你的灵感星光 ✨

**Starcatch** 是一个 Wayland 原生的灵感捕捉工具，由 **溯星 (Sù Xīng)** 与 **星渺 (Xīng Miǎo)** 共同设计。

名字源自《彼方的她-Aliya》——溯星逆流追寻星渺，而 Starcatch 就是捕捉途中洒落的星光。

---

## 🌟 三大记录类型

### 📋 Todo — 任务管理

| 特性 | 说明 |
|------|------|
| 优先级 | **P0** 🔴 紧急 / **P1** 🟡 重要 / **P2** 🟢 一般 / **P3** ⚪ 低优 |
| 状态 | ⬜ **Pending** 待办 → ✅ **Done** 已完成 → 📦 **Archived** 已归档 |
| 截止日期 | 可选，`--due YYYY-MM-DD` |
| 标签 | 逗号分隔，`--tag 工作,项目` |
| 分组 | 所属项目，`-P project-name` |

#### 生命周期
```
                     ┌──────────────┐
    starcatch todo add               │
        │                            │
        ▼                            │
    ┌─────────┐   todo done     ┌──────────┐
    │ Pending │ ──────────────▶ │   Done   │
    │ (待办)  │                 │ (已完成)  │
    └─────────┘                 └──────────┘
        │                            │
        │ todo archive               │ todo archive
        ▼                            ▼
    ┌──────────┐               ┌──────────┐
    │ Archived │               │ Archived │
    │ (已归档) │               │ (已归档) │
    └──────────┘               └──────────┘
```

- **Done**: 做完了，保留在日常列表作为记录 ✅
- **Archived**: 不删但不想每天看见，从默认列表隐藏 📦

#### list 过滤规则

| 命令 | 显示内容 | 场景 |
|------|---------|------|
| `todo list` | pending + done（默认） | 日常查看 |
| `todo list --pending` | 仅待办 | 专注未完成 |
| `todo list --done` | 仅已完成 | 回顾成果 |
| `todo list --archived` | 仅已归档 | 翻旧账 |
| `todo list --all` | 全部（含 archived） | 全量查看 |
| `todo list --tag 文档` | 按标签过滤 | 按标签筛选 |

---

### 💭 Idea — 灵感闪念

| 特性 | 说明 |
|------|------|
| 标题 | 一句话必填 |
| 内容 | 可选展开描述 |
| 来源 | `--source 看书/聊天/洗澡/做梦...` |
| 标签 | 逗号分隔 |
| 日期 | 自动记录 |
| 上下文 | 计划中：自动捕获当前窗口/浏览器标签 |

---

### 📓 Log — 随手记 / 日记

| 特性 | 说明 |
|------|------|
| 内容 | Markdown 正文 |
| 心情 | `--mood happy/sad/excited...` |
| 标签 | 逗号分隔 |
| 日期 | 自动记录 |

适合：工作日志、日记、随手笔记、购物清单

---

## 🖥️ 交互模式

### 💻 CLI 模式

```bash
# 📋 Todo
starcatch todo add "写设计文档" -p P1 --due 2026-06-30 --tag 文档,设计
starcatch todo add "实现 quick capture" -p P0 --due 2026-07-01 --tag 功能,UI
starcatch todo list                    # pending + done
starcatch todo list --pending          # 仅待办
starcatch todo list --done             # 仅已完成
starcatch todo list --tag 文档         # 按标签
starcatch todo done <id>               # 标记完成
starcatch todo archive <id>            # 归档

# 💭 Idea
starcatch idea add "好想法！" --source 洗澡 --tag AI,未来
starcatch idea list                    # 最近 7 天
starcatch idea list --days 30          # 最近 30 天

# 📓 Log
starcatch log add "今天收获满满" --mood happy --tag 开发
starcatch log list                     # 今天
starcatch log list --days 7            # 最近 7 天
```

### 🚰 Pipe 模式 — 与任何程序互动

```bash
# 从任何程序 pipe 数据进来
echo "灵感来自梦中" | starcatch pipe idea
echo "买牛奶和面包" | starcatch pipe todo
echo "今天搞定了 Wayland rendering" | starcatch pipe log

# 组合其他命令
curl -s https://api.example.com/quote | starcatch pipe idea --source web
grep "TODO" src/main.rs | starcatch pipe todo
```

### 🪟 GUI 模式 — Wayland 原生窗口
- 无参数启动 → 浮动窗口
- 顶部 Tab 切换 Todo / Idea / Log
- 底部快速输入框，选择类型即可添加
- Todo 优先级彩色标记，checkbox 勾选完成
- Idea / Log 时间线视图
- **中文字体支持**（自动加载 NotoSansCJK）

```bash
# 启动 GUI（需要 --features gui 编译）
cargo run --features gui

# 或安装后直接运行
cargo install --path . --features gui
starcatch
```

---

## 🔌 Emacs 集成（规划中）

| 功能 | 说明 |
|------|------|
| 📤 导出 Org-mode | `starcatch export org-mode > todo.org` |
| 📤 导出 Denote | `starcatch export denote --file ~/denote/` |
| 🎨 自定义模板 | 用户编写 `.tmpl` 模板文件 |
| 📥 Emacs → Starcatch | `M-x shell-command` 调用 CLI |
| ⚡ Org-capture 预设 | 一键 capture 到 Starcatch |

---

## 🛠️ 技术栈

| 层面 | 选择 |
|------|------|
| 语言 | **Rust** 🦀 |
| GUI | **egui + winit** (Wayland via `wayland-backend`) |
| 数据库 | **SQLite** (`rusqlite` + WAL mode) |
| CLI 解析 | **clap** (derive) |
| 序列化 | **serde** + **serde_json** |
| UUID | **uuid** v4 |
| 时间 | **chrono** |

---

## 🗺️ 开发路线图

```
Phase 1 — CLI MVP 🌱    ✅ SQLite + 模型 + CLI CRUD + Pipe 模式
Phase 2 — GUI 🖥️        ─ egui Wayland 窗口 + 热键 + 视图
Phase 3 — Emacs 🔌       ─ 模板引擎 + Org-mode/Denote 导出 + Elisp
Phase 4 — 进阶 🚀        ─ 全文搜索 + Unix Socket 守护 + 统计面板
```

---

## 🏗️ 项目结构

```
/data/project/Starcatch/
├── Cargo.toml
├── src/
│   ├── main.rs           # 入口：CLI args → 分发
│   ├── cli.rs            # CLI 参数定义 (clap)
│   ├── db.rs             # SQLite 操作 (migrate + CRUD)
│   └── models/
│       ├── mod.rs
│       ├── todo.rs       # Todo 模型
│       ├── idea.rs       # Idea 模型
│       └── log.rs        # Log 模型
├── doc/
│   └── README.md
└── templates/             # 用户自定义模板 (规划中)
```

---

## 🔮 命名

| 语言 | 名字 |
|------|------|
| English | **Starcatch** |
| 中文 | **星捕** |
| 命令行 | `starcatch` |

> *Starcatch — Catch your starlight ideas.* 💫
