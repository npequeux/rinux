//! Kernel Test Suite
//!
//! Tests for core kernel functionality

pub mod process_tests;

use crate::printkln;

/// Run all kernel tests
pub fn run_all() {
    printkln!("=== Running Kernel Tests ===\n");

    process_tests::run();

    printkln!("\n=== All Tests Complete ===");
}
