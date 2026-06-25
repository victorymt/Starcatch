---
name: starcatch
description: Use when the user wants to capture ideas, manage todos, write daily logs, or interact with the Starcatch (星捕) CLI tool. Triggers on: "starcatch", "星捕", "capture this", "add todo", "log this", "save this idea", "记一下", "待办", "灵感", "日志".
---

# Starcatch (星捕) — Catch Your Starlight Ideas

## Overview

Starcatch is a local CLI personal knowledge capture tool. It stores data in a SQLite database at `~/.local/share/starcatch/starcatch.db`. Use it to quickly capture todos, ideas, and daily logs from the terminal — or via pipe from any other tool.

**Core principle:** When the user mentions something worth keeping — a task to do, an idea that sparked, or a thought to remember — proactively offer to capture it with starcatch. Don't wait for them to say "starcatch."

## When to Use

**Always use starcatch when the user:**
- Mentions a task they need to do → `starcatch todo add`
- Shares an idea or inspiration → `starcatch idea add`
- Reflects on something or shares a mood → `starcatch log add`
- Asks "what do I have to do?" → `starcatch todo list`
- Wants to review recent thoughts → `starcatch idea list` / `starcatch log list`
- Pipes output from another command → `starcatch pipe`

**Don't use starcatch when:**
- The user is just chatting, not capturing
- The content is ephemeral (a one-off question)
- The user explicitly says "don't save this"

## Quick Reference

```
Command                          What it does
─────────────────────────────────────────────────────────────
starcatch todo add "title"       Add a todo
starcatch todo list              List active todos (pending + done)
starcatch todo done <id>         Mark todo as done
starcatch todo archive <id>      Archive a todo
starcatch idea add "title"       Capture an idea
starcatch idea list              List recent ideas (7 days)
starcatch log add "content"      Write a daily log entry
starcatch log list               List today's logs
starcatch pipe todo              Capture piped stdin as a todo
starcatch pipe idea              Capture piped stdin as an idea
starcatch pipe log               Capture piped stdin as a log
```

## Commands In Detail

### Todo (`starcatch todo`)

Manage tasks with priority levels, due dates, tags, and projects.

**Add a todo:**
```bash
starcatch todo add "Fix the login bug"                          # Basic
starcatch todo add "Deploy v2.0" -p P0 -d "明天"                # Urgent, due tomorrow
starcatch todo add "Refactor auth" -p P1 -t "tech,backend"      # With tags
starcatch todo add "Design API" -p P2 -P "project-x"            # With project
starcatch todo add "Write docs" -d "2026-06-30"                 # With date
```

| Flag | Description |
|------|-------------|
| `-p, --priority` | P0 🔴 urgent, P1 🟡 important, P2 🟢 normal (default), P3 ⚪ low |
| `-d, --desc` | Longer description |
| `--due` | Due date: `YYYY-MM-DD`, `今天`, `明天`, `3天`, `next monday`, `下周一` |
| `-t, --tag` | Comma-separated tags |
| `-P, --project` | Project name |

**List todos:**
```bash
starcatch todo list                 # Active: pending + done
starcatch todo list --pending       # Only pending
starcatch todo list --done          # Only done
starcatch todo list --all           # Everything including archived
starcatch todo list -t "backend"    # Filter by tag
```

**Update status:**
```bash
starcatch todo done <id>            # Mark complete
starcatch todo archive <id>         # Move to archive
```

### Idea (`starcatch idea`)

Capture sparks of inspiration before they vanish.

```bash
starcatch idea add "Build a CLI for personal knowledge mgmt"            # Title only
starcatch idea add "AI agent mesh" -c "Decentralized agent routing..."  # With content
starcatch idea add "New arch" -s "reading Clean Architecture" -t "tech" # Source + tags
```

| Flag | Description |
|------|-------------|
| `-c, --content` | Extended body text |
| `-s, --source` | Where the idea came from |
| `-t, --tag` | Comma-separated tags |

```bash
starcatch idea list          # Last 7 days
starcatch idea list -d 30    # Last 30 days
```

### Log (`starcatch log`)

Daily journal entries with optional mood tracking.

```bash
starcatch log add "Finished the auth module rewrite"                # Plain
starcatch log add "Deploy went smoothly" -m "happy"                 # With mood
starcatch log add "Debugged for 3 hours" -m "tired" -t "debugging"  # Mood + tags
```

| Flag | Description |
|------|-------------|
| `-m, --mood` | Mood: happy, sad, excited, tired, anxious, calm, etc. |
| `-t, --tag` | Comma-separated tags |

```bash
starcatch log list        # Today
starcatch log list -d 7   # Last 7 days
```

### Pipe (`starcatch pipe`)

Reads stdin and captures it as a todo, idea, or log entry. Useful for shell pipelines and automation.

```bash
echo "Fix the CI pipeline" | starcatch pipe todo
cat meeting_notes.txt | starcatch pipe idea
echo "Day ended with all tests green" | starcatch pipe log
```

**Critical limitation: pipe has NO flags.** You cannot set priority, tags, due date, project, mood, or source via pipe. Everything gets hardcoded defaults:

| pipe type | What input becomes | Defaults (locked) |
|-----------|-------------------|--------------------|
| `todo` | Title | P2, no tags, no due date, no project, no description |
| `idea` | Title | No source, no tags, no content body |
| `log` | Content | No mood, no tags |

**Multiline input:** reads ALL of stdin until EOF, then trims whitespace. The entire input becomes a single entry — it does NOT create one entry per line.

```bash
# This creates ONE todo with a multiline title:
printf "Line 1\nLine 2\nLine 3" | starcatch pipe todo
# → Todo: "Line 1\nLine 2\nLine 3" (one entry)
```

**When to use `pipe` vs `add`:**

```
Situation                                          Use
──────────────────────────────────────────────────────────
One-liner, metadata doesn't matter                 pipe ✅
Shell pipeline / script automation                 pipe ✅
Needs priority, tags, or due date                  add  ✅
Needs mood or source                               add  ✅
Multi-line content that should be one entry        pipe ✅
Multiple entries from a file                       loop `add` per line
```

**Workaround for bulk import with pipe:**
```bash
# One entry per line — use a while loop with `add`, not `pipe`:
while IFS= read -r line; do
  [ -n "$line" ] && starcatch todo add "$line" -t "imported"
done < tasks.txt
```

## Natural Date Parsing

Starcatch understands dates in both English and Chinese:

| Input | Result |
|-------|--------|
| `2026-06-30` | Exact date |
| `today` / `今天` | Today |
| `tomorrow` / `明天` | Tomorrow |
| `3` / `3天` / `3 days` | 3 days from now |
| `next monday` | Next Monday |
| `下周一` | Next Monday |
| `本周三` | This Wednesday |
| `yesterday` / `昨天` | Yesterday |

## How Claude Should Use Starcatch

### Pattern 1: Proactive Capture

When the user mentions something actionable, reflective, or inspirational, ask briefly or just capture it:

```
User: "I need to update the SSL cert before it expires next week"
→ starcatch todo add "Update SSL certificate" -p P0 -d "下周"
```

```
User: "What if we used a content-addressable store instead of a relational DB?"
→ starcatch idea add "Content-addressable store instead of RDB" -s "user brainstorm"
```

```
User: "Today was productive — shipped 3 features"
→ starcatch log add "Shipped 3 features" -m "happy" -t "shipping"
```

**But don't be annoying.** If the user is clearly just thinking out loud or the item is trivial, skip it. Use judgment. When in doubt, ask: "Save this to starcatch?"

### Pattern 2: Review on Request

```
User: "What's on my plate?"
→ starcatch todo list

User: "What ideas did I have this month?"
→ starcatch idea list -d 30

User: "How was my week?"
→ starcatch log list -d 7
```

### Pattern 3: After Task Completion

When you finish a task the user gave you, and it was tracked in starcatch:

```bash
starcatch todo done <id>
```

### Pattern 4: Capture Context After Sessions

When wrapping up a productive session, offer to capture:
- Open loops as todos
- Insights as ideas
- The session summary as a log entry

### Pattern 5: Pipe for Bulk Capture

When generating output that should become tasks:

```bash
# After code review finds issues:
echo "Fix XSS in login.ts" | starcatch pipe todo
echo "Add rate limiting to API" | starcatch pipe todo
```

## Priority System

| Level | Icon | Meaning | When to Use |
|-------|------|---------|-------------|
| P0 | 🔴 | Urgent | Drop everything. Fire. Deadline imminent. |
| P1 | 🟡 | Important | Must do this week. Blocks other work. |
| P2 | 🟢 | Normal | Should do. Default for most things. |
| P3 | ⚪ | Low | Nice to have. Someday/maybe. |

## Database Location

Default: `~/.local/share/starcatch/starcatch.db`

Override with `-D`:
```bash
starcatch -D /path/to/custom.db todo list
```

## Common Workflows

### Morning Check-in
```bash
starcatch todo list --pending    # What's on my plate?
starcatch log list -d 1          # What happened yesterday?
```

### Evening Wrap-up
```bash
starcatch log add "Finished X, started Y" -m "calm"
starcatch todo list --done       # See what got done
```

### Brain Dump Session
```bash
starcatch idea add "..." -s "brain dump"
starcatch idea add "..." -s "brain dump"
# ... as many as needed
starcatch idea list              # Review the haul
```

### Project Organization
```bash
starcatch todo add "..." -P "project-alpha" -t "frontend"
starcatch todo list -t "frontend"  # See all frontend tasks
```

## Red Flags

- **Don't use starcatch for passwords, tokens, or secrets.** It's a local SQLite file with no encryption.
- **Don't capture the user's every sentence.** Be selective. Capture value, not noise.
- **Don't guess IDs.** When marking done/archive, always list first to confirm the ID.
- **Don't create todos for tasks Claude is doing right now.** Capture only future/recurring work.
