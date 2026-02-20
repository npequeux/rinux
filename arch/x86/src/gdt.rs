//! Global Descriptor Table
//!
//! GDT setup and management.

use core::arch::asm;
use spin::Mutex;

/// GDT entry
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn null() -> Self {
        GdtEntry {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    const fn new(base: u32, limit: u32, access: u8, granularity: u8) -> Self {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: (granularity & 0xF0) | (((limit >> 16) & 0x0F) as u8),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

/// GDT pointer
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

/// GDT
struct Gdt {
    entries: [GdtEntry; 5],
}

impl Gdt {
    const fn new() -> Self {
        Gdt {
            entries: [
                GdtEntry::null(),                      // 0x00: Null
                GdtEntry::new(0, 0xFFFFF, 0x9A, 0xA0), // 0x08: Code segment (64-bit)
                GdtEntry::new(0, 0xFFFFF, 0x92, 0xA0), // 0x10: Data segment (64-bit)
                GdtEntry::new(0, 0xFFFFF, 0xFA, 0xA0), // 0x18: User code segment
                GdtEntry::new(0, 0xFFFFF, 0xF2, 0xA0), // 0x20: User data segment
            ],
        }
    }

    fn pointer(&self) -> GdtPointer {
        GdtPointer {
            limit: (core::mem::size_of::<Self>() - 1) as u16,
            base: self.entries.as_ptr() as u64,
        }
    }
}

static GDT: Mutex<Gdt> = Mutex::new(Gdt::new());

/// Initialize GDT
pub fn init() {
    let gdt = GDT.lock();
    let pointer = gdt.pointer();

    unsafe {
        asm!(
            "lgdt [{}]",
            in(reg) &pointer,
            options(readonly, nostack)
        );

        // Reload segment registers
        asm!(
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            out("rax") _,
            options(nostack)
        );
    }
}
