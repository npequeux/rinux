//! Graphics Subsystem
//!
//! Graphics drivers for modern laptop GPUs.

pub mod amd;
pub mod framebuffer;
pub mod intel;
pub mod nvidia;

use crate::pci::{PciClass, PciDevice};

/// GPU vendor IDs
pub const VENDOR_INTEL: u16 = 0x8086;
pub const VENDOR_AMD: u16 = 0x1002;
pub const VENDOR_NVIDIA: u16 = 0x10DE;

/// GPU type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuType {
    Intel,
    AMD,
    Nvidia,
    Other,
}

/// GPU information
#[derive(Debug, Clone, Copy)]
pub struct GpuInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub gpu_type: GpuType,
    pub name: &'static str,
}

impl GpuInfo {
    pub fn from_pci_device(device: &PciDevice) -> Option<Self> {
        if device.class != PciClass::DisplayController {
            return None;
        }

        let gpu_type = match device.vendor_id {
            VENDOR_INTEL => GpuType::Intel,
            VENDOR_AMD => GpuType::AMD,
            VENDOR_NVIDIA => GpuType::Nvidia,
            _ => GpuType::Other,
        };

        let name = match gpu_type {
            GpuType::Intel => "Intel Integrated Graphics",
            GpuType::AMD => "AMD Radeon Graphics",
            GpuType::Nvidia => "NVIDIA Graphics",
            GpuType::Other => "Unknown GPU",
        };

        Some(GpuInfo {
            vendor_id: device.vendor_id,
            device_id: device.device_id,
            gpu_type,
            name,
        })
    }
}

/// Initialize graphics subsystem
pub fn init() {
    rinux_kernel::printk::printk("Initializing graphics subsystem...\n");

    // Initialize framebuffer
    framebuffer::init();

    // Detect GPUs via PCI
    let scanner = crate::pci::scanner();

    for device in scanner.find_by_class(PciClass::DisplayController) {
        if let Some(gpu_info) = GpuInfo::from_pci_device(device) {
            rinux_kernel::printk::printk("  Found GPU: ");
            rinux_kernel::printk::printk(gpu_info.name);
            rinux_kernel::printk::printk("\n");

            // Initialize GPU-specific driver
            match gpu_info.gpu_type {
                GpuType::Intel => {
                    if let Err(e) = intel::init_device(device) {
                        rinux_kernel::printk::printk("    Intel GPU init failed: ");
                        rinux_kernel::printk::printk(e);
                        rinux_kernel::printk::printk("\n");
                    }
                }
                GpuType::AMD => {
                    amd::detect_device(device);
                }
                GpuType::Nvidia => {
                    nvidia::detect_device(device);
                }
                GpuType::Other => {}
            }
        }
    }
}
