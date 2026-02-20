//! ACPI (Advanced Configuration and Power Interface) Support
//!
//! ACPI provides power management, hardware configuration, and system information.

use core::ptr;

/// ACPI RSDP (Root System Description Pointer) signature
const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";

/// ACPI table signatures
pub const RSDT_SIGNATURE: u32 = u32::from_le_bytes(*b"RSDT");
pub const XSDT_SIGNATURE: u32 = u32::from_le_bytes(*b"XSDT");
pub const FADT_SIGNATURE: u32 = u32::from_le_bytes(*b"FACP");
pub const MADT_SIGNATURE: u32 = u32::from_le_bytes(*b"APIC");
pub const MCFG_SIGNATURE: u32 = u32::from_le_bytes(*b"MCFG");
pub const HPET_SIGNATURE: u32 = u32::from_le_bytes(*b"HPET");

/// ACPI RSDP structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

/// ACPI RSDP 2.0 extended structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Rsdp2 {
    pub rsdp: Rsdp,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

/// ACPI table header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AcpiTableHeader {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

/// ACPI FADT (Fixed ACPI Description Table)
#[repr(C, packed)]
pub struct Fadt {
    pub header: AcpiTableHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    pub reserved: u8,
    pub preferred_pm_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    // ... many more fields
}

/// Power management profile types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PmProfile {
    Unspecified = 0,
    Desktop = 1,
    Mobile = 2,
    Workstation = 3,
    EnterpriseServer = 4,
    SohoServer = 5,
    AppliancePc = 6,
    PerformanceServer = 7,
    Tablet = 8,
}

impl From<u8> for PmProfile {
    fn from(value: u8) -> Self {
        match value {
            1 => PmProfile::Desktop,
            2 => PmProfile::Mobile,
            3 => PmProfile::Workstation,
            4 => PmProfile::EnterpriseServer,
            5 => PmProfile::SohoServer,
            6 => PmProfile::AppliancePc,
            7 => PmProfile::PerformanceServer,
            8 => PmProfile::Tablet,
            _ => PmProfile::Unspecified,
        }
    }
}

/// ACPI system information
pub struct AcpiInfo {
    pub rsdp_address: u64,
    pub revision: u8,
    pub pm_profile: PmProfile,
}

impl AcpiInfo {
    pub const fn new() -> Self {
        Self {
            rsdp_address: 0,
            revision: 0,
            pm_profile: PmProfile::Unspecified,
        }
    }
}

/// Global ACPI information
static mut ACPI_INFO: AcpiInfo = AcpiInfo::new();

/// Search for RSDP in memory
unsafe fn find_rsdp() -> Option<u64> {
    // RSDP is typically located in:
    // 1. First KB of EBDA (Extended BIOS Data Area)
    // 2. BIOS read-only memory space 0xE0000 to 0xFFFFF

    // Check EBDA (first 1KB)
    let ebda_ptr = *(0x40E as *const u16) as u64;
    let ebda_start = (ebda_ptr << 4) as usize;
    
    if ebda_start != 0 {
        if let Some(addr) = search_rsdp(ebda_start, 1024) {
            return Some(addr);
        }
    }

    // Check BIOS ROM area
    search_rsdp(0xE0000, 0x20000)
}

/// Search for RSDP in a memory region
unsafe fn search_rsdp(start: usize, length: usize) -> Option<u64> {
    let end = start + length;
    let mut addr = start;

    while addr < end - 16 {
        let ptr = addr as *const u8;
        
        // Check signature
        let mut matches = true;
        for i in 0..8 {
            if ptr.add(i).read() != RSDP_SIGNATURE[i] {
                matches = false;
                break;
            }
        }

        if matches {
            // Verify checksum
            let rsdp_ptr = addr as *const Rsdp;
            let rsdp = ptr::read(rsdp_ptr);
            
            let mut sum: u8 = 0;
            for i in 0..core::mem::size_of::<Rsdp>() {
                sum = sum.wrapping_add(ptr.add(i).read());
            }

            if sum == 0 {
                return Some(addr as u64);
            }
        }

        addr += 16; // RSDP is 16-byte aligned
    }

    None
}

/// Initialize ACPI subsystem
pub fn init() {
    rinux_kernel::printk::printk("Initializing ACPI...\n");

    unsafe {
        if let Some(rsdp_addr) = find_rsdp() {
            rinux_kernel::printk::printk("  ACPI: Found RSDP at address\n");
            
            let rsdp_ptr = rsdp_addr as *const Rsdp;
            let rsdp = ptr::read(rsdp_ptr);
            
            ACPI_INFO.rsdp_address = rsdp_addr;
            ACPI_INFO.revision = rsdp.revision;

            rinux_kernel::printk::printk("  ACPI: Revision ");
            if rsdp.revision >= 2 {
                rinux_kernel::printk::printk("2.0+\n");
            } else {
                rinux_kernel::printk::printk("1.0\n");
            }

            // Try to read FADT for power management profile
            if let Some(pm_profile) = read_pm_profile(&rsdp) {
                ACPI_INFO.pm_profile = pm_profile;
                
                rinux_kernel::printk::printk("  ACPI: Power Profile - ");
                match pm_profile {
                    PmProfile::Mobile => rinux_kernel::printk::printk("Mobile/Laptop\n"),
                    PmProfile::Desktop => rinux_kernel::printk::printk("Desktop\n"),
                    PmProfile::Tablet => rinux_kernel::printk::printk("Tablet\n"),
                    PmProfile::Workstation => rinux_kernel::printk::printk("Workstation\n"),
                    _ => rinux_kernel::printk::printk("Other\n"),
                }
            }
        } else {
            rinux_kernel::printk::printk("  ACPI: RSDP not found\n");
        }
    }
}

/// Read power management profile from FADT
unsafe fn read_pm_profile(rsdp: &Rsdp) -> Option<PmProfile> {
    // This is a simplified version - would need to parse RSDT/XSDT
    // and find the FADT table
    
    // For now, default to Mobile for laptop support
    Some(PmProfile::Mobile)
}

/// Get ACPI info
pub fn get_info() -> &'static AcpiInfo {
    unsafe { &ACPI_INFO }
}

/// Check if system is a laptop
pub fn is_laptop() -> bool {
    unsafe { 
        matches!(ACPI_INFO.pm_profile, PmProfile::Mobile | PmProfile::Tablet)
    }
}
