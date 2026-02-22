//! User/Kernel Space Separation
//!
//! Ring 0 (kernel) and Ring 3 (user) privilege level support.

use core::sync::atomic::{AtomicU8, Ordering};

/// Privilege level (CPU ring)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PrivilegeLevel {
    /// Ring 0 - Kernel mode
    Kernel = 0,
    /// Ring 1 - Reserved
    Reserved1 = 1,
    /// Ring 2 - Reserved
    Reserved2 = 2,
    /// Ring 3 - User mode
    User = 3,
}

impl PrivilegeLevel {
    /// Create from raw ring value
    pub const fn from_raw(ring: u8) -> Option<Self> {
        match ring {
            0 => Some(PrivilegeLevel::Kernel),
            1 => Some(PrivilegeLevel::Reserved1),
            2 => Some(PrivilegeLevel::Reserved2),
            3 => Some(PrivilegeLevel::User),
            _ => None,
        }
    }

    /// Get raw ring value
    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Check if this is kernel privilege
    pub const fn is_kernel(&self) -> bool {
        matches!(self, PrivilegeLevel::Kernel)
    }

    /// Check if this is user privilege
    pub const fn is_user(&self) -> bool {
        matches!(self, PrivilegeLevel::User)
    }
}

/// Per-CPU current privilege level
///
/// In a real implementation, this would be per-CPU.
/// For now, using a simple global with atomic operations.
static CURRENT_PRIVILEGE: AtomicU8 = AtomicU8::new(PrivilegeLevel::Kernel as u8);

/// Get current privilege level
pub fn current_privilege() -> PrivilegeLevel {
    let ring = CURRENT_PRIVILEGE.load(Ordering::Acquire);
    PrivilegeLevel::from_raw(ring).unwrap_or(PrivilegeLevel::Kernel)
}

/// Set current privilege level (internal use only)
fn set_privilege(level: PrivilegeLevel) {
    CURRENT_PRIVILEGE.store(level.as_u8(), Ordering::Release);
}

/// Check if currently in kernel mode
pub fn in_kernel_mode() -> bool {
    current_privilege().is_kernel()
}

/// Check if currently in user mode
pub fn in_user_mode() -> bool {
    current_privilege().is_user()
}

/// Require kernel privilege
///
/// # Errors
///
/// Returns error if not in kernel mode
pub fn require_kernel() -> Result<(), &'static str> {
    if in_kernel_mode() {
        Ok(())
    } else {
        Err("Operation requires kernel privilege")
    }
}

/// Require user privilege
///
/// # Errors
///
/// Returns error if not in user mode
pub fn require_user() -> Result<(), &'static str> {
    if in_user_mode() {
        Ok(())
    } else {
        Err("Operation requires user privilege")
    }
}

/// Context for privilege level transitions
#[derive(Debug, Clone, Copy)]
pub struct PrivilegeContext {
    /// Previous privilege level
    pub prev_level: PrivilegeLevel,
}

impl PrivilegeContext {
    /// Create a new context with current privilege
    pub fn current() -> Self {
        PrivilegeContext {
            prev_level: current_privilege(),
        }
    }

    /// Restore previous privilege level
    pub fn restore(&self) {
        set_privilege(self.prev_level);
    }
}

/// Enter userspace (Ring 3)
///
/// # Safety
///
/// Caller must ensure that user mode state is properly set up,
/// including stack, instruction pointer, and segment selectors.
///
/// This is a simplified version. Real implementation would use
/// iret or sysret instructions to transition to Ring 3.
pub unsafe fn enter_userspace() -> PrivilegeContext {
    let ctx = PrivilegeContext::current();
    set_privilege(PrivilegeLevel::User);
    ctx
}

/// Return to kernel mode (Ring 0)
///
/// Called when returning from user mode via syscall or interrupt.
pub fn return_to_kernel() -> PrivilegeContext {
    let ctx = PrivilegeContext::current();
    set_privilege(PrivilegeLevel::Kernel);
    ctx
}

/// Execute a function with temporary privilege level
///
/// # Safety
///
/// Caller must ensure the operation at the given privilege level is safe.
pub unsafe fn with_privilege<F, R>(level: PrivilegeLevel, f: F) -> R
where
    F: FnOnce() -> R,
{
    let ctx = PrivilegeContext::current();
    set_privilege(level);
    let result = f();
    ctx.restore();
    result
}

/// Execute a function in kernel mode
///
/// # Safety
///
/// Caller must ensure the kernel operation is safe.
pub unsafe fn with_kernel_privilege<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    with_privilege(PrivilegeLevel::Kernel, f)
}

/// Execute a function in user mode
///
/// # Safety
///
/// Caller must ensure the user operation is safe and user state is valid.
pub unsafe fn with_user_privilege<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    with_privilege(PrivilegeLevel::User, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_level_from_raw() {
        assert_eq!(PrivilegeLevel::from_raw(0), Some(PrivilegeLevel::Kernel));
        assert_eq!(PrivilegeLevel::from_raw(3), Some(PrivilegeLevel::User));
        assert_eq!(PrivilegeLevel::from_raw(4), None);
    }

    #[test]
    fn test_privilege_level_as_u8() {
        assert_eq!(PrivilegeLevel::Kernel.as_u8(), 0);
        assert_eq!(PrivilegeLevel::User.as_u8(), 3);
    }

    #[test]
    fn test_privilege_checks() {
        assert!(PrivilegeLevel::Kernel.is_kernel());
        assert!(!PrivilegeLevel::Kernel.is_user());
        assert!(PrivilegeLevel::User.is_user());
        assert!(!PrivilegeLevel::User.is_kernel());
    }

    #[test]
    fn test_current_privilege() {
        // Should start in kernel mode
        assert!(in_kernel_mode());
        assert!(!in_user_mode());
    }

    #[test]
    fn test_privilege_context() {
        let ctx = PrivilegeContext::current();
        assert_eq!(ctx.prev_level, PrivilegeLevel::Kernel);
    }

    #[test]
    fn test_require_kernel() {
        // Should succeed in kernel mode
        assert!(require_kernel().is_ok());
    }

    #[test]
    fn test_with_privilege() {
        unsafe {
            let result = with_user_privilege(|| {
                assert!(in_user_mode());
                42
            });
            assert_eq!(result, 42);
            // Should be back in kernel mode
            assert!(in_kernel_mode());
        }
    }
}
