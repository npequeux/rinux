//! RISC-V Control and Status Registers (CSR)
//!
//! Access to RISC-V CSRs.

use core::arch::asm;

/// Read mvendorid CSR
#[inline]
pub fn read_mvendorid() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, mvendorid", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read marchid CSR
#[inline]
pub fn read_marchid() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, marchid", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read mimpid CSR
#[inline]
pub fn read_mimpid() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, mimpid", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read mhartid CSR
#[inline]
pub fn read_mhartid() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, mhartid", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read misa CSR
#[inline]
pub fn read_misa() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, misa", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read sstatus CSR
#[inline]
pub fn read_sstatus() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, sstatus", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write sstatus CSR
#[inline]
pub fn write_sstatus(val: usize) {
    unsafe {
        asm!("csrw sstatus, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read sie CSR (Supervisor Interrupt Enable)
#[inline]
pub fn read_sie() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, sie", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write sie CSR
#[inline]
pub fn write_sie(val: usize) {
    unsafe {
        asm!("csrw sie, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read sip CSR (Supervisor Interrupt Pending)
#[inline]
pub fn read_sip() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, sip", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read stvec CSR (Supervisor Trap Vector)
#[inline]
pub fn read_stvec() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, stvec", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write stvec CSR
#[inline]
pub fn write_stvec(val: usize) {
    unsafe {
        asm!("csrw stvec, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read scause CSR (Supervisor Trap Cause)
#[inline]
pub fn read_scause() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, scause", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read stval CSR (Supervisor Trap Value)
#[inline]
pub fn read_stval() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, stval", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read sepc CSR (Supervisor Exception Program Counter)
#[inline]
pub fn read_sepc() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, sepc", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write sepc CSR
#[inline]
pub fn write_sepc(val: usize) {
    unsafe {
        asm!("csrw sepc, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read satp CSR (Supervisor Address Translation and Protection)
#[inline]
pub fn read_satp() -> usize {
    let val: usize;
    unsafe {
        asm!("csrr {}, satp", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write satp CSR
#[inline]
pub fn write_satp(val: usize) {
    unsafe {
        asm!("csrw satp, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read time CSR
#[inline]
pub fn read_time() -> u64 {
    let val: u64;
    unsafe {
        asm!("rdtime {}", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read cycle CSR
#[inline]
pub fn read_cycle() -> u64 {
    let val: u64;
    unsafe {
        asm!("rdcycle {}", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read instret CSR (Instructions Retired)
#[inline]
pub fn read_instret() -> u64 {
    let val: u64;
    unsafe {
        asm!("rdinstret {}", out(reg) val, options(nomem, nostack));
    }
    val
}
