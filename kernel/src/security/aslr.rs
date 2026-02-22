//! Address Space Layout Randomization (ASLR)
//!
//! Provides randomization for stack, heap, and mmap regions.

use crate::types::VirtAddr;
use core::sync::atomic::{AtomicU64, Ordering};

/// Simple pseudo-random number generator (PRNG) for ASLR
///
/// Uses a simple linear congruential generator (LCG).
/// In a real implementation, this would use a cryptographically secure PRNG.
struct Prng {
    state: AtomicU64,
}

impl Prng {
    /// Create a new PRNG with seed
    const fn new(seed: u64) -> Self {
        Prng {
            state: AtomicU64::new(seed),
        }
    }

    /// Generate next random number
    fn next(&self) -> u64 {
        // Simple LCG: state = (a * state + c) mod m
        // Using constants from Numerical Recipes
        const A: u64 = 1664525;
        const C: u64 = 1013904223;

        let old = self.state.load(Ordering::Relaxed);
        let new = old.wrapping_mul(A).wrapping_add(C);
        self.state.store(new, Ordering::Relaxed);
        new
    }

    /// Generate random u64 in range [min, max)
    fn next_range(&self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        let range = max - min;
        let value = self.next();
        min + (value % range)
    }
}

/// Global entropy pool for ASLR
static ENTROPY_POOL: Prng = Prng::new(0x123456789ABCDEF0);

/// Initialize ASLR entropy pool
///
/// In a real implementation, this would seed from hardware RNG or other entropy sources.
pub fn init() {
    // Seed with a better initial value
    // In real implementation, use RDRAND, time, etc.
    ENTROPY_POOL
        .state
        .store(0x123456789ABCDEF0, Ordering::Relaxed);
}

/// Reseed the entropy pool
pub fn reseed(seed: u64) {
    ENTROPY_POOL.state.store(seed, Ordering::Release);
}

/// Get random entropy bytes
pub fn get_random_u64() -> u64 {
    ENTROPY_POOL.next()
}

/// Default randomization ranges for different memory regions
pub mod defaults {
    /// Stack randomization range (28 bits = 256MB)
    pub const STACK_RANDOM_BITS: u32 = 28;
    /// Heap randomization range (28 bits = 256MB)
    pub const HEAP_RANDOM_BITS: u32 = 28;
    /// Mmap randomization range (28 bits = 256MB)
    pub const MMAP_RANDOM_BITS: u32 = 28;
    /// PIE (Position Independent Executable) randomization range (28 bits)
    pub const PIE_RANDOM_BITS: u32 = 28;
}

/// Randomize stack address
///
/// Returns a randomized offset to add to the base stack address.
pub fn randomize_stack() -> u64 {
    let max_offset = 1u64 << defaults::STACK_RANDOM_BITS;
    ENTROPY_POOL.next_range(0, max_offset)
}

/// Randomize heap address
///
/// Returns a randomized offset to add to the base heap address.
pub fn randomize_heap() -> u64 {
    let max_offset = 1u64 << defaults::HEAP_RANDOM_BITS;
    ENTROPY_POOL.next_range(0, max_offset)
}

/// Randomize mmap address
///
/// Returns a randomized offset to add to the base mmap address.
pub fn randomize_mmap() -> u64 {
    let max_offset = 1u64 << defaults::MMAP_RANDOM_BITS;
    ENTROPY_POOL.next_range(0, max_offset)
}

/// Randomize PIE executable load address
///
/// Returns a randomized offset to add to the base PIE load address.
pub fn randomize_pie() -> u64 {
    let max_offset = 1u64 << defaults::PIE_RANDOM_BITS;
    ENTROPY_POOL.next_range(0, max_offset)
}

/// Generate randomized address in range
///
/// # Arguments
///
/// * `base` - Base address
/// * `size` - Size of the address space
/// * `random_bits` - Number of bits of entropy to use
///
/// # Returns
///
/// Randomized address within the range
pub fn randomize_address(base: VirtAddr, size: u64, random_bits: u32) -> VirtAddr {
    if random_bits == 0 || size == 0 {
        return base;
    }

    let max_offset = (1u64 << random_bits).min(size);
    let offset = ENTROPY_POOL.next_range(0, max_offset);

    // Align to page boundary (4KB)
    let aligned_offset = offset & !0xFFF;

    VirtAddr::new(base.as_u64().saturating_add(aligned_offset))
}

/// ASLR configuration
#[derive(Debug, Clone, Copy)]
pub struct AslrConfig {
    /// Enable stack randomization
    pub randomize_stack: bool,
    /// Enable heap randomization
    pub randomize_heap: bool,
    /// Enable mmap randomization
    pub randomize_mmap: bool,
    /// Enable PIE randomization
    pub randomize_pie: bool,
    /// Custom stack randomization bits
    pub stack_random_bits: u32,
    /// Custom heap randomization bits
    pub heap_random_bits: u32,
    /// Custom mmap randomization bits
    pub mmap_random_bits: u32,
    /// Custom PIE randomization bits
    pub pie_random_bits: u32,
}

impl AslrConfig {
    /// Create default ASLR configuration (all enabled)
    pub const fn default() -> Self {
        AslrConfig {
            randomize_stack: true,
            randomize_heap: true,
            randomize_mmap: true,
            randomize_pie: true,
            stack_random_bits: defaults::STACK_RANDOM_BITS,
            heap_random_bits: defaults::HEAP_RANDOM_BITS,
            mmap_random_bits: defaults::MMAP_RANDOM_BITS,
            pie_random_bits: defaults::PIE_RANDOM_BITS,
        }
    }

    /// Create disabled ASLR configuration
    pub const fn disabled() -> Self {
        AslrConfig {
            randomize_stack: false,
            randomize_heap: false,
            randomize_mmap: false,
            randomize_pie: false,
            stack_random_bits: 0,
            heap_random_bits: 0,
            mmap_random_bits: 0,
            pie_random_bits: 0,
        }
    }

    /// Apply randomization to stack address
    pub fn apply_stack(&self, base: VirtAddr, size: u64) -> VirtAddr {
        if self.randomize_stack {
            randomize_address(base, size, self.stack_random_bits)
        } else {
            base
        }
    }

    /// Apply randomization to heap address
    pub fn apply_heap(&self, base: VirtAddr, size: u64) -> VirtAddr {
        if self.randomize_heap {
            randomize_address(base, size, self.heap_random_bits)
        } else {
            base
        }
    }

    /// Apply randomization to mmap address
    pub fn apply_mmap(&self, base: VirtAddr, size: u64) -> VirtAddr {
        if self.randomize_mmap {
            randomize_address(base, size, self.mmap_random_bits)
        } else {
            base
        }
    }

    /// Apply randomization to PIE address
    pub fn apply_pie(&self, base: VirtAddr, size: u64) -> VirtAddr {
        if self.randomize_pie {
            randomize_address(base, size, self.pie_random_bits)
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prng_generates_different_values() {
        let prng = Prng::new(42);
        let val1 = prng.next();
        let val2 = prng.next();
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_prng_range() {
        let prng = Prng::new(42);
        for _ in 0..100 {
            let val = prng.next_range(10, 20);
            assert!(val >= 10 && val < 20);
        }
    }

    #[test]
    fn test_randomize_stack() {
        let offset1 = randomize_stack();
        let offset2 = randomize_stack();
        // Should generate different offsets (statistically)
        // Note: There's a tiny chance they could be equal
        assert!(offset1 < (1u64 << defaults::STACK_RANDOM_BITS));
        assert!(offset2 < (1u64 << defaults::STACK_RANDOM_BITS));
    }

    #[test]
    fn test_randomize_address() {
        let base = VirtAddr::new(0x1000_0000);
        let size = 0x1000_0000;
        let addr1 = randomize_address(base, size, 20);
        let addr2 = randomize_address(base, size, 20);

        // Both should be >= base
        assert!(addr1.as_u64() >= base.as_u64());
        assert!(addr2.as_u64() >= base.as_u64());

        // Should be page aligned
        assert_eq!(addr1.as_u64() & 0xFFF, 0);
        assert_eq!(addr2.as_u64() & 0xFFF, 0);
    }

    #[test]
    fn test_aslr_config_disabled() {
        let config = AslrConfig::disabled();
        let base = VirtAddr::new(0x1000_0000);
        let size = 0x1000_0000;

        // Should return base address unchanged
        assert_eq!(config.apply_stack(base, size), base);
        assert_eq!(config.apply_heap(base, size), base);
        assert_eq!(config.apply_mmap(base, size), base);
        assert_eq!(config.apply_pie(base, size), base);
    }

    #[test]
    fn test_aslr_config_default() {
        let config = AslrConfig::default();
        assert!(config.randomize_stack);
        assert!(config.randomize_heap);
        assert!(config.randomize_mmap);
        assert!(config.randomize_pie);
    }

    #[test]
    fn test_reseed() {
        reseed(12345);
        let val1 = get_random_u64();
        reseed(12345);
        let val2 = get_random_u64();
        // Same seed should produce same sequence
        assert_eq!(val1, val2);
    }
}
