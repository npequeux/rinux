//! Rinux Standard Library
//!
//! Common data structures and utilities for the kernel.

#![no_std]

pub mod math;
pub mod string;
pub mod list;

/// Version information
pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;

pub fn version_string() -> &'static str {
    "Rinux 0.1.0"
}
