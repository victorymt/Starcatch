# Starcatch CLI 功能缺口计划

> 创建时间: 2026-06-25
> 作者: 星渺 (Xīng Miǎo)

---

## ✅ 已有功能

| 模块 | 功能 | 说明 |
|------|------|------|
| **Todo** | `add` | 添加待办，支持 priority/due/tag/project/desc |
| | `list` | 列表，支持 --pending/--done/--archived/--all/--tag 筛选 |
| | `done` | 标记完成 |
| | `archive` | 归档 |
| | `reopen` | 重新打开 |
| **Idea** | `add` | 添加灵感，支持 content/source/tag |
| | `list` | 列表（默认7天），支持 --days |
| **Log** | `add` | 添加日志，支持 mood/tag |
| | `list` | 列表（默认1天），支持 --days |
| **Pipe** | `todo/idea/log` | 从 stdin 管道捕获，支持解析 P0-P3 / due: / #tags |
| **通用** | `--json` | 输出 JSON 格式 |
| | `-D` | 自定义数据库路径 |
| | 自然语言日期 | `tomorrow`, `下周X`, `N天`, `next Monday` 等 |

---

## 🔴 核心缺失 — 高优先级

### 1. `starcatch todo edit <id>`

目前只能改 status（done/archive/reopen），不能修改：
- title
- priority
- due_date
- tags
- project
- description

**实现方案**: 在 `cli.rs` 添加 `TodoCommands::Edit(TodoEditArgs)`，在 `db.rs` 添加 `update_todo()`，在 `main.rs` 添加 `handle_todo_edit()`。

### 2. `starcatch todo delete <id>`

目前没有删除功能，只能归档。

**实现方案**: 在 `cli.rs` 添加 `TodoCommands::Delete { id }`，在 `db.rs` 添加 `delete_todo()`。

### 3. `starcatch idea delete <id>`

Idea 完全无法删除或修改。

**实现方案**: 在 `cli.rs` 的 `IdeaCommands` 添加 `Delete { id }`，在 `db.rs` 添加 `delete_idea()`。

### 4. `starcatch log delete <id>`

Log 完全无法删除。

**实现方案**: 在 `cli.rs` 的 `LogCommands` 添加 `Delete { id }`，在 `db.rs` 添加 `delete_log()`。

### 5. `starcatch idea edit <id>`

修改 idea 的 title/content/source/tags。

### 6. `starcatch log edit <id>`

修改 log 的 content/mood/tags。

---

## 🟡 增强体验 — 中优先级

### 7. `starcatch todo show <id>`

查看单个 todo 的完整信息（包括 description、完整时间等）。

### 8. `starcatch idea show <id>`

查看 idea 详情（显示 content）。

### 9. `starcatch log show <id>`

查看 log 详情。

### 10. `starcatch stats`

统计/概览，显示：
- 今日待办数 / 完成数
- 本周灵感数
- 近7天日志数

### 11. `starcatch search <query>`

全局搜索 todos/ideas/logs 的 title 和 content。

**实现方案**: 在 `cli.rs` 添加 `Commands::Search(SearchArgs)`，使用 SQL `LIKE` 查询三个表。

### 12. `starcatch export --format json/csv`

导出数据。

### 13. `starcatch todo list --project <name>`

已有 project 字段但 list 不支持筛选。

### 14. `starcatch todo list --overdue`

只显示已过期（due_date < today 且 status != done）的待办。

---

## 🔵 细节打磨 — 低优先级

### 15. `starcatch todo list --today`

显示今天到期的待办（due_date == today）。

### 16. Idea list 支持 tag 筛选

已有 todo 支持 `--tag` 筛选。

### 17. Log list 支持 tag/mood 筛选

Log 有 mood 和 tags 字段，但没有筛选。

### 18. Pipe 模式支持 `project:`

Pipe 解析了 priority/due/tag，但没解析 project。

### 19. Shell Tab 补全

安装 shell 自动补全会更方便。

### 20. `starcatch commit`

类似 git commit 风格的一条命令直接捕获：

```
starcatch commit -m "P1 fix login bug #urgent due:tomorrow"
```

自动解析 priority/tags/due/type。

---

## 💫 推荐开发顺序

1. **`starcatch todo edit`** — 最常见的日常需求，改 priority/due/tags
2. **`starcatch todo delete` / `starcatch idea delete` / `starcatch log delete`** — 基本 CRUD
3. **`starcatch todo list --overdue` + `--today`** — 日常高频
4. **`starcatch search`** — 全局搜索超实用
5. **`starcatch idea edit` / `starcatch log edit`** — 修改已有内容
