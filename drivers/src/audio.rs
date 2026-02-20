//! Audio Subsystem
//!
//! High Definition Audio (HDA/Azalia) support for laptop audio.

use crate::pci::{PciDevice, PciClass};
use core::ptr;

/// Intel HDA vendor/device IDs
pub const INTEL_HDA_DEVICES: &[(u16, &str)] = &[
    (0x2668, "Intel HDA - ICH6"),
    (0x27D8, "Intel HDA - ICH7"),
    (0x284B, "Intel HDA - ICH8"),
    (0x293E, "Intel HDA - ICH9"),
    (0x3A3E, "Intel HDA - ICH10"),
    (0x3B56, "Intel HDA - PCH"),
    (0x1C20, "Intel HDA - PCH"),
    (0x1E20, "Intel HDA - PCH"),
    (0x8C20, "Intel HDA - Lynx Point"),
    (0x9C20, "Intel HDA - Lynx Point-LP"),
    (0xA170, "Intel HDA - Sunrise Point"),
    (0x9D70, "Intel HDA - Sunrise Point-LP"),
    (0xA348, "Intel HDA - Cannon Lake"),
    (0x9DC8, "Intel HDA - Cannon Point-LP"),
    (0x02C8, "Intel HDA - Comet Lake"),
    (0x06C8, "Intel HDA - Comet Lake"),
    (0xF0C8, "Intel HDA - Tiger Lake"),
    (0x43C8, "Intel HDA - Tiger Lake-LP"),
    (0x4DC8, "Intel HDA - Jasper Lake"),
    (0x51C8, "Intel HDA - Alder Lake"),
    (0x54C8, "Intel HDA - Alder Lake"),
    (0x7AD0, "Intel HDA - Raptor Lake"),
];

/// HDA register offsets
const HDA_GCAP: u32 = 0x00;     // Global Capabilities
const HDA_VMIN: u32 = 0x02;     // Minor Version
const HDA_VMAJ: u32 = 0x03;     // Major Version
const HDA_GCTL: u32 = 0x08;     // Global Control
const HDA_STATESTS: u32 = 0x0E; // State Change Status
const HDA_INTCTL: u32 = 0x20;   // Interrupt Control
const HDA_INTSTS: u32 = 0x24;   // Interrupt Status

/// HDA Global Control register bits
const HDA_GCTL_RESET: u32 = 1 << 0;
const HDA_GCTL_ACCEPT_UNSOL: u32 = 1 << 8;

/// Audio codec information
#[derive(Debug, Clone, Copy)]
pub struct CodecInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub name: &'static str,
}

/// HDA (High Definition Audio) controller
pub struct HdaController {
    pci_device: PciDevice,
    mmio_base: u64,
    num_codecs: u8,
}

impl HdaController {
    /// Create new HDA controller
    pub fn new(pci_device: &PciDevice) -> Result<Self, &'static str> {
        // Get MMIO base from BAR0
        let bar0 = pci_device.bars[0];
        if bar0 == 0 || (bar0 & 1) == 1 {
            return Err("Invalid BAR0");
        }

        let mmio_base = (bar0 & !0xF) as u64;
        let mmio_base = if (bar0 & 0x4) != 0 {
            mmio_base | ((pci_device.bars[1] as u64) << 32)
        } else {
            mmio_base
        };

        if mmio_base == 0 {
            return Err("Invalid MMIO base");
        }

        Ok(Self {
            pci_device: *pci_device,
            mmio_base,
            num_codecs: 0,
        })
    }

    /// Read HDA register
    unsafe fn read_reg(&self, offset: u32) -> u32 {
        let addr = (self.mmio_base + offset as u64) as *const u32;
        ptr::read_volatile(addr)
    }

    /// Write HDA register
    unsafe fn write_reg(&self, offset: u32, value: u32) {
        let addr = (self.mmio_base + offset as u64) as *mut u32;
        ptr::write_volatile(addr, value);
    }

    /// Reset the controller
    unsafe fn reset(&mut self) -> Result<(), &'static str> {
        // Clear reset bit
        self.write_reg(HDA_GCTL, 0);

        // Wait for reset
        for _ in 0..1000 {
            if self.read_reg(HDA_GCTL) & HDA_GCTL_RESET == 0 {
                break;
            }
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }

        // Set reset bit
        self.write_reg(HDA_GCTL, HDA_GCTL_RESET);

        // Wait for controller to be ready
        for _ in 0..1000 {
            if self.read_reg(HDA_GCTL) & HDA_GCTL_RESET != 0 {
                return Ok(());
            }
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }

        Err("Reset timeout")
    }

    /// Detect codecs
    unsafe fn detect_codecs(&mut self) {
        let statests = self.read_reg(HDA_STATESTS);
        self.num_codecs = 0;

        for i in 0..15 {
            if (statests & (1 << i)) != 0 {
                self.num_codecs += 1;
            }
        }
    }

    /// Initialize the controller
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Enable bus mastering and memory access
        self.pci_device.enable_bus_mastering();
        self.pci_device.enable_memory_space();

        unsafe {
            // Get version
            let vmaj = self.read_reg(HDA_VMAJ);
            let vmin = self.read_reg(HDA_VMIN);

            rinux_kernel::printk::printk("    HDA Version: ");
            // TODO: Print version numbers
            rinux_kernel::printk::printk("\n");

            // Reset controller
            self.reset()?;

            // Detect codecs
            self.detect_codecs();

            rinux_kernel::printk::printk("    Found ");
            // TODO: Print codec count
            rinux_kernel::printk::printk(" audio codec(s)\n");
        }

        Ok(())
    }
}

/// Common audio codec vendors
pub const CODEC_VENDORS: &[(u16, &str)] = &[
    (0x1002, "AMD/ATI"),
    (0x10EC, "Realtek"),
    (0x1106, "VIA"),
    (0x111D, "IDT"),
    (0x11D4, "Analog Devices"),
    (0x13F6, "C-Media"),
    (0x14F1, "Conexant"),
    (0x434D, "C-Media"),
    (0x8086, "Intel"),
    (0x8384, "SigmaTel"),
];

/// Initialize audio subsystem
pub fn init() {
    rinux_kernel::printk::printk("  Initializing audio subsystem...\n");

    let scanner = crate::pci::scanner();
    let mut found_audio = false;

    // Look for audio controllers
    for device in scanner.find_by_class(PciClass::MultimediaController) {
        // Check if it's an audio controller (subclass 0x01)
        if device.subclass == 0x01 {
            found_audio = true;

            // Check for Intel HDA
            let is_intel_hda = device.vendor_id == 0x8086 && 
                INTEL_HDA_DEVICES.iter().any(|(id, _)| *id == device.device_id);

            if is_intel_hda {
                rinux_kernel::printk::printk("    Found Intel HDA audio controller\n");
                
                match HdaController::new(device) {
                    Ok(mut controller) => {
                        if let Err(e) = controller.init() {
                            rinux_kernel::printk::printk("      HDA init failed: ");
                            rinux_kernel::printk::printk(e);
                            rinux_kernel::printk::printk("\n");
                        }
                    }
                    Err(e) => {
                        rinux_kernel::printk::printk("      Failed to create HDA controller: ");
                        rinux_kernel::printk::printk(e);
                        rinux_kernel::printk::printk("\n");
                    }
                }
            } else {
                rinux_kernel::printk::printk("    Found audio controller (vendor: ");
                // TODO: Print vendor/device ID
                rinux_kernel::printk::printk(")\n");
            }
        }
    }

    if !found_audio {
        rinux_kernel::printk::printk("    No audio controllers found\n");
    }
}
