---
slug: fix-tui-edit-metadata-loss
status: approved
intent: clear
pending-action: write .omo/plans/fix-tui-edit-metadata-loss.md
approach: Prefill full metadata in start_edit() for round-trip safety
---

# Draft: fix-tui-edit-metadata-loss

## Components (topology ledger)
| id | outcome | status | evidence path |
|----|---------|--------|-------------|
| C1 | 修改 `tui/src/app.rs:start_edit()` 预填完整元数据 | active | app.rs:399-429 |
| C2 | Rust 编译 + cargo test 通过 | active | cargo build |

## Open assumptions (announced defaults)
| assumption | adopted default | rationale | reversible? |
|-----------|----------------|-----------|-------------|
| 只修 bug #10 | 其他 medium/low 后续单独 PR | 用户说"严重bug先修" | Yes |
| 不修主 CLI/Qt GUI 的 bug | 用户只看 TUI | 范围清晰 | Yes |

## Findings (cited - path:lines)
- app.rs:399-429 — `start_edit()` 只取 `t.title.clone()` 或 `t.content.clone()`，不含元数据
- app.rs:164-198 — `submit_input()` 编辑模式下 `parse_pipe_*` 重解析，元数据字段无法从纯 title/content 中还原
- todo.rs:27-36 — Priority Display 输出 "P0"-"P3"，被 parse_pipe_todo 识别
- parser.rs:64-129 — parse_pipe_todo 识别 "P0-P3" / "#tag" / "due:" / "project:" 语法

## Decisions (with rationale)
| # | Decision | Rationale |
|---|----------|-----------|
| 1 | 改 start_edit() 而非 submit_input() | prefill 是源头，改了所有编辑路径都受益 |
| 2 | 用 Vec<String> 拼接再 join(" ") | 保证 token 级 round-trip，无需处理边界空格 |
| 3 | Priority 用 `t.priority.to_string()` → "P0"等 | Display impl 输出恰好被 parser 识别 |

## Scope IN
- 改 `tui/src/app.rs:start_edit()`
- 构造完整 prefill text（含 priority、tags、due_date、source、mood、project）
- Rust 编译通过
- 验证 round-trip

## Scope OUT (Must NOT have)
- 不改其他文件（handler.rs、ui.rs、components/）
- 不改 Rust CLI（src/）或 Qt GUI（qt/）
- 不改 parser 或 db 逻辑
- 不改测试文件（已有单元测试足够）

## Open questions
None.

## Approval gate
status: approved — 用户口头批准 "开始吧"
Approach: 改 start_edit() prefill text
