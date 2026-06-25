# 🔍 代码审查 Bug 报告

**审查日期：** 2025-01
**审查范围：** Rust CLI (`src/`) + C++ Qt GUI (`qt/src/`)

---

## 🔴 严重 Bug（功能不正确）

### 1. 内联编辑：`editingFinished` + `returnPressed` 双重触发 → 重复 `refresh()`

**文件：** `qt/src/ideapanel.cpp:96-97`, `qt/src/todopanel.cpp:174-175`, `qt/src/logpanel.cpp:113-114`

每个 `startEdit()` 同时连接了 `returnPressed` 和 `editingFinished`：
```cpp
connect(edit, &QLineEdit::returnPressed, this, [finish]() { finish(true); });
connect(edit, &QLineEdit::editingFinished, this, [finish]() { finish(true); });
```

**影响：** 在 Qt 中，按下 Enter 时**两个信号都会按顺序触发**。`returnPressed` 先触发 →
`finish(true)` → `emit titleEdited(...)` → `handleTitleEdit()` → `refresh()`（重建整个列表）→ 然后
`editingFinished` 触发 → `finish(true)` **再次执行** → 又一轮刷新。导致两次连续的 `refresh()`
（性能浪费 + UI 闪烁），且 `lay->removeWidget(edit)` 和 `edit->deleteLater()` 在已被清理的 widget
上重复调用。

**修复方案：** 只连接 `returnPressed`，移除 `editingFinished` 的连接。

---

### 2. Rust CLI：`handle_pipe` 使用了错误的错误类型

**文件：** `src/main.rs:376`

```rust
.map_err(|e| rusqlite::Error::InvalidParameterName(format!("stdin read error: {}", e)))?;
```

**影响：** `rusqlite::Error::InvalidParameterName` 是用于**无效 SQL 参数名称**的错误变体，
用它包装 stdin IO 错误是语义错误。如果 stdin 读取失败，用户会看到误导性的错误消息。

**修复方案：** 改用 `rusqlite::Error::ToSqlConversionFailure(Box::new(e))`，或在捕获 IO 错误时
直接 `eprintln` 并提前返回 `Ok(())`。

---

### 3. Rust & C++：正则表达式 `THIS_ZH_RE` / `NEXT_ZH_RE` 对特定输入匹配错误

**文件：** `src/main.rs:64-65`, `qt/src/inputparser.cpp:135-136`

**模式 1**（THIS）：`^(?:这|本|这周|本周|这星期|本星期)(?:周|星期|礼拜)?(.)`
**模式 2**（NEXT）：`^下(?:周|星期|礼拜)?(.)`

**问题：** 短选项排在长选项之前。对输入 **"本周礼拜五"**：
- `^(?:这|本|这周|本周|这星期|本星期)` 匹配 `本`
- `(?:周|星期|礼拜)?` 匹配 `周`
- `(.)` 捕获 `礼` ❌（应为 `五`）

同理 **"下周礼拜五"** → 捕获 `礼` ❌（应为 `五`）。

**修复方案：** 将长选项移到短选项之前：
```
(?:这周|本周|这星期|本星期|这|本)
```

---

## 🟡 中等 Bug（逻辑/设计问题）

### 4. C++ `AllPanel::rebuildList` 硬编码天数为 7 和 1

**文件：** `qt/src/allpanel.cpp:165,181`

```cpp
auto ideas = m_db->listIdeas(7);
auto logs = m_db->listLogs(1);
```

**影响：** All 面板使用固定天数，不尊重 Idea/Log 面板滑块的当前值。如果用户将 Idea 滑块设为
30 天，All 面板仍只显示 7 天的想法，造成数据不一致。

**修复方案：** 从全局配置或面板实例获取当前天数，或使用更大的固定值（如 365）。

---

### 5. `TodoItemWidget` 计算了 `dueToday` 但从未使用

**文件：** `qt/src/todopanel.cpp:98`

```cpp
bool overdue = due.isValid() && due <= today;
bool dueToday = due.isValid() && due == today;  // ← 从未使用
```

**影响：** `dueToday` 被计算但从未被引用。"今天到期"的样式丢失——今天的 todo 本应有特殊视觉标记
（如"due today"标签）。

**修复方案：** 使用 `dueToday` 变量为今天到期的条目添加特殊样式，或移除未使用的变量。

---

### 6. `MainWindow::updateTabLabels` 有无用变量

**文件：** `qt/src/mainwindow.cpp:219`

```cpp
int total = activeCount + (int)todos.size() - activeCount; // pending + done
```

**影响：** `total` 的计算结果等价于 `todos.size()`，且该变量**从未被使用**。

**修复方案：** 移除未使用的变量。

---

### 7. 面板中的 `handleTitleEdit`/`handleContentEdit` 绕过 `Database` 类

**文件：** `qt/src/ideapanel.cpp:220-222`、`qt/src/todopanel.cpp:376-378`、`qt/src/logpanel.cpp:237-239`

这些方法直接使用硬编码的 `QSqlDatabase::database("starcatch_conn")` 和执行原始 SQL，
而不是通过 `Database` 类的实例。

**影响：** 如果连接名称变化或 `Database` 类内部重构，这些方法会静默失效。
`Database` 类缺少 `updateIdeaTitle()`、`updateTodoTitle()`、`updateLogContent()` 方法。

**修复方案：** 在 `Database` 类中添加相应方法并让面板调用它们。

---

### 8. `priorityToString` 在 `AllPanel::rebuildList` 中被调用了 3 次

**文件：** `qt/src/allpanel.cpp:152-155`

```cpp
e.icon = priorityToString(t.priority) == QStringLiteral("P0") ? "🔴" :
         priorityToString(t.priority) == QStringLiteral("P1") ? "🟡" :
         priorityToString(t.priority) == QStringLiteral("P3") ? "⚪" :
         "📋";
```

**影响：** `priorityToString` 被调用了 3 次（性能浪费）。且 P2 显示 "📋" 而其他面板用 🟢，
UI 一致性不够。

**修复方案：** 局部变量存储一次调用结果。P2 也应显示 🟢。

---

## ⚪ 轻微 Bug / 代码异味

### 9. Rust `parse_natural_date` 中 "next week" 分支的死代码

**文件：** `src/main.rs:105-106`

```rust
let delta = 7 - today.weekday().num_days_from_monday() as i64;
if delta <= 0 { ... }  // 不可达：delta 范围 1-7
```

`num_days_from_monday()` 返回 0-6，所以 `delta` 范围是 1-7，条件分支永远不可达。

### 10. C++ `parseNaturalDate` 缺少数字溢出保护

**文件：** `qt/src/inputparser.cpp:102`

```cpp
m.captured(1).toInt()
```

如果输入极端大的数字（如 `999999999天`），`toInt()` 可能溢出。建议加长度或范围检查。

### 11. Rust CLI 中 match 块样式不一致

**文件：** `src/main.rs:292-329, 338-367`

`handle_idea` 和 `handle_log` 的 match 分支以 `println!` 结尾（返回 `()`），依赖函数末尾的
`Ok(())`。可以工作但可读性不如在每个分支显式写 `Ok(())`。

### 12. 内联 `finish` lambda 的 `lay` 指针隐患

如果 `refresh()` 在 `finish()` 执行期间完全销毁了 widget，`lay`（`QHBoxLayout*`）指针会变成
野指针。当前因 `deleteLater` 的延迟删除而不会立即崩溃，但这是一个脆弱的设计。

---

## 📊 统计

| 严重级别 | 数量 | 编号 |
|---------|------|------|
| 🔴 严重 | 3 | #1, #2, #3 |
| 🟡 中等 | 5 | #4, #5, #6, #7, #8 |
| ⚪ 轻微 | 4 | #9, #10, #11, #12 |
| **合计** | **12** | |

---

## 🎯 优先修复建议

1. **#1** 双重信号 → 移除 `editingFinished` 连接（3 个文件修改）——低风险高回报
2. **#3** 正则表达式匹配错误 -> 调换备选项顺序——影响自然语言解析准确性
3. **#2** 错误类型误用 -> 改用正确的 `rusqlite::Error` 变体
4. **#4** All 面板硬编码天数 -> 读取面板当前值或使用 365
