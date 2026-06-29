# fix-tui-edit-metadata-loss - Work Plan

## TL;DR (For humans)

**What you'll get:** TUI 编辑 todo/idea/log 时不再丢失优先级、标签、截止日期、来源、心情、项目这些元数据。按 `e` 编辑时会看到完整的文本（含所有元数据），修改后提交，一切保持原样。

**Why this approach:** 问题出在 `start_edit()` 只预填了 title/content，提交时重新解析导致元数据丢失。改成预填完整文本后，解析器能正确 round-trip，不用改 parse 逻辑或 DB 层。

**What it will NOT do:** 不改 CLI（src/）和 Qt GUI（qt/）；不改 parser 和 DB；不改 handler/ui/component 文件。

**Effort:** Quick
**Risk:** Low — 单文件改一个方法，已有单元测试覆盖 parser 的 round-trip
**Decisions to sanity-check:** Priority 用 `to_string()` 输出 "P0"-"P3"，恰好被 parser 识别

Your next move: 执行 `$start-work` 启动修复。下面是详细执行计划。

---

> TL;DR (machine): Quick | Low | 修 TUI 编辑时元数据丢失 bug，改 `tui/src/app.rs:start_edit()` 预填完整文本

## Scope
### Must have
- 修改 `tui/src/app.rs` 中 `start_edit()` 方法
- Todo 预填格式：`"P1 标题 #tag due:2026-06-28 project:xxx"`
- Idea 预填格式：`"标题 source:book #tag project:xxx"`
- Log 预填格式：`"内容 mood:happy #tag project:xxx"`
- Rust 编译通过（`cargo build -p starcatch-tui`）
- 现有测试通过（`cargo test -p starcatch-tui`）
- round-trip 验证：prefill → parse 结果与原字段一致

### Must NOT have (guardrails, anti-slop, scope boundaries)
- 不改其他文件（handler.rs, ui.rs, components/*, event.rs, styles.rs）
- 不改 Rust CLI（src/）或 Qt GUI（qt/）
- 不改 parser.rs 或 db.rs
- 不改测试文件
- 不引入 emoji/非 ascii 符号到 prefill text（只使用纯文本语法如 "P1", "#tag", "due:", "mood:", "source:", "project:"）

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: tests-after — 已有 parser 单元测试，改完后重新编译运行确认
- Evidence: `cargo build -p starcatch-tui 2>&1` + `cargo test -p starcatch-core 2>&1`

## Execution strategy
### Parallel execution waves
Single wave: 1 todo（单文件单方法修改）

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1 | — | — | — |

## Todos
> Implementation + Test = ONE todo. Never separate.
<!-- APPEND TASK BATCHES BELOW THIS LINE WITH edit/apply_patch - never rewrite the headers above. -->
- [ ] 1. 修复 start_edit() 预填完整元数据
  What to do / Must NOT do:
  改 `tui/src/app.rs` 中 `start_edit()` 方法（当前行 399-429）。把原来的 `(t.id.clone(), t.title.clone())` / `(t.id.clone(), t.content.clone())` 替换为拼装完整元数据的文本。

  修改逻辑：
  - **Todo**：`[priority] [title] [#tag1] [#tag2] [due:YYYY-MM-DD] [project:xxx]`
    使用 `t.priority.to_string()` 获取 "P0"-"P3"；tags 逐个加 `#` 前缀；due_date 加 `due:` 前缀；project 加 `project:` 前缀
  - **Idea**：`[title] [source:xxx] [#tag] [project:xxx]`
  - **Log**：`[content] [mood:xxx] [#tag] [project:xxx]`

  用 `Vec<String>` 收集各部分，最后 `.join(" ")` 拼成完整文本。

  Must NOT do: 不改其他文件；不改 parser/DB；不使用 emoji 作为语法标记。

  Parallelization: Wave 1 | Blocked by: — | Blocks: —
  References (executor has NO interview context - be exhaustive):
  - `tui/src/app.rs:399-429` — 当前 start_edit() 实现
  - `tui/src/app.rs:164-198` — submit_input() 编辑模式调用 parse_pipe_* 重解析
  - `starcatch-core/src/parser.rs:64-129` — parse_pipe_todo 可识别的语法
  - `starcatch-core/src/parser.rs:132-171` — parse_pipe_idea 可识别的语法
  - `starcatch-core/src/parser.rs:174-213` — parse_pipe_log 可识别的语法
  - `starcatch-core/src/models/todo.rs:27-36` — Priority::Display 输出 "P0"-"P3"
  - Draft: `.omo/drafts/fix-tui-edit-metadata-loss.md`
  Acceptance criteria (agent-executable):
  1. `cargo build -p starcatch-tui 2>&1` 编译无 error/warning
  2. `cargo test -p starcatch-core 2>&1` 全部通过（parser 单元测试验证 round-trip）
  3. `cargo test -p starcatch-tui 2>&1` 全部通过（现有 app.rs 工具函数测试）
  QA scenarios:
  - Happy: 改完后编译 + 测试全部通过，prefill 文本格式可被 parser 正确 round-trip
  - Failure: 如果 prefill 格式与 parser 期望不一致，cargo test 中的 parser 用例应能捕获
  Evidence: `.omo/evidence/task-1-fix-tui-edit-metadata-loss.txt`
  Commit: Y | `fix(tui): prefill full metadata in start_edit to prevent silent data loss on edit`

## Final verification wave
> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.
- [ ] F1. Plan compliance audit — 是否只改了一个文件一个方法
- [ ] F2. Code quality review — 代码风格、unwrap 风险检查
- [ ] F3. Real manual QA — 手动审查 prefill 格式与 parse_* 输入的兼容性
- [ ] F4. Scope fidelity — 没有漏改/多改

## Commit strategy
1 个 commit: `fix(tui): prefill full metadata in start_edit to prevent silent data loss on edit`

## Success criteria
- 编译通过 ✅
- 全部测试通过 ✅
- Bug #10 修复：编辑任何类型的条目，所有元数据 round-trip 正确 ✅
