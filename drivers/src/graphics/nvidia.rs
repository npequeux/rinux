//! NVIDIA Graphics Driver
//!
//! Support for NVIDIA GeForce/Quadro graphics.

use crate::pci::PciDevice;

/// NVIDIA GPU architectures
#[derive(Debug, Clone, Copy)]
pub enum NvidiaArchitecture {
    Maxwell, // GTX 900 series
    Pascal,  // GTX 1000 series
    Turing,  // RTX 2000 series
    Ampere,  // RTX 3000 series
    Ada,     // RTX 4000 series
    Unknown,
}

/// Detect NVIDIA graphics device
pub fn detect_device(pci_device: &PciDevice) {
    let device_id = pci_device.device_id;

    let architecture = match device_id {
        // Ada Lovelace (RTX 4000 series)
        0x2680..=0x27FF => NvidiaArchitecture::Ada,
        // Ampere (RTX 3000 series)
        0x2200..=0x25FF => NvidiaArchitecture::Ampere,
        // Turing (RTX 2000 series, GTX 1600 series)
        0x1E00..=0x21FF => NvidiaArchitecture::Turing,
        // Pascal (GTX 1000 series)
        0x1B00..=0x1DFF => NvidiaArchitecture::Pascal,
        // Maxwell (GTX 900 series)
        0x1340..=0x17FF => NvidiaArchitecture::Maxwell,
        _ => NvidiaArchitecture::Unknown,
    };

    let arch_name = match architecture {
        NvidiaArchitecture::Maxwell => "Maxwell (GTX 900 series)",
        NvidiaArchitecture::Pascal => "Pascal (GTX 1000 series)",
        NvidiaArchitecture::Turing => "Turing (RTX 2000 series)",
        NvidiaArchitecture::Ampere => "Ampere (RTX 3000 series)",
        NvidiaArchitecture::Ada => "Ada Lovelace (RTX 4000 series)",
        NvidiaArchitecture::Unknown => "Unknown",
    };

    rinux_kernel::printk::printk("    NVIDIA GPU Architecture: ");
    rinux_kernel::printk::printk(arch_name);
    rinux_kernel::printk::printk("\n");
    rinux_kernel::printk::printk("    NVIDIA driver support not fully implemented yet\n");
}

/// Common NVIDIA laptop GPU device IDs
pub const NVIDIA_DEVICE_IDS: &[(u16, &str)] = &[
    // RTX 4000 Mobile
    (0x28E0, "NVIDIA GeForce RTX 4090 Laptop GPU"),
    (0x28E1, "NVIDIA GeForce RTX 4080 Laptop GPU"),
    (0x28E2, "NVIDIA GeForce RTX 4070 Laptop GPU"),
    (0x28E3, "NVIDIA GeForce RTX 4060 Laptop GPU"),
    // RTX 3000 Mobile
    (0x2520, "NVIDIA GeForce RTX 3080 Ti Laptop GPU"),
    (0x2523, "NVIDIA GeForce RTX 3070 Ti Laptop GPU"),
    (0x2560, "NVIDIA GeForce RTX 3080 Laptop GPU"),
    (0x2563, "NVIDIA GeForce RTX 3070 Laptop GPU"),
    (0x2520, "NVIDIA GeForce RTX 3060 Laptop GPU"),
    // RTX 2000 Mobile
    (0x1F10, "NVIDIA GeForce RTX 2080 SUPER Mobile"),
    (0x1F11, "NVIDIA GeForce RTX 2070 SUPER Mobile"),
    (0x1F50, "NVIDIA GeForce RTX 2060 Mobile"),
    // GTX 1000 Mobile
    (0x1C20, "NVIDIA GeForce GTX 1080 Mobile"),
    (0x1C60, "NVIDIA GeForce GTX 1070 Mobile"),
    (0x1C8D, "NVIDIA GeForce GTX 1060 Mobile"),
];
