//! RISC-V Supervisor Binary Interface (SBI)
//!
//! Interface to the SBI firmware/runtime.

use core::arch::asm;

/// SBI extension IDs
mod eid {
    pub const SET_TIMER: usize = 0x00;
    pub const CONSOLE_PUTCHAR: usize = 0x01;
    pub const CONSOLE_GETCHAR: usize = 0x02;
    pub const CLEAR_IPI: usize = 0x03;
    pub const SEND_IPI: usize = 0x04;
    pub const REMOTE_FENCE_I: usize = 0x05;
    pub const REMOTE_SFENCE_VMA: usize = 0x06;
    pub const REMOTE_SFENCE_VMA_ASID: usize = 0x07;
    pub const SHUTDOWN: usize = 0x08;
}

/// SBI error codes
#[derive(Debug)]
pub enum SbiError {
    Success = 0,
    Failed = -1,
    NotSupported = -2,
    InvalidParam = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
}

/// SBI return value
#[repr(C)]
pub struct SbiRet {
    pub error: isize,
    pub value: usize,
}

/// Make an SBI call
#[inline]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> SbiRet {
    let error: isize;
    let value: usize;
    
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            in("a0") arg0,
            in("a1") arg1,
            in("a2") arg2,
            lateout("a0") error,
            lateout("a1") value,
        );
    }
    
    SbiRet { error, value }
}

/// Set timer
pub fn set_timer(stime_value: u64) {
    sbi_call(eid::SET_TIMER, 0, stime_value as usize, 0, 0);
}

/// Console putchar
pub fn console_putchar(ch: u8) {
    sbi_call(eid::CONSOLE_PUTCHAR, 0, ch as usize, 0, 0);
}

/// Console getchar
pub fn console_getchar() -> Option<u8> {
    let ret = sbi_call(eid::CONSOLE_GETCHAR, 0, 0, 0, 0);
    if ret.error == 0 {
        Some(ret.value as u8)
    } else {
        None
    }
}

/// Send IPI to other harts
pub fn send_ipi(hart_mask: usize, hart_mask_base: usize) {
    sbi_call(eid::SEND_IPI, 0, hart_mask, hart_mask_base, 0);
}

/// Clear IPI
pub fn clear_ipi() {
    sbi_call(eid::CLEAR_IPI, 0, 0, 0, 0);
}

/// Shutdown system
pub fn shutdown() -> ! {
    sbi_call(eid::SHUTDOWN, 0, 0, 0, 0);
    loop {
        crate::halt();
    }
}

/// Initialize SBI
pub fn init() {
    kernel::printk!("[RISCV] SBI initialized\n");
}
