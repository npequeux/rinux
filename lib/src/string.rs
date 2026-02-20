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
