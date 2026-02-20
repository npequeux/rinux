//! CPU Management
//!
//! CPU detection and management.

use core::arch::asm;

/// CPU vendor information
#[derive(Debug, Clone, Copy)]
pub enum CpuVendor {
    Intel,
    AMD,
    Unknown,
}

/// CPU information
#[allow(dead_code)]
pub struct CpuInfo {
    vendor: CpuVendor,
    family: u32,
    model: u32,
    stepping: u32,
    features: CpuFeatures,
}

bitflags::bitflags! {
    /// CPU features
    pub struct CpuFeatures: u64 {
        const SSE = 1 << 0;
        const SSE2 = 1 << 1;
        const SSE3 = 1 << 2;
        const AVX = 1 << 3;
        const AVX2 = 1 << 4;
        const FPU = 1 << 5;
        const MMX = 1 << 6;
        const APIC = 1 << 7;
        const MSR = 1 << 8;
        const PAT = 1 << 9;
        const PSE = 1 << 10;
        const PAE = 1 << 11;
    }
}

impl CpuInfo {
    /// Get CPU information
    pub fn detect() -> Self {
        let (vendor, family, model, stepping) = cpuid_basic();
        let features = detect_features();

        CpuInfo {
            vendor,
            family,
            model,
            stepping,
            features,
        }
    }
}

/// Execute CPUID instruction
pub fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let mut eax: u32;
    let mut ebx: u32;
    let mut ecx: u32;
    let mut edx: u32;

    unsafe {
        asm!(
            "mov r11, rbx",      // Save rbx to r11
            "cpuid",             // Execute cpuid
            "mov {ebx:e}, ebx",  // Copy ebx result to output register
            "mov rbx, r11",      // Restore rbx
            ebx = out(reg) ebx,
            inout("eax") leaf => eax,
            out("ecx") ecx,
            out("edx") edx,
            out("r11") _,        // Mark r11 as clobbered
            options(nomem, nostack)
        );
    }

    (eax, ebx, ecx, edx)
}

/// Get basic CPU information
fn cpuid_basic() -> (CpuVendor, u32, u32, u32) {
    let (_eax, ebx, ecx, edx) = cpuid(0);

    // Determine vendor
    let vendor = if ebx == 0x756e6547 && edx == 0x49656e69 && ecx == 0x6c65746e {
        CpuVendor::Intel
    } else if ebx == 0x68747541 && edx == 0x69746e65 && ecx == 0x444d4163 {
        CpuVendor::AMD
    } else {
        CpuVendor::Unknown
    };

    // Get family, model, stepping
    let (eax, _, _, _) = cpuid(1);
    let family = ((eax >> 8) & 0xF) + ((eax >> 20) & 0xFF);
    let model = ((eax >> 4) & 0xF) | ((eax >> 12) & 0xF0);
    let stepping = eax & 0xF;

    (vendor, family, model, stepping)
}

/// Detect CPU features
fn detect_features() -> CpuFeatures {
    let (_, _, ecx, edx) = cpuid(1);

    let mut features = CpuFeatures::empty();

    if edx & (1 << 0) != 0 {
        features |= CpuFeatures::FPU;
    }
    if edx & (1 << 23) != 0 {
        features |= CpuFeatures::MMX;
    }
    if edx & (1 << 25) != 0 {
        features |= CpuFeatures::SSE;
    }
    if edx & (1 << 26) != 0 {
        features |= CpuFeatures::SSE2;
    }
    if ecx & (1 << 0) != 0 {
        features |= CpuFeatures::SSE3;
    }
    if ecx & (1 << 28) != 0 {
        features |= CpuFeatures::AVX;
    }

    features
}

/// Read MSR
///
/// # Safety
///
/// The caller must ensure that the specified MSR is valid and accessible.
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let (low, high): (u32, u32);
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack)
    );
    ((high as u64) << 32) | (low as u64)
}

/// Write MSR
///
/// # Safety
///
/// The caller must ensure that the specified MSR is valid and accessible, and that
/// the value being written is appropriate for the MSR.
pub unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}
