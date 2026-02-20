//! Rinux Standard Library
//!
//! Common data structures and utilities for the kernel.

#![no_std]

pub mod list;
pub mod math;
pub mod string;

/// Version information
pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;

pub fn version_string() -> &'static str {
    "Rinux 0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert_eq!(VERSION_MAJOR, 0);
        assert_eq!(VERSION_MINOR, 1);
        assert_eq!(VERSION_PATCH, 0);
    }

    #[test]
    fn test_version_string() {
        let version = version_string();
        assert_eq!(version, "Rinux 0.1.0");
        assert!(version.starts_with("Rinux"));
        assert!(version.contains("0.1.0"));
    }

    #[test]
    fn test_version_string_static() {
        // Verify it returns a static reference
        let v1 = version_string();
        let v2 = version_string();
        assert_eq!(v1, v2);
    }
}
