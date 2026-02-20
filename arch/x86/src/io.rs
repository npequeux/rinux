//! Port I/O
//!
//! Functions for reading and writing to I/O ports.

use core::arch::asm;

/// Read a byte from a port
///
/// # Safety
///
/// The caller must ensure that the port is valid and safe to read from.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack)
    );
    value
}

/// Write a byte to a port
///
/// # Safety
///
/// The caller must ensure that the port is valid and safe to write to.
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack)
    );
}

/// Read a word from a port
///
/// # Safety
///
/// The caller must ensure that the port is valid and safe to read from.
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    asm!(
        "in ax, dx",
        out("ax") value,
        in("dx") port,
        options(nomem, nostack)
    );
    value
}

/// Write a word to a port
///
/// # Safety
///
/// The caller must ensure that the port is valid and safe to write to.
#[inline(always)]
pub unsafe fn outw(port: u16, value: u16) {
    asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") value,
        options(nomem, nostack)
    );
}

/// Read a dword from a port
///
/// # Safety
///
/// The caller must ensure that the port is valid and safe to read from.
#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    asm!(
        "in eax, dx",
        out("eax") value,
        in("dx") port,
        options(nomem, nostack)
    );
    value
}

/// Write a dword to a port
///
/// # Safety
///
/// The caller must ensure that the port is valid and safe to write to.
#[inline(always)]
pub unsafe fn outl(port: u16, value: u32) {
    asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nomem, nostack)
    );
}

/// Wait for I/O operation to complete
///
/// # Safety
///
/// This function performs an I/O write operation to port 0x80.
#[inline(always)]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}
