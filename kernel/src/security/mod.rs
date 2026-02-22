//! Security Subsystem
//!
//! Provides security features including privilege levels, capabilities,
//! access control, ASLR, and syscall parameter validation.

pub mod access;
pub mod aslr;
pub mod capabilities;
pub mod privilege;
pub mod validation;

use core::sync::atomic::{AtomicBool, Ordering};

static SECURITY_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the security subsystem
pub fn init() {
    if SECURITY_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    // Initialize ASLR entropy pool
    aslr::init();

    SECURITY_INITIALIZED.store(true, Ordering::Release);
}

/// Check if security subsystem is initialized
pub fn is_initialized() -> bool {
    SECURITY_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
        assert!(is_initialized());
    }
}
