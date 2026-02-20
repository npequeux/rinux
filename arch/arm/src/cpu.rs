//! ARM64 CPU Management
//!
//! CPU detection and feature management for ARM64.

use core::arch::asm;

/// CPU information
pub struct CpuInfo {
    pub midr: u64,      // Main ID Register
    pub mpidr: u64,     // Multiprocessor Affinity Register
    pub revidr: u64,    // Revision ID Register
    pub features: CpuFeatures,
}

bitflags::bitflags! {
    /// CPU features
    pub struct CpuFeatures: u64 {
        const FP = 1 << 0;      // Floating Point
        const ASIMD = 1 << 1;   // Advanced SIMD
        const AES = 1 << 2;     // AES instructions
        const SHA1 = 1 << 3;    // SHA1 instructions
        const SHA2 = 1 << 4;    // SHA2 instructions
        const CRC32 = 1 << 5;   // CRC32 instructions
        const ATOMICS = 1 << 6; // Atomic instructions
        const SVE = 1 << 7;     // Scalable Vector Extension
    }
}

/// Read MIDR_EL1 (Main ID Register)
#[inline]
fn read_midr() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, midr_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read MPIDR_EL1 (Multiprocessor Affinity Register)
#[inline]
fn read_mpidr() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read REVIDR_EL1 (Revision ID Register)
#[inline]
fn read_revidr() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, revidr_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read ID_AA64ISAR0_EL1 (Instruction Set Attribute Register 0)
#[inline]
fn read_isar0() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, id_aa64isar0_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read ID_AA64PFR0_EL1 (Processor Feature Register 0)
#[inline]
fn read_pfr0() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, id_aa64pfr0_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Detect CPU features
fn detect_features() -> CpuFeatures {
    let isar0 = read_isar0();
    let pfr0 = read_pfr0();
    
    let mut features = CpuFeatures::empty();
    
    // Check for FP and ASIMD
    let fp_asimd = pfr0 & 0xF;
    if fp_asimd != 0xF {
        features |= CpuFeatures::FP | CpuFeatures::ASIMD;
    }
    
    // Check for AES
    let aes = (isar0 >> 4) & 0xF;
    if aes >= 1 {
        features |= CpuFeatures::AES;
    }
    
    // Check for SHA1
    let sha1 = (isar0 >> 8) & 0xF;
    if sha1 >= 1 {
        features |= CpuFeatures::SHA1;
    }
    
    // Check for SHA2
    let sha2 = (isar0 >> 12) & 0xF;
    if sha2 >= 1 {
        features |= CpuFeatures::SHA2;
    }
    
    // Check for CRC32
    let crc32 = (isar0 >> 16) & 0xF;
    if crc32 >= 1 {
        features |= CpuFeatures::CRC32;
    }
    
    // Check for Atomics
    let atomics = (isar0 >> 20) & 0xF;
    if atomics >= 2 {
        features |= CpuFeatures::ATOMICS;
    }
    
    // Check for SVE
    let sve = (pfr0 >> 32) & 0xF;
    if sve >= 1 {
        features |= CpuFeatures::SVE;
    }
    
    features
}

/// Get CPU information
pub fn get_cpu_info() -> CpuInfo {
    CpuInfo {
        midr: read_midr(),
        mpidr: read_mpidr(),
        revidr: read_revidr(),
        features: detect_features(),
    }
}

/// Get current CPU ID
pub fn current_cpu_id() -> u32 {
    let mpidr = read_mpidr();
    (mpidr & 0xFF) as u32
}

/// Initialize CPU
pub fn init() {
    let info = get_cpu_info();
    
    kernel::printk!("[ARM64] CPU Information:\n");
    kernel::printk!("  MIDR:   {:#018x}\n", info.midr);
    kernel::printk!("  MPIDR:  {:#018x}\n", info.mpidr);
    kernel::printk!("  CPU ID: {}\n", current_cpu_id());
    kernel::printk!("  Features: {:?}\n", info.features);
}
