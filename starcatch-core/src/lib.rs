pub mod db;
pub mod models;
pub mod parser;

pub use db::*;
pub use models::*;
pub use parser::*;

/// Safely truncate a string to at most `max_bytes` bytes, on a character
/// boundary. If truncation occurs, appends "..." (included in max_bytes).
/// Always yields valid UTF-8 — walks backward to find a char boundary,
/// avoiding the byte-slicing panics that raw `&s[..N]` suffers from with
/// multi-byte UTF-8 (CJK, emoji, etc.).
pub fn safe_truncate_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        let cutoff = max_bytes.saturating_sub(3);
        let mut boundary = cutoff;
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        format!("{}...", &s[..boundary])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_truncate_bytes_short() {
        assert_eq!(safe_truncate_bytes("hello", 10), "hello");
        assert_eq!(safe_truncate_bytes("", 5), "");
    }

    #[test]
    fn test_safe_truncate_bytes_long_ascii() {
        assert_eq!(safe_truncate_bytes("hello world!", 8), "hello...");
        assert_eq!(safe_truncate_bytes("abc", 3), "abc");
        assert_eq!(safe_truncate_bytes("abcd", 3), "...");
    }

    #[test]
    fn test_safe_truncate_bytes_unicode_no_panic() {
        let s = "中".repeat(20);
        let result = safe_truncate_bytes(&s, 30);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 30);
    }

    #[test]
    fn test_safe_truncate_bytes_mixed() {
        let s = "hello世界!";
        let result = safe_truncate_bytes(s, 8);
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_safe_truncate_bytes_emoji() {
        let s = "😀😀😀😀😀";
        let result = safe_truncate_bytes(s, 15);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 15);
    }

    #[test]
    fn test_safe_truncate_bytes_char_boundary() {
        let s = "a中b";
        assert_eq!(safe_truncate_bytes(s, 4), "a...");
        assert_eq!(safe_truncate_bytes(s, 3), "...");
        assert_eq!(safe_truncate_bytes(s, 5), "a中b");
        assert_eq!(safe_truncate_bytes(s, 8), "a中b");
    }
}
