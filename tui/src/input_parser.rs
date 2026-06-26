use starcatch_core::models::Priority;

/// Parsed todo input from the quick input bar
#[derive(Debug, Default)]
pub struct ParsedTodoInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
}

/// Parsed idea input
#[derive(Debug, Default)]
pub struct ParsedIdeaInput {
    pub title: String,
    pub content: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
}

/// Parsed log input
#[derive(Debug, Default)]
pub struct ParsedLogInput {
    pub content: String,
    pub mood: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
}

/// Parse priority prefix like "P1 " or "P0 "
fn parse_priority(input: &str) -> (Priority, String) {
    let trimmed = input.trim_start();
    // Use char-based check for at least 2 characters
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 2 {
        let prefix: String = chars[..2].iter().collect();
        let upper = prefix.to_uppercase();
        let priority = match upper.as_str() {
            "P0" => Some(Priority::P0),
            "P1" => Some(Priority::P1),
            "P2" => Some(Priority::P2),
            "P3" => Some(Priority::P3),
            _ => None,
        };
        if priority.is_some() && trimmed.len() > 2 && trimmed.as_bytes()[2] == b' ' {
            return (priority.unwrap(), trimmed[3..].trim_start().to_string());
        }
    }
    (Priority::P2, trimmed.to_string())
}

/// Check if a token at the end of text is a known attribute pattern
/// and strip it. Returns (key, value, rest) if found.
enum Stripped {
    Due(String),
    Project(String),
    Tag(String),
    Source(String),
    Mood(String),
}

/// Try to strip one attribute token from the end of text.
/// Processes text right-to-left, checking the last space-separated token.
fn strip_from_end(input: &str) -> Option<(Stripped, String)> {
    let trimmed = input.trim_end();
    if let Some(last_space) = trimmed.rfind(' ') {
        let last = &trimmed[last_space + 1..];
        let rest = trimmed[..last_space].to_string();

        // due:value or 截止:value
        if let Some(val) = last
            .strip_prefix("due:")
            .or_else(|| last.strip_prefix("截止:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Due(val.to_string()), rest));
            }
        }

        // project:value or 项目:value or proj:value
        if let Some(val) = last
            .strip_prefix("project:")
            .or_else(|| last.strip_prefix("项目:"))
            .or_else(|| last.strip_prefix("proj:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Project(val.to_string()), rest));
            }
        }

        // #tag
        if last.starts_with('#') && last.len() > 1 {
            return Some((Stripped::Tag(last[1..].to_string()), rest));
        }

        // source:value or 来源:value or from:value
        if let Some(val) = last
            .strip_prefix("source:")
            .or_else(|| last.strip_prefix("来源:"))
            .or_else(|| last.strip_prefix("from:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Source(val.to_string()), rest));
            }
        }

        // mood:value or 心情:value
        if let Some(val) = last
            .strip_prefix("mood:")
            .or_else(|| last.strip_prefix("心情:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Mood(val.to_string()), rest));
            }
        }
    } else {
        // Single token — check if it's an attribute
        if let Some(val) = input
            .strip_prefix("due:")
            .or_else(|| input.strip_prefix("截止:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Due(val.to_string()), String::new()));
            }
        }
        if let Some(val) = input
            .strip_prefix("project:")
            .or_else(|| input.strip_prefix("项目:"))
            .or_else(|| input.strip_prefix("proj:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Project(val.to_string()), String::new()));
            }
        }
        if input.starts_with('#') && input.len() > 1 {
            return Some((Stripped::Tag(input[1..].to_string()), String::new()));
        }
        if let Some(val) = input
            .strip_prefix("source:")
            .or_else(|| input.strip_prefix("来源:"))
            .or_else(|| input.strip_prefix("from:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Source(val.to_string()), String::new()));
            }
        }
        if let Some(val) = input
            .strip_prefix("mood:")
            .or_else(|| input.strip_prefix("心情:"))
        {
            if !val.is_empty() {
                return Some((Stripped::Mood(val.to_string()), String::new()));
            }
        }
    }
    None
}

pub fn parse_todo_input(input: &str) -> ParsedTodoInput {
    let (priority, mut working) = parse_priority(input);
    let mut result = ParsedTodoInput {
        priority,
        ..Default::default()
    };

    loop {
        match strip_from_end(&working) {
            Some((Stripped::Due(val), r)) => {
                result.due_date = Some(val);
                working = r;
            }
            Some((Stripped::Project(val), r)) => {
                result.project = Some(val);
                working = r;
            }
            Some((Stripped::Tag(val), r)) => {
                result.tags.push(val);
                working = r;
            }
            Some((Stripped::Source(_), r)) | Some((Stripped::Mood(_), r)) => {
                working = r;
            }
            None => break,
        }
    }

    result.tags.reverse();
    result.title = working.trim().to_string();
    result
}

pub fn parse_idea_input(input: &str) -> ParsedIdeaInput {
    let mut result = ParsedIdeaInput::default();
    let mut working = input.to_string();

    loop {
        match strip_from_end(&working) {
            Some((Stripped::Tag(val), r)) => {
                result.tags.push(val);
                working = r;
            }
            Some((Stripped::Project(val), r)) => {
                result.project = Some(val);
                working = r;
            }
            Some((Stripped::Source(val), r)) => {
                result.source = Some(val);
                working = r;
            }
            Some(_) => {
                break;
            }
            None => break,
        }
    }

    result.tags.reverse();
    result.title = working.trim().to_string();
    result
}

pub fn parse_log_input(input: &str) -> ParsedLogInput {
    let mut result = ParsedLogInput::default();
    let mut working = input.to_string();

    loop {
        match strip_from_end(&working) {
            Some((Stripped::Tag(val), r)) => {
                result.tags.push(val);
                working = r;
            }
            Some((Stripped::Project(val), r)) => {
                result.project = Some(val);
                working = r;
            }
            Some((Stripped::Mood(val), r)) => {
                result.mood = Some(val);
                working = r;
            }
            Some(_) => {
                break;
            }
            None => break,
        }
    }

    result.tags.reverse();
    result.content = working.trim().to_string();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_todo_plain() {
        let r = parse_todo_input("买牛奶");
        assert_eq!(r.title, "买牛奶");
        assert_eq!(r.priority, Priority::P2);
    }

    #[test]
    fn parse_todo_with_priority() {
        let r = parse_todo_input("P1 重要事情");
        assert_eq!(r.title, "重要事情");
        assert_eq!(r.priority, Priority::P1);
    }

    #[test]
    fn parse_todo_with_tags() {
        let r = parse_todo_input("买东西 #shopping #urgent");
        assert_eq!(r.title, "买东西");
        assert_eq!(r.tags, vec!["shopping", "urgent"]);
    }

    #[test]
    fn parse_todo_with_due() {
        let r = parse_todo_input("完成报告 due:2025-01-15");
        assert_eq!(r.title, "完成报告");
        assert_eq!(r.due_date, Some("2025-01-15".to_string()));
    }

    #[test]
    fn parse_todo_all() {
        let r = parse_todo_input("P0 紧急项目 #work project:核心系统 due:明天");
        assert_eq!(r.title, "紧急项目");
        assert_eq!(r.priority, Priority::P0);
        assert_eq!(r.tags, vec!["work"]);
        assert_eq!(r.project, Some("核心系统".to_string()));
        assert_eq!(r.due_date, Some("明天".to_string()));
    }

    #[test]
    fn parse_idea_plain() {
        let r = parse_idea_input("一个有趣的想法");
        assert_eq!(r.title, "一个有趣的想法");
    }

    #[test]
    fn parse_idea_with_source() {
        let r = parse_idea_input("算法优化 from:论文阅读");
        assert_eq!(r.title, "算法优化");
        assert_eq!(r.source, Some("论文阅读".to_string()));
    }

    #[test]
    fn parse_log_content() {
        let r = parse_log_input("今天完成了模块重构 #dev");
        assert_eq!(r.content, "今天完成了模块重构");
        assert_eq!(r.tags, vec!["dev"]);
    }

    #[test]
    fn parse_log_with_mood() {
        let r = parse_log_input("工作完成 mood:happy");
        assert_eq!(r.content, "工作完成");
        assert_eq!(r.mood, Some("happy".to_string()));
    }
}
