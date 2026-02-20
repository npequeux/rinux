//! String Utilities
//!
//! String manipulation functions.

/// Compare two strings
pub fn strcmp(s1: &str, s2: &str) -> i32 {
    let bytes1 = s1.as_bytes();
    let bytes2 = s2.as_bytes();

    let min_len = bytes1.len().min(bytes2.len());

    for i in 0..min_len {
        if bytes1[i] != bytes2[i] {
            return bytes1[i] as i32 - bytes2[i] as i32;
        }
    }

    bytes1.len() as i32 - bytes2.len() as i32
}

/// Find character in string
pub fn strchr(s: &str, c: char) -> Option<usize> {
    s.chars().position(|ch| ch == c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strcmp_equal() {
<<<<<<< copilot/increase-linux-coverage
        assert_eq!(strcmp("", ""), 0);
        assert_eq!(strcmp("a", "a"), 0);
        assert_eq!(strcmp("hello", "hello"), 0);
        assert_eq!(strcmp("test string", "test string"), 0);
=======
        assert_eq!(strcmp("hello", "hello"), 0);
        assert_eq!(strcmp("", ""), 0);
>>>>>>> master
    }

    #[test]
    fn test_strcmp_less_than() {
        assert!(strcmp("a", "b") < 0);
        assert!(strcmp("abc", "abd") < 0);
        assert!(strcmp("hello", "world") < 0);
        assert!(strcmp("test", "testing") < 0);
        assert!(strcmp("", "a") < 0);
    }

    #[test]
    fn test_strcmp_greater_than() {
        assert!(strcmp("b", "a") > 0);
        assert!(strcmp("abd", "abc") > 0);
        assert!(strcmp("world", "hello") > 0);
        assert!(strcmp("testing", "test") > 0);
        assert!(strcmp("a", "") > 0);
    }

    #[test]
    fn test_strcmp_prefixes() {
        assert!(strcmp("test", "testing") < 0);
        assert!(strcmp("testing", "test") > 0);
        assert!(strcmp("hello", "hello world") < 0);
        assert!(strcmp("hello world", "hello") > 0);
    }

    #[test]
    fn test_strcmp_case_sensitive() {
        assert!(strcmp("Hello", "hello") < 0); // 'H' < 'h'
        assert!(strcmp("HELLO", "hello") < 0);
        assert!(strcmp("hello", "Hello") > 0);
    }

    #[test]
    fn test_strchr_found() {
        assert_eq!(strchr("hello", 'h'), Some(0));
        assert_eq!(strchr("hello", 'e'), Some(1));
        assert_eq!(strchr("hello", 'l'), Some(2));
        assert_eq!(strchr("hello", 'o'), Some(4));
        assert_eq!(strchr("test string", ' '), Some(4));
        assert_eq!(strchr("abc", 'a'), Some(0));
        assert_eq!(strchr("abc", 'b'), Some(1));
        assert_eq!(strchr("abc", 'c'), Some(2));
    }

    #[test]
    fn test_strchr_not_found() {
        assert_eq!(strchr("hello", 'x'), None);
        assert_eq!(strchr("hello", 'H'), None);
        assert_eq!(strchr("test", 'z'), None);
        assert_eq!(strchr("", 'a'), None);
    }

    #[test]
    fn test_strchr_empty_string() {
        assert_eq!(strchr("", 'a'), None);
        assert_eq!(strchr("", '\0'), None);
    }

    #[test]
    fn test_strchr_special_chars() {
        assert_eq!(strchr("hello\n", '\n'), Some(5));
        assert_eq!(strchr("a\tb\tc", '\t'), Some(1));
        assert_eq!(strchr("test!", '!'), Some(4));
    }

    #[test]
    fn test_strchr_unicode() {
        assert_eq!(strchr("hello", 'l'), Some(2));
        // Test basic ASCII chars
        assert_eq!(strchr("café", 'c'), Some(0));
        assert_eq!(strchr("café", 'a'), Some(1));
    }
}
