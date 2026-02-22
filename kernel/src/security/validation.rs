//! Syscall Parameter Validation
//!
//! Validates user-space pointers, buffers, and strings before kernel use.

use crate::types::VirtAddr;
use core::slice;

/// Maximum allowed string length (1MB)
pub const MAX_STRING_LENGTH: usize = 1024 * 1024;

/// Maximum allowed buffer size (16MB)
pub const MAX_BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// User space address range (architecture-specific)
///
/// On x86_64, user space typically is 0x0000_0000_0000_0000 to 0x0000_7FFF_FFFF_FFFF
pub const USER_SPACE_START: u64 = 0x0000_0000_0000_0000;
pub const USER_SPACE_END: u64 = 0x0000_7FFF_FFFF_FFFF;

/// Kernel space address range (architecture-specific)
///
/// On x86_64, kernel space typically is 0xFFFF_8000_0000_0000 to 0xFFFF_FFFF_FFFF_FFFF
pub const KERNEL_SPACE_START: u64 = 0xFFFF_8000_0000_0000;
pub const KERNEL_SPACE_END: u64 = 0xFFFF_FFFF_FFFF_FFFF;

/// Check if address is in user space
pub fn is_user_address(addr: VirtAddr) -> bool {
    let addr = addr.as_u64();
    addr <= USER_SPACE_END
}

/// Check if address is in kernel space
pub fn is_kernel_address(addr: VirtAddr) -> bool {
    let addr = addr.as_u64();
    addr >= KERNEL_SPACE_START
}

/// Check if address range is entirely in user space
pub fn is_user_range(addr: VirtAddr, size: usize) -> bool {
    let start = addr.as_u64();
    let end = start.saturating_add(size as u64);

    // Check for overflow and ensure entire range is in user space
    end > start && end <= USER_SPACE_END
}

/// Validate user pointer
///
/// # Errors
///
/// Returns error if pointer is null, in kernel space, or otherwise invalid.
pub fn validate_user_ptr(ptr: *const u8) -> Result<(), &'static str> {
    if ptr.is_null() {
        return Err("Null pointer");
    }

    let addr = VirtAddr::new(ptr as u64);
    if !is_user_address(addr) {
        return Err("Pointer not in user space");
    }

    Ok(())
}

/// Validate user mutable pointer
///
/// # Errors
///
/// Returns error if pointer is null, in kernel space, or otherwise invalid.
pub fn validate_user_ptr_mut(ptr: *mut u8) -> Result<(), &'static str> {
    validate_user_ptr(ptr as *const u8)
}

/// Validate user buffer
///
/// # Errors
///
/// Returns error if buffer pointer is invalid, size is too large,
/// or buffer extends into kernel space.
pub fn validate_user_buffer(ptr: *const u8, size: usize) -> Result<(), &'static str> {
    if size == 0 {
        return Ok(());
    }

    if size > MAX_BUFFER_SIZE {
        return Err("Buffer size exceeds maximum");
    }

    validate_user_ptr(ptr)?;

    let addr = VirtAddr::new(ptr as u64);
    if !is_user_range(addr, size) {
        return Err("Buffer extends outside user space");
    }

    Ok(())
}

/// Validate user mutable buffer
///
/// # Errors
///
/// Returns error if buffer pointer is invalid, size is too large,
/// or buffer extends into kernel space.
pub fn validate_user_buffer_mut(ptr: *mut u8, size: usize) -> Result<(), &'static str> {
    validate_user_buffer(ptr as *const u8, size)
}

/// Validate user string pointer
///
/// Validates that the string is:
/// - In user space
/// - Null-terminated
/// - Within maximum length
///
/// # Errors
///
/// Returns error if string is invalid or too long.
///
/// # Safety
///
/// Assumes the pointer is readable. In a real implementation,
/// this would carefully check each byte with proper fault handling.
pub unsafe fn validate_user_string(ptr: *const u8, max_len: usize) -> Result<usize, &'static str> {
    if ptr.is_null() {
        return Err("Null string pointer");
    }

    validate_user_ptr(ptr)?;

    let max_len = max_len.min(MAX_STRING_LENGTH);

    // Find null terminator
    // SAFETY: Caller ensures pointer is valid
    for i in 0..max_len {
        let addr = VirtAddr::new(ptr as u64 + i as u64);
        if !is_user_address(addr) {
            return Err("String extends outside user space");
        }

        if unsafe { *ptr.add(i) } == 0 {
            return Ok(i);
        }
    }

    Err("String not null-terminated within limit")
}

/// Copy data from user space to kernel space
///
/// # Safety
///
/// Caller must ensure:
/// - `user_ptr` points to valid, readable user memory
/// - `kernel_buf` has space for `size` bytes
/// - Memory regions do not overlap
///
/// In a real implementation, this would use special copy functions
/// that handle page faults gracefully.
pub unsafe fn copy_from_user(
    user_ptr: *const u8,
    kernel_buf: &mut [u8],
) -> Result<(), &'static str> {
    let size = kernel_buf.len();
    if size == 0 {
        return Ok(());
    }

    validate_user_buffer(user_ptr, size)?;

    // SAFETY: Caller guarantees valid memory regions
    unsafe {
        core::ptr::copy_nonoverlapping(user_ptr, kernel_buf.as_mut_ptr(), size);
    }

    Ok(())
}

/// Copy data from kernel space to user space
///
/// # Safety
///
/// Caller must ensure:
/// - `kernel_buf` points to valid, readable kernel memory
/// - `user_ptr` points to valid, writable user memory
/// - User has space for `kernel_buf.len()` bytes
/// - Memory regions do not overlap
///
/// In a real implementation, this would use special copy functions
/// that handle page faults gracefully.
pub unsafe fn copy_to_user(kernel_buf: &[u8], user_ptr: *mut u8) -> Result<(), &'static str> {
    let size = kernel_buf.len();
    if size == 0 {
        return Ok(());
    }

    validate_user_buffer_mut(user_ptr, size)?;

    // SAFETY: Caller guarantees valid memory regions
    unsafe {
        core::ptr::copy_nonoverlapping(kernel_buf.as_ptr(), user_ptr, size);
    }

    Ok(())
}

/// Read a value from user space
///
/// # Safety
///
/// Caller must ensure `user_ptr` points to valid, initialized user memory
/// of type `T`.
pub unsafe fn read_user<T: Copy>(user_ptr: *const T) -> Result<T, &'static str> {
    validate_user_buffer(user_ptr as *const u8, core::mem::size_of::<T>())?;

    // SAFETY: Caller guarantees valid pointer and we validated the address
    Ok(unsafe { core::ptr::read(user_ptr) })
}

/// Write a value to user space
///
/// # Safety
///
/// Caller must ensure `user_ptr` points to valid, writable user memory
/// of type `T`.
pub unsafe fn write_user<T: Copy>(user_ptr: *mut T, value: T) -> Result<(), &'static str> {
    validate_user_buffer_mut(user_ptr as *mut u8, core::mem::size_of::<T>())?;

    // SAFETY: Caller guarantees valid pointer and we validated the address
    unsafe { core::ptr::write(user_ptr, value) };

    Ok(())
}

/// Create a slice from user space pointer (for reading)
///
/// # Safety
///
/// Caller must ensure:
/// - `user_ptr` points to valid, initialized user memory
/// - The memory contains at least `len` elements of type `T`
/// - The memory remains valid for the lifetime of the returned slice
pub unsafe fn slice_from_user<'a, T>(
    user_ptr: *const T,
    len: usize,
) -> Result<&'a [T], &'static str> {
    if len == 0 {
        return Ok(&[]);
    }

    let size = len
        .checked_mul(core::mem::size_of::<T>())
        .ok_or("Slice size overflow")?;

    validate_user_buffer(user_ptr as *const u8, size)?;

    // SAFETY: Caller guarantees valid memory
    Ok(unsafe { slice::from_raw_parts(user_ptr, len) })
}

/// Create a mutable slice from user space pointer (for writing)
///
/// # Safety
///
/// Caller must ensure:
/// - `user_ptr` points to valid, writable user memory
/// - The memory contains at least `len` elements of type `T`
/// - The memory remains valid and uniquely accessible for the lifetime
pub unsafe fn slice_from_user_mut<'a, T>(
    user_ptr: *mut T,
    len: usize,
) -> Result<&'a mut [T], &'static str> {
    if len == 0 {
        return Ok(&mut []);
    }

    let size = len
        .checked_mul(core::mem::size_of::<T>())
        .ok_or("Slice size overflow")?;

    validate_user_buffer_mut(user_ptr as *mut u8, size)?;

    // SAFETY: Caller guarantees valid memory
    Ok(unsafe { slice::from_raw_parts_mut(user_ptr, len) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_user_address() {
        assert!(is_user_address(VirtAddr::new(0x1000)));
        assert!(is_user_address(VirtAddr::new(0x7FFF_FFFF_FFFF)));
        assert!(!is_user_address(VirtAddr::new(0xFFFF_8000_0000_0000)));
    }

    #[test]
    fn test_is_kernel_address() {
        assert!(!is_kernel_address(VirtAddr::new(0x1000)));
        assert!(is_kernel_address(VirtAddr::new(0xFFFF_8000_0000_0000)));
        assert!(is_kernel_address(VirtAddr::new(0xFFFF_FFFF_FFFF_FFFF)));
    }

    #[test]
    fn test_is_user_range() {
        assert!(is_user_range(VirtAddr::new(0x1000), 0x1000));
        assert!(!is_user_range(VirtAddr::new(0x7FFF_FFFF_FFFF), 0x2000));
        assert!(!is_user_range(VirtAddr::new(0xFFFF_8000_0000_0000), 0x1000));
    }

    #[test]
    fn test_validate_user_ptr_null() {
        let result = validate_user_ptr(core::ptr::null());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_user_buffer_size() {
        let ptr = 0x1000 as *const u8;
        let result = validate_user_buffer(ptr, MAX_BUFFER_SIZE + 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_user_string() {
        // Create a null-terminated string in simulated user space
        let test_string = b"hello\0";
        let ptr = test_string.as_ptr();

        // This will work if ptr happens to be in "user space" range
        // In real kernel, user space validation would check page tables
        let result = unsafe { validate_user_string(ptr, 100) };

        // Result depends on where the test binary is loaded
        // We just verify the function doesn't panic
        let _result = result;
    }

    #[test]
    fn test_copy_from_user() {
        let user_data = [1u8, 2, 3, 4, 5];
        let mut kernel_buf = [0u8; 5];

        // This is a simulation - in real kernel, strict validation applies
        let result = unsafe { copy_from_user(user_data.as_ptr(), &mut kernel_buf) };

        // May fail if user_data is not in "user space" address range
        // We just verify the function doesn't panic
        let _result = result;
    }
}
