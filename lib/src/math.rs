//! Math Utilities
//!
//! Mathematical functions and utilities.

/// Divide and round up
pub const fn div_round_up(n: usize, d: usize) -> usize {
    n.div_ceil(d)
}

/// Align up to power of 2
pub const fn align_up(n: usize, align: usize) -> usize {
    (n + align - 1) & !(align - 1)
}

/// Align down to power of 2
pub const fn align_down(n: usize, align: usize) -> usize {
    n & !(align - 1)
}

/// Check if value is aligned
pub const fn is_aligned(n: usize, align: usize) -> bool {
    n & (align - 1) == 0
}

/// Check if value is power of 2
pub const fn is_power_of_2(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}
