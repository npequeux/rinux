//! RISC-V CPU Management
//!
//! CPU detection and feature management.

use crate::csr;

/// CPU information
pub struct CpuInfo {
    pub mvendorid: usize,
    pub marchid: usize,
    pub mimpid: usize,
    pub mhartid: usize,
    pub features: CpuFeatures,
}

bitflags::bitflags! {
    /// CPU features (from misa)
    pub struct CpuFeatures: usize {
        const A = 1 << 0;   // Atomic extension
        const C = 1 << 2;   // Compressed extension
        const D = 1 << 3;   // Double-precision floating-point
        const F = 1 << 5;   // Single-precision floating-point
        const I = 1 << 8;   // RV32I/64I/128I base ISA
        const M = 1 << 12;  // Integer multiply/divide
        const S = 1 << 18;  // Supervisor mode
        const U = 1 << 20;  // User mode
        const V = 1 << 21;  // Vector extension
    }
}

/// Detect CPU features from misa
fn detect_features() -> CpuFeatures {
    let misa = csr::read_misa();
    CpuFeatures::from_bits_truncate(misa)
}

/// Get CPU information
pub fn get_cpu_info() -> CpuInfo {
    CpuInfo {
        mvendorid: csr::read_mvendorid(),
        marchid: csr::read_marchid(),
        mimpid: csr::read_mimpid(),
        mhartid: csr::read_mhartid(),
        features: detect_features(),
    }
}

/// Get current hart (hardware thread) ID
pub fn current_hart_id() -> usize {
    csr::read_mhartid()
}

/// Initialize CPU
pub fn init() {
    let info = get_cpu_info();
    
    kernel::printk!("[RISCV] CPU Information:\n");
    kernel::printk!("  Vendor ID:  {:#x}\n", info.mvendorid);
    kernel::printk!("  Arch ID:    {:#x}\n", info.marchid);
    kernel::printk!("  Impl ID:    {:#x}\n", info.mimpid);
    kernel::printk!("  Hart ID:    {}\n", info.mhartid);
    kernel::printk!("  Features:   {:?}\n", info.features);
}
