use std::sync::LazyLock;

use chrono::{Datelike, Duration, Utc, Weekday};
use regex::Regex;

use crate::models::Priority;

// ─── Data types ──────────────────────────────────────────────────────

pub struct ParsedPipeTodo {
    pub title: String,
    pub priority: Priority,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
}

pub struct ParsedPipeIdea {
    pub title: String,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
}

pub struct ParsedPipeLog {
    pub content: String,
    pub mood: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
}

// ─── Punctuation helpers ─────────────────────────────────────────────

fn is_trailing_punct(c: char) -> bool {
    c.is_ascii_punctuation()
        || c == '\u{3001}' // 、
        || c == '\u{3002}' // 。
        || c == '\u{FF0C}' // ，
        || c == '\u{FF1B}' // ；
        || c == '\u{FF1A}' // ：
        || c == '\u{FF01}' // ！
        || c == '\u{FF1F}' // ？
        || c == '\u{FF09}' // ）
        || c == '\u{FF3D}' // 】
        || c == '\u{300D}' // 》
}

fn trim_trailing_punct(s: &str) -> &str {
    let mut end = s.len();
    for (byte_offset, c) in s.char_indices().rev() {
        if is_trailing_punct(c) {
            end = byte_offset;
        } else {
            break;
        }
    }
    &s[..end]
}

// ─── Separator for edit-mode ────────────────────────────────────────
/// When present in the input, everything to the left of " | " is treated
/// as the literal title/content (no metadata parsing), and everything to
/// the right is metadata-only (no title token collection).
const SEPARATOR: &str = " | ";

fn split_separator(raw: &str) -> Option<(&str, &str)> {
    let pos = raw.find(SEPARATOR)?;
    Some((&raw[..pos], raw[pos + SEPARATOR.len()..].trim()))
}

// ─── Meta-only parsers (for the right side of " | ") ────────────────

fn parse_todo_meta(raw: &str) -> (Priority, Option<String>, Vec<String>, Option<String>) {
    let mut priority = Priority::P2;
    let mut due_date = None;
    let mut tags: Vec<String> = Vec::new();
    let mut project = None;

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        match token {
            "P0" => priority = Priority::P0,
            "P1" => priority = Priority::P1,
            "P3" => priority = Priority::P3,
            "P2" => priority = Priority::P2,
            _ => {
                if let Some(val) = token.strip_prefix("due:").or_else(|| token.strip_prefix("due：")) {
                    if !val.is_empty() {
                        due_date = Some(parse_natural_date(val).unwrap_or_else(|| val.to_string()));
                    } else if i + 1 < tokens.len() {
                        i += 1;
                        due_date = Some(parse_natural_date(tokens[i]).unwrap_or_else(|| tokens[i].to_string()));
                    }
                } else if let Some(val) = token.strip_prefix("project:").or_else(|| token.strip_prefix("project：")) {
                    if !val.is_empty() {
                        project = Some(val.to_string());
                    } else if i + 1 < tokens.len() {
                        i += 1;
                        project = Some(tokens[i].to_string());
                    }
                } else if let Some(tag) = token.strip_prefix('#') {
                    let tag = trim_trailing_punct(tag.trim());
                    if !tag.is_empty() {
                        tags.push(tag.to_string());
                    }
                }
            }
        }
        i += 1;
    }
    (priority, due_date, tags, project)
}

fn parse_idea_meta(raw: &str) -> (Option<String>, Vec<String>, Option<String>) {
    let mut source = None;
    let mut tags: Vec<String> = Vec::new();
    let mut project = None;

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        if let Some(val) = token.strip_prefix("source:").or_else(|| token.strip_prefix("source：")) {
            if !val.is_empty() {
                source = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                source = Some(tokens[i].to_string());
            }
        } else if let Some(val) = token.strip_prefix("project:").or_else(|| token.strip_prefix("project：")) {
            if !val.is_empty() {
                project = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                project = Some(tokens[i].to_string());
            }
        } else if let Some(tag) = token.strip_prefix('#') {
            let tag = trim_trailing_punct(tag.trim());
            if !tag.is_empty() {
                tags.push(tag.to_string());
            }
        }
        i += 1;
    }
    (source, tags, project)
}

fn parse_log_meta(raw: &str) -> (Option<String>, Vec<String>, Option<String>) {
    let mut mood = None;
    let mut tags: Vec<String> = Vec::new();
    let mut project = None;

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        if let Some(val) = token.strip_prefix("mood:").or_else(|| token.strip_prefix("mood：")) {
            if !val.is_empty() {
                mood = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                mood = Some(tokens[i].to_string());
            }
        } else if let Some(val) = token.strip_prefix("project:").or_else(|| token.strip_prefix("project：")) {
            if !val.is_empty() {
                project = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                project = Some(tokens[i].to_string());
            }
        } else if let Some(tag) = token.strip_prefix('#') {
            let tag = trim_trailing_punct(tag.trim());
            if !tag.is_empty() {
                tags.push(tag.to_string());
            }
        }
        i += 1;
    }
    (mood, tags, project)
}

// ─── Pipe input parsers ──────────────────────────────────────────────

/// Parse a quick-input (or edit) string for pipe mode.
/// If " | " is present, splits into title (left) and metadata (right).
/// Extracts P0-P3 priority, due:/due： dates, #tags, project: — the rest becomes the title.
pub fn parse_pipe_todo(raw: &str) -> ParsedPipeTodo {
    // Edit-mode path: explicit separator
    if let Some((title, meta)) = split_separator(raw) {
        let (priority, due_date, tags, project) = if meta.is_empty() {
            (Priority::P2, None, vec![], None)
        } else {
            parse_todo_meta(meta)
        };
        return ParsedPipeTodo { title: title.to_string(), priority, due_date, tags, project };
    }

    // Quick-input path (no separator)
    let mut priority = Priority::P2;
    let mut due_date = None;
    let mut tags: Vec<String> = Vec::new();
    let mut project = None;
    let mut title_parts: Vec<&str> = Vec::new();

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];

        // Priority keywords
        if token == "P0" {
            priority = Priority::P0;
        } else if token == "P1" {
            priority = Priority::P1;
        } else if token == "P3" {
            priority = Priority::P3;
        } else if token == "P2" {
            priority = Priority::P2;
        }
        // due: / due： prefix — value may be in same token or the next
        else if let Some(val) = token.strip_prefix("due:").or_else(|| token.strip_prefix("due：")) {
            let val = val.trim();
            if !val.is_empty() {
                due_date = Some(parse_natural_date(val).unwrap_or_else(|| val.to_string()));
            } else if i + 1 < tokens.len() {
                i += 1;
                let next = tokens[i];
                due_date = Some(parse_natural_date(next).unwrap_or_else(|| next.to_string()));
            }
        }
        // project: prefix — value may be in same token or the next
        else if let Some(val) = token.strip_prefix("project:").or_else(|| token.strip_prefix("project：")) {
            let val = val.trim();
            if !val.is_empty() {
                project = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                project = Some(tokens[i].to_string());
            }
        }
        // #tag — strip leading # then trim trailing punctuation
        else if let Some(tag) = token.strip_prefix('#') {
            let tag = trim_trailing_punct(tag.trim());
            if !tag.is_empty() {
                tags.push(tag.to_string());
            }
        }
        // Plain title word
        else {
            title_parts.push(token);
        }

        i += 1;
    }

    let title = if title_parts.is_empty() {
        raw.to_string()
    } else {
        title_parts.join(" ")
    };

    ParsedPipeTodo { title, priority, due_date, tags, project }
}

/// Parse pipe input for idea: extracts #tags, source:, project: — the rest becomes title.
pub fn parse_pipe_idea(raw: &str) -> ParsedPipeIdea {
    // Edit-mode path: explicit separator
    if let Some((title, meta)) = split_separator(raw) {
        let (source, tags, project) = if meta.is_empty() {
            (None, vec![], None)
        } else {
            parse_idea_meta(meta)
        };
        return ParsedPipeIdea { title: title.to_string(), source, tags, project };
    }

    // Quick-input path (no separator)
    let mut source = None;
    let mut tags: Vec<String> = Vec::new();
    let mut project = None;
    let mut title_parts: Vec<&str> = Vec::new();

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        if let Some(val) = token.strip_prefix("source:").or_else(|| token.strip_prefix("source：")) {
            let val = val.trim();
            if !val.is_empty() {
                source = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                source = Some(tokens[i].to_string());
            }
        } else if let Some(val) = token.strip_prefix("project:").or_else(|| token.strip_prefix("project：")) {
            let val = val.trim();
            if !val.is_empty() {
                project = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                project = Some(tokens[i].to_string());
            }
        } else if let Some(tag) = token.strip_prefix('#') {
            let tag = trim_trailing_punct(tag.trim());
            if !tag.is_empty() {
                tags.push(tag.to_string());
            }
        } else {
            title_parts.push(token);
        }
        i += 1;
    }

    let title = if title_parts.is_empty() { raw.to_string() } else { title_parts.join(" ") };
    ParsedPipeIdea { title, source, tags, project }
}

/// Parse pipe input for log: extracts #tags, mood:, project: — the rest becomes content.
pub fn parse_pipe_log(raw: &str) -> ParsedPipeLog {
    // Edit-mode path: explicit separator
    if let Some((content, meta)) = split_separator(raw) {
        let (mood, tags, project) = if meta.is_empty() {
            (None, vec![], None)
        } else {
            parse_log_meta(meta)
        };
        return ParsedPipeLog { content: content.to_string(), mood, tags, project };
    }

    // Quick-input path (no separator)
    let mut mood = None;
    let mut tags: Vec<String> = Vec::new();
    let mut project = None;
    let mut content_parts: Vec<&str> = Vec::new();

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        if let Some(val) = token.strip_prefix("mood:").or_else(|| token.strip_prefix("mood：")) {
            let val = val.trim();
            if !val.is_empty() {
                mood = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                mood = Some(tokens[i].to_string());
            }
        } else if let Some(val) = token.strip_prefix("project:").or_else(|| token.strip_prefix("project：")) {
            let val = val.trim();
            if !val.is_empty() {
                project = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                project = Some(tokens[i].to_string());
            }
        } else if let Some(tag) = token.strip_prefix('#') {
            let tag = trim_trailing_punct(tag.trim());
            if !tag.is_empty() {
                tags.push(tag.to_string());
            }
        } else {
            content_parts.push(token);
        }
        i += 1;
    }

    let content = if content_parts.is_empty() { raw.to_string() } else { content_parts.join(" ") };
    ParsedPipeLog { content, mood, tags, project }
}

// ─── Natural date parser ─────────────────────────────────────────────

pub fn parse_natural_date(text: &str) -> Option<String> {
    static DATE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());
    static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)\s*(天|d|day|days)?(后|後| later)?$").unwrap());
    static NEXT_EN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^next\s+(\w+)").unwrap());
    static NEXT_ZH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^下(?:周|星期|礼拜)?(.)").unwrap());
    static THIS_ZH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:这|本|这周|本周|这星期|本星期)(?:周|星期|礼拜)?(.)").unwrap());

    let today = Utc::now().date_naive();
    let t = text.trim();

    // Already yyyy-MM-dd
    if DATE_RE.is_match(t) {
        return Some(t.to_string());
    }

    // Numeric: N (days later)
    if let Some(cap) = NUM_RE.captures(t) {
        let n: i64 = cap[1].parse().unwrap_or(0);
        return Some((today + Duration::days(n)).format("%Y-%m-%d").to_string());
    }

    // Day-of-week maps
    let dow_en: Vec<(&str, Weekday)> = vec![
        ("mon", Weekday::Mon), ("monday", Weekday::Mon),
        ("tue", Weekday::Tue), ("tuesday", Weekday::Tue),
        ("wed", Weekday::Wed), ("wednesday", Weekday::Wed),
        ("thu", Weekday::Thu), ("thursday", Weekday::Thu),
        ("fri", Weekday::Fri), ("friday", Weekday::Fri),
        ("sat", Weekday::Sat), ("saturday", Weekday::Sat),
        ("sun", Weekday::Sun), ("sunday", Weekday::Sun),
    ];
    let dow_zh: Vec<(&str, Weekday)> = vec![
        ("一", Weekday::Mon), ("二", Weekday::Tue), ("三", Weekday::Wed),
        ("四", Weekday::Thu), ("五", Weekday::Fri), ("六", Weekday::Sat),
        ("日", Weekday::Sun), ("天", Weekday::Sun),
    ];

    // Absolute keywords
    match t {
        "今天" | "today" => return Some(today.format("%Y-%m-%d").to_string()),
        "明天" | "tomorrow" => return Some((today + Duration::days(1)).format("%Y-%m-%d").to_string()),
        "后天" | "後天" => return Some((today + Duration::days(2)).format("%Y-%m-%d").to_string()),
        "大后天" | "大後天" => return Some((today + Duration::days(3)).format("%Y-%m-%d").to_string()),
        "昨天" | "yesterday" => return Some((today + Duration::days(-1)).format("%Y-%m-%d").to_string()),
        "下周" | "下週" | "next week" => {
            let delta = 7 - today.weekday().num_days_from_monday() as i64;
            if delta <= 0 { return Some((today + Duration::days(delta + 7)).format("%Y-%m-%d").to_string()); }
            return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
        }
        _ => {}
    }

    // "next <weekday>"
    if let Some(cap) = NEXT_EN_RE.captures(t) {
        let w = cap[1].to_lowercase();
        for (key, wd) in &dow_en {
            if w == *key {
                let target = wd.num_days_from_monday() as i64;
                let cur = today.weekday().num_days_from_monday() as i64;
                let mut delta = target - cur;
                if delta <= 0 { delta += 7; }
                return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
            }
        }
    }

    // "下<星期X>" / "下周<X>"
    if let Some(cap) = NEXT_ZH_RE.captures(t) {
        let ch = &cap[1];
        for (key, wd) in &dow_zh {
            if ch == *key {
                let target = wd.num_days_from_monday() as i64;
                let cur = today.weekday().num_days_from_monday() as i64;
                let mut delta = target - cur;
                if delta <= 0 { delta += 7; }
                return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
            }
        }
    }

    // "本周X" / "这周X"
    if let Some(cap) = THIS_ZH_RE.captures(t) {
        let ch = &cap[1];
        for (key, wd) in &dow_zh {
            if ch == *key {
                let target = wd.num_days_from_monday() as i64;
                let cur = today.weekday().num_days_from_monday() as i64;
                let delta = target - cur;
                return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        // Ensures static LazyLock values are initialized in test context
    }

    // ── parse_natural_date tests ──

    #[test]
    fn today() {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(parse_natural_date("today"), Some(today.clone()));
        assert_eq!(parse_natural_date("今天"), Some(today));
    }

    #[test]
    fn tomorrow() {
        let tomorrow = (Utc::now() + Duration::days(1)).format("%Y-%m-%d").to_string();
        assert_eq!(parse_natural_date("tomorrow"), Some(tomorrow.clone()));
        assert_eq!(parse_natural_date("明天"), Some(tomorrow));
    }

    #[test]
    fn numeric_days() {
        let expected = (Utc::now() + Duration::days(3)).format("%Y-%m-%d").to_string();
        assert_eq!(parse_natural_date("3天"), Some(expected.clone()));
        assert_eq!(parse_natural_date("3d"), Some(expected));
    }

    #[test]
    fn iso_date() {
        assert_eq!(parse_natural_date("2025-01-15"), Some("2025-01-15".to_string()));
    }

    #[test]
    fn next_weekday() {
        // Just check it returns Some value
        assert!(parse_natural_date("next monday").is_some());
        assert!(parse_natural_date("下周一").is_some());
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(parse_natural_date("foobar"), None);
    }

    // ── parse_pipe_todo tests ──

    #[test]
    fn todo_plain() {
        let r = parse_pipe_todo("买牛奶");
        assert_eq!(r.title, "买牛奶");
        assert_eq!(r.priority, Priority::P2);
    }

    #[test]
    fn todo_with_priority() {
        let r = parse_pipe_todo("P1 重要事情");
        assert_eq!(r.title, "重要事情");
        assert_eq!(r.priority, Priority::P1);
    }

    #[test]
    fn todo_with_due() {
        let r = parse_pipe_todo("完成报告 due:2025-01-15");
        assert_eq!(r.title, "完成报告");
        assert_eq!(r.due_date, Some("2025-01-15".to_string()));
    }

    #[test]
    fn todo_with_tags() {
        let r = parse_pipe_todo("买东西 #shopping #urgent");
        assert_eq!(r.title, "买东西");
        assert_eq!(r.tags, vec!["shopping", "urgent"]);
    }

    #[test]
    fn todo_with_project() {
        let r = parse_pipe_todo("修复bug project:核心系统");
        assert_eq!(r.title, "修复bug");
        assert_eq!(r.project, Some("核心系统".to_string()));
    }

    #[test]
    fn todo_all_features() {
        let r = parse_pipe_todo("P0 紧急项目 #work project:核心系统 due:明天");
        assert_eq!(r.title, "紧急项目");
        assert_eq!(r.priority, Priority::P0);
        assert_eq!(r.tags, vec!["work"]);
        assert_eq!(r.project, Some("核心系统".to_string()));
        assert!(r.due_date.is_some());
    }

    #[test]
    fn todo_only_keywords_uses_raw() {
        let r = parse_pipe_todo("P1");
        assert_eq!(r.title, "P1");
        assert_eq!(r.priority, Priority::P1);
    }

    #[test]
    fn todo_fullwidth_colon() {
        let r = parse_pipe_todo("会议 due：明天");
        assert_eq!(r.title, "会议");
        assert!(r.due_date.is_some());
    }

    // ── parse_pipe_idea tests ──

    #[test]
    fn idea_plain() {
        let r = parse_pipe_idea("一个有趣的想法");
        assert_eq!(r.title, "一个有趣的想法");
    }

    #[test]
    fn idea_with_source() {
        let r = parse_pipe_idea("算法优化 source:论文阅读");
        assert_eq!(r.title, "算法优化");
        assert_eq!(r.source, Some("论文阅读".to_string()));
    }

    #[test]
    fn idea_with_tags() {
        let r = parse_pipe_idea("新功能 #innovation #tech");
        assert_eq!(r.title, "新功能");
        assert_eq!(r.tags, vec!["innovation", "tech"]);
    }

    #[test]
    fn idea_all_features() {
        let r = parse_pipe_idea("好点子 #idea source:读书 project:个人");
        assert_eq!(r.title, "好点子");
        assert_eq!(r.tags, vec!["idea"]);
        assert_eq!(r.source, Some("读书".to_string()));
        assert_eq!(r.project, Some("个人".to_string()));
    }

    // ── parse_pipe_log tests ──

    #[test]
    fn log_plain() {
        let r = parse_pipe_log("今天完成了模块重构");
        assert_eq!(r.content, "今天完成了模块重构");
    }

    #[test]
    fn log_with_mood() {
        let r = parse_pipe_log("工作完成 mood:happy");
        assert_eq!(r.content, "工作完成");
        assert_eq!(r.mood, Some("happy".to_string()));
    }

    #[test]
    fn log_with_tags() {
        let r = parse_pipe_log("完成重构 #dev #rust");
        assert_eq!(r.content, "完成重构");
        assert_eq!(r.tags, vec!["dev", "rust"]);
    }

    #[test]
    fn log_all_features() {
        let r = parse_pipe_log("今天很顺利 mood:happy #work project:refactor");
        assert_eq!(r.content, "今天很顺利");
        assert_eq!(r.mood, Some("happy".to_string()));
        assert_eq!(r.tags, vec!["work"]);
        assert_eq!(r.project, Some("refactor".to_string()));
    }

    // ── Separator (edit-mode) tests ──

    #[test]
    fn todo_separator_preserves_title_verbatim() {
        let r = parse_pipe_todo("买牛奶 | P1 #shopping due:明天");
        assert_eq!(r.title, "买牛奶");
        assert_eq!(r.priority, Priority::P1);
        assert_eq!(r.tags, vec!["shopping"]);
        assert!(r.due_date.is_some());
    }

    #[test]
    fn todo_separator_title_containing_p0_is_safe() {
        // "P0" in title would be eaten by quick-input path — separator preserves it
        let r = parse_pipe_todo("Release P0 | P1 #urgent");
        assert_eq!(r.title, "Release P0");
        assert_eq!(r.priority, Priority::P1);
        assert_eq!(r.tags, vec!["urgent"]);
    }

    #[test]
    fn todo_separator_title_containing_due_keyword_is_safe() {
        let r = parse_pipe_todo("meeting due:tomorrow | P0 #work");
        assert_eq!(r.title, "meeting due:tomorrow");
        assert_eq!(r.priority, Priority::P0);
        assert_eq!(r.tags, vec!["work"]);
    }

    #[test]
    fn todo_separator_no_meta() {
        let r = parse_pipe_todo("买牛奶 | ");
        assert_eq!(r.title, "买牛奶");
        assert_eq!(r.priority, Priority::P2);
        assert!(r.tags.is_empty());
        assert!(r.due_date.is_none());
        assert!(r.project.is_none());
    }

    #[test]
    fn todo_separator_meta_only_no_priority() {
        let r = parse_pipe_todo("标题 | #tag project:foo");
        assert_eq!(r.title, "标题");
        assert_eq!(r.priority, Priority::P2);
        assert_eq!(r.tags, vec!["tag"]);
        assert_eq!(r.project, Some("foo".to_string()));
    }

    #[test]
    fn todo_without_separator_still_works() {
        let r = parse_pipe_todo("P1 重要事情");
        assert_eq!(r.title, "重要事情");
        assert_eq!(r.priority, Priority::P1);
    }

    #[test]
    fn idea_separator_preserves_title_verbatim() {
        let r = parse_pipe_idea("好点子 #idea | source:读书 project:个人");
        assert_eq!(r.title, "好点子 #idea");
        assert_eq!(r.source, Some("读书".to_string()));
        assert_eq!(r.project, Some("个人".to_string()));
    }

    #[test]
    fn idea_separator_no_meta() {
        let r = parse_pipe_idea("plain idea | ");
        assert_eq!(r.title, "plain idea");
        assert!(r.source.is_none());
        assert!(r.tags.is_empty());
    }

    #[test]
    fn log_separator_preserves_content_verbatim() {
        let r = parse_pipe_log("今天很顺利 mood:happy | mood:sad #work");
        assert_eq!(r.content, "今天很顺利 mood:happy");
        assert_eq!(r.mood, Some("sad".to_string()));
        assert_eq!(r.tags, vec!["work"]);
    }

    #[test]
    fn log_separator_no_meta() {
        let r = parse_pipe_log("plain log | ");
        assert_eq!(r.content, "plain log");
        assert!(r.mood.is_none());
        assert!(r.tags.is_empty());
    }

    #[test]
    fn quick_input_no_separator_unchanged() {
        // Existing quick-input behavior is unchanged when " | " is absent
        let r = parse_pipe_todo("买东西 #shopping #urgent");
        assert_eq!(r.title, "买东西");
        assert_eq!(r.tags, vec!["shopping", "urgent"]);

        let r = parse_pipe_idea("新功能 #innovation #tech");
        assert_eq!(r.title, "新功能");
        assert_eq!(r.tags, vec!["innovation", "tech"]);

        let r = parse_pipe_log("完成重构 #dev #rust");
        assert_eq!(r.content, "完成重构");
        assert_eq!(r.tags, vec!["dev", "rust"]);
    }
}
