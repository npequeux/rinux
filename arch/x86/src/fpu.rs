//! FPU and SSE Context Management
//!
//! Save and restore FPU/SSE/AVX context for task switching.

use core::arch::asm;

/// FPU/SSE context saved with FXSAVE
#[repr(C, align(16))]
pub struct FxsaveArea {
    pub fcw: u16, // FPU Control Word
    pub fsw: u16, // FPU Status Word
    pub ftw: u8,  // FPU Tag Word (abridged)
    _reserved1: u8,
    pub fop: u16,        // FPU Opcode
    pub fip: u64,        // FPU Instruction Pointer
    pub fdp: u64,        // FPU Data Pointer
    pub mxcsr: u32,      // MXCSR Register
    pub mxcsr_mask: u32, // MXCSR Mask
    pub st: [u128; 8],   // ST0-ST7 / MM0-MM7
    pub xmm: [u128; 16], // XMM0-XMM15
    _reserved2: [u128; 3],
}

impl FxsaveArea {
    /// Create a new FPU context with default values
    pub const fn new() -> Self {
        Self {
            fcw: 0x037F, // Default FPU control word
            fsw: 0,
            ftw: 0,
            _reserved1: 0,
            fop: 0,
            fip: 0,
            fdp: 0,
            mxcsr: 0x1F80, // Default MXCSR (all exceptions masked)
            mxcsr_mask: 0xFFFF,
            st: [0; 8],
            xmm: [0; 16],
            _reserved2: [0; 3],
        }
    }
}

/// Extended state (XSAVE)
#[repr(C, align(64))]
pub struct XsaveArea {
    pub fxsave: FxsaveArea,
    pub xsave_header: XsaveHeader,
    // Extended state components follow
}

#[repr(C)]
pub struct XsaveHeader {
    pub xstate_bv: u64,
    pub xcomp_bv: u64,
    _reserved: [u64; 6],
}

/// FPU state management
pub struct FpuContext {
    area: FxsaveArea,
}

impl FpuContext {
    /// Create new FPU context
    pub const fn new() -> Self {
        Self {
            area: FxsaveArea::new(),
        }
    }

    /// Save FPU state using FXSAVE
    pub fn save(&mut self) {
        unsafe {
            asm!(
                "fxsave [{}]",
                in(reg) &mut self.area as *mut FxsaveArea,
                options(nostack)
            );
        }
    }

    /// Restore FPU state using FXRSTOR
    pub fn restore(&self) {
        unsafe {
            asm!(
                "fxrstor [{}]",
                in(reg) &self.area as *const FxsaveArea,
                options(nostack)
            );
        }
    }

    /// Initialize FPU to default state
    pub fn init(&mut self) {
        unsafe {
            asm!("fninit", options(nostack, nomem));
        }
        self.save();
    }
}

/// Check if FXSAVE/FXRSTOR is supported
pub fn has_fxsr() -> bool {
    use crate::cpu::cpuid;
    let (_, _, _, edx) = cpuid(1);
    (edx & (1 << 24)) != 0
}

/// Check if XSAVE is supported
pub fn has_xsave() -> bool {
    use crate::cpu::cpuid;
    let (_, _, ecx, _) = cpuid(1);
    (ecx & (1 << 26)) != 0
}

/// Check if AVX is supported
pub fn has_avx() -> bool {
    use crate::cpu::cpuid;
    let (_, _, ecx, _) = cpuid(1);
    (ecx & (1 << 28)) != 0
}

/// Enable SSE/SSE2
pub fn enable_sse() {
    use crate::long_mode::{read_cr0, read_cr4, write_cr0, write_cr4};

    unsafe {
        // Clear CR0.EM (bit 2) - Enable FPU
        // Set CR0.MP (bit 1) - Monitor coprocessor
        let mut cr0 = read_cr0();
        cr0 &= !(1 << 2); // Clear EM
        cr0 |= 1 << 1; // Set MP
        write_cr0(cr0);

        // Set CR4.OSFXSR (bit 9) - Enable FXSAVE/FXRSTOR
        // Set CR4.OSXMMEXCPT (bit 10) - Enable SSE exceptions
        let mut cr4 = read_cr4();
        cr4 |= (1 << 9) | (1 << 10);
        write_cr4(cr4);
    }

    rinux_kernel::printk!("[FPU] SSE enabled\n");
}

/// Enable AVX (requires XSAVE)
pub fn enable_avx() -> bool {
    if !has_xsave() || !has_avx() {
        return false;
    }

    use crate::long_mode::{read_cr4, write_cr4};

    unsafe {
        // Enable XSAVE in CR4
        let mut cr4 = read_cr4();
        cr4 |= 1 << 18; // Set CR4.OSXSAVE
        write_cr4(cr4);

        // Enable x87 and SSE state (bits 0-1)
        // Enable AVX state (bit 2)
        const XCR0_X87: u64 = 1 << 0;
        const XCR0_SSE: u64 = 1 << 1;
        const XCR0_AVX: u64 = 1 << 2;
        let xcr0 = XCR0_X87 | XCR0_SSE | XCR0_AVX;

        // Write to XCR0 using XSETBV
        asm!(
            "xsetbv",
            in("ecx") 0u32,  // XCR0
            in("eax") (xcr0 & 0xFFFF_FFFF) as u32,
            in("edx") (xcr0 >> 32) as u32,
            options(nostack, nomem)
        );
    }

    rinux_kernel::printk!("[FPU] AVX enabled\n");
    true
}

/// Initialize FPU/SSE support
pub fn init() {
    rinux_kernel::printk!("[FPU] Initializing FPU/SSE support...\n");

    // Check for FXSR support
    if !has_fxsr() {
        rinux_kernel::printk!("[FPU] WARNING: FXSAVE/FXRSTOR not supported\n");
        return;
    }

    // Enable SSE
    enable_sse();

    // Try to enable AVX if available
    if has_avx() {
        if enable_avx() {
            rinux_kernel::printk!("[FPU] AVX support enabled\n");
        }
    }

    // Initialize FPU state
    unsafe {
        asm!("fninit", options(nostack, nomem));
    }

    rinux_kernel::printk!("[FPU] Initialization complete\n");
    rinux_kernel::printk!(
        "[FPU] Features: FXSR={}, XSAVE={}, AVX={}\n",
        has_fxsr(),
        has_xsave(),
        has_avx()
    );
}

/// Save extended state with XSAVE (if available)
pub unsafe fn xsave(area: *mut u8, mask: u64) {
    asm!(
        "xsave [{}]",
        in(reg) area,
        in("eax") (mask & 0xFFFF_FFFF) as u32,
        in("edx") (mask >> 32) as u32,
        options(nostack)
    );
}

/// Restore extended state with XRSTOR (if available)
pub unsafe fn xrstor(area: *const u8, mask: u64) {
    asm!(
        "xrstor [{}]",
        in(reg) area,
        in("eax") (mask & 0xFFFF_FFFF) as u32,
        in("edx") (mask >> 32) as u32,
        options(nostack)
    );
}
