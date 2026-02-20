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
        assert_eq!(version_string(), "Rinux 0.1.0");
    }
}
