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
        assert_eq!(strcmp("hello", "hello"), 0);
        assert_eq!(strcmp("", ""), 0);
    }

    #[test]
    fn test_strcmp_less_than() {
        assert!(strcmp("abc", "abd") < 0);
        assert!(strcmp("", "a") < 0);
        assert!(strcmp("abc", "abcd") < 0);
    }

    #[test]
    fn test_strcmp_greater_than() {
        assert!(strcmp("abd", "abc") > 0);
        assert!(strcmp("a", "") > 0);
        assert!(strcmp("abcd", "abc") > 0);
    }

    #[test]
    fn test_strchr_found() {
        assert_eq!(strchr("hello", 'h'), Some(0));
        assert_eq!(strchr("hello", 'e'), Some(1));
        assert_eq!(strchr("hello", 'l'), Some(2));
        assert_eq!(strchr("hello", 'o'), Some(4));
    }

    #[test]
    fn test_strchr_not_found() {
        assert_eq!(strchr("hello", 'x'), None);
        assert_eq!(strchr("", 'a'), None);
    }
}
