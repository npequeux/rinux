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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_round_up() {
        assert_eq!(div_round_up(0, 4), 0);
        assert_eq!(div_round_up(1, 4), 1);
        assert_eq!(div_round_up(4, 4), 1);
        assert_eq!(div_round_up(5, 4), 2);
        assert_eq!(div_round_up(8, 4), 2);
        assert_eq!(div_round_up(9, 4), 3);
        assert_eq!(div_round_up(100, 8), 13);
        assert_eq!(div_round_up(1024, 4096), 1);
        assert_eq!(div_round_up(4096, 4096), 1);
        assert_eq!(div_round_up(4097, 4096), 2);
    }

    #[test]
    fn test_align_up() {
        // Test alignment to 4
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(2, 4), 4);
        assert_eq!(align_up(3, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
        
        // Test alignment to 8
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(7, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
        
        // Test alignment to page size (4096)
        assert_eq!(align_up(0, 4096), 0);
        assert_eq!(align_up(1, 4096), 4096);
        assert_eq!(align_up(4095, 4096), 4096);
        assert_eq!(align_up(4096, 4096), 4096);
        assert_eq!(align_up(4097, 4096), 8192);
    }

    #[test]
    fn test_align_down() {
        // Test alignment to 4
        assert_eq!(align_down(0, 4), 0);
        assert_eq!(align_down(1, 4), 0);
        assert_eq!(align_down(2, 4), 0);
        assert_eq!(align_down(3, 4), 0);
        assert_eq!(align_down(4, 4), 4);
        assert_eq!(align_down(5, 4), 4);
        assert_eq!(align_down(7, 4), 4);
        assert_eq!(align_down(8, 4), 8);
        
        // Test alignment to 8
        assert_eq!(align_down(0, 8), 0);
        assert_eq!(align_down(7, 8), 0);
        assert_eq!(align_down(8, 8), 8);
        assert_eq!(align_down(15, 8), 8);
        assert_eq!(align_down(16, 8), 16);
        
        // Test alignment to page size (4096)
        assert_eq!(align_down(0, 4096), 0);
        assert_eq!(align_down(4095, 4096), 0);
        assert_eq!(align_down(4096, 4096), 4096);
        assert_eq!(align_down(8191, 4096), 4096);
        assert_eq!(align_down(8192, 4096), 8192);
    }

    #[test]
    fn test_is_aligned() {
        // Test alignment to 4
        assert!(is_aligned(0, 4));
        assert!(!is_aligned(1, 4));
        assert!(!is_aligned(2, 4));
        assert!(!is_aligned(3, 4));
        assert!(is_aligned(4, 4));
        assert!(!is_aligned(5, 4));
        assert!(is_aligned(8, 4));
        assert!(is_aligned(12, 4));
        
        // Test alignment to 8
        assert!(is_aligned(0, 8));
        assert!(!is_aligned(4, 8));
        assert!(is_aligned(8, 8));
        assert!(!is_aligned(12, 8));
        assert!(is_aligned(16, 8));
        
        // Test alignment to page size (4096)
        assert!(is_aligned(0, 4096));
        assert!(!is_aligned(1, 4096));
        assert!(!is_aligned(4095, 4096));
        assert!(is_aligned(4096, 4096));
        assert!(!is_aligned(4097, 4096));
        assert!(is_aligned(8192, 4096));
    }

    #[test]
    fn test_is_power_of_2() {
        // Test zero and one
        assert!(!is_power_of_2(0));
        assert!(is_power_of_2(1));
        
        // Test powers of 2
        assert!(is_power_of_2(2));
        assert!(is_power_of_2(4));
        assert!(is_power_of_2(8));
        assert!(is_power_of_2(16));
        assert!(is_power_of_2(32));
        assert!(is_power_of_2(64));
        assert!(is_power_of_2(128));
        assert!(is_power_of_2(256));
        assert!(is_power_of_2(512));
        assert!(is_power_of_2(1024));
        assert!(is_power_of_2(4096));
        
        // Test non-powers of 2
        assert!(!is_power_of_2(3));
        assert!(!is_power_of_2(5));
        assert!(!is_power_of_2(6));
        assert!(!is_power_of_2(7));
        assert!(!is_power_of_2(9));
        assert!(!is_power_of_2(15));
        assert!(!is_power_of_2(100));
        assert!(!is_power_of_2(1000));
        assert!(!is_power_of_2(4095));
    }
}
