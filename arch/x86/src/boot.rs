//! Boot Code
//!
//! Early boot initialization for x86_64.

use core::slice;
use core::str;

/// Multiboot header constants
const MULTIBOOT_MAGIC: u32 = 0x1BADB002;
const MULTIBOOT_FLAGS: u32 = 0x00000003;
const MULTIBOOT_CHECKSUM: u32 = 0u32
    .wrapping_sub(MULTIBOOT_MAGIC)
    .wrapping_sub(MULTIBOOT_FLAGS);

/// Multiboot boot loader magic value
const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

/// Multiboot header
#[repr(C, align(4))]
struct MultibootHeader {
    magic: u32,
    flags: u32,
    checksum: u32,
}

#[used]
#[link_section = ".multiboot"]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: MULTIBOOT_MAGIC,
    flags: MULTIBOOT_FLAGS,
    checksum: MULTIBOOT_CHECKSUM,
};

/// Multiboot info structure (passed by bootloader)
#[repr(C)]
#[allow(dead_code)]
pub struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    syms: [u32; 4],
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,
}

impl MultibootInfo {
    /// Check if memory info is valid
    pub fn has_memory_info(&self) -> bool {
        (self.flags & 0x1) != 0
    }

    /// Check if command line is present
    pub fn has_cmdline(&self) -> bool {
        (self.flags & 0x4) != 0
    }

    /// Get command line string if present
    ///
    /// # Safety
    ///
    /// The cmdline pointer must point to a valid null-terminated string
    pub unsafe fn get_cmdline(&self) -> Option<&str> {
        if !self.has_cmdline() || self.cmdline == 0 {
            return None;
        }

        // Find length of null-terminated string
        let ptr = self.cmdline as *const u8;
        let mut len = 0;
        while *ptr.add(len) != 0 && len < 4096 {
            len += 1;
        }

        let slice = slice::from_raw_parts(ptr, len);
        str::from_utf8(slice).ok()
    }

    /// Get lower memory size in KB
    pub fn lower_memory(&self) -> u32 {
        if self.has_memory_info() {
            self.mem_lower
        } else {
            0
        }
    }

    /// Get upper memory size in KB
    pub fn upper_memory(&self) -> u32 {
        if self.has_memory_info() {
            self.mem_upper
        } else {
            0
        }
    }
}

/// Boot stack size
const STACK_SIZE: usize = 16384;

/// Boot stack
#[repr(align(16))]
struct BootStack(#[allow(dead_code)] [u8; STACK_SIZE]);

#[used]
static mut BOOT_STACK: BootStack = BootStack([0; STACK_SIZE]);

/// Early boot initialization
///
/// # Arguments
///
/// * `multiboot_magic` - Magic value from bootloader (should be 0x2BADB002)
/// * `multiboot_info_addr` - Physical address of Multiboot info structure
///
/// # Safety
///
/// This function must be called exactly once during boot, before paging is fully set up.
/// The multiboot_info_addr must point to a valid Multiboot info structure.
pub unsafe fn early_init(multiboot_magic: u32, multiboot_info_addr: u32) -> Result<(), &'static str> {
    // Verify multiboot magic
    if multiboot_magic != MULTIBOOT_BOOTLOADER_MAGIC {
        return Err("Invalid Multiboot magic value");
    }

    // Validate multiboot info pointer
    if multiboot_info_addr == 0 {
        return Err("NULL Multiboot info pointer");
    }

    // Parse multiboot info
    let mbi = &*(multiboot_info_addr as *const MultibootInfo);

    // Extract and log memory information
    if mbi.has_memory_info() {
        let lower = mbi.lower_memory();
        let upper = mbi.upper_memory();
        // Memory info available: lower KB, upper KB
        // In a real implementation, we'd store this for the memory subsystem
        let _ = (lower, upper);
    }

    // Extract command line if present
    if let Some(cmdline) = mbi.get_cmdline() {
        // Command line will be parsed later by kernel::cmdline::init()
        // For now, just validate it exists
        let _ = cmdline;
    }

    Ok(())
}

/// Get Multiboot info structure
///
/// # Safety
///
/// This assumes the multiboot info was saved during early_init
pub unsafe fn get_multiboot_info(addr: u32) -> Option<&'static MultibootInfo> {
    if addr == 0 {
        None
    } else {
        Some(&*(addr as *const MultibootInfo))
    }
}
