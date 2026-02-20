//! Intel Integrated Graphics Driver
//!
//! Support for Intel HD/Iris/UHD Graphics found in most laptops.

use crate::pci::PciDevice;
use core::ptr;

/// Intel graphics device generations
#[derive(Debug, Clone, Copy)]
pub enum IntelGeneration {
    Gen6,  // Sandy Bridge
    Gen7,  // Ivy Bridge, Haswell
    Gen8,  // Broadwell
    Gen9,  // Skylake, Kaby Lake, Coffee Lake
    Gen11, // Ice Lake
    Gen12, // Tiger Lake, Alder Lake
    Unknown,
}

/// Intel graphics device
pub struct IntelGpu {
    pci_device: PciDevice,
    generation: IntelGeneration,
    gtt_base: u64,
    mmio_base: u64,
}

impl IntelGpu {
    /// Create new Intel GPU instance
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

        // Get GTT base from BAR2 (Graphics Translation Table)
        let bar2 = pci_device.bars[2];
        let gtt_base = if bar2 != 0 {
            let base = (bar2 & !0xF) as u64;
            if (bar2 & 0x4) != 0 {
                base | ((pci_device.bars[3] as u64) << 32)
            } else {
                base
            }
        } else {
            0
        };

        let generation = Self::detect_generation(pci_device.device_id);

        Ok(Self {
            pci_device: *pci_device,
            generation,
            gtt_base,
            mmio_base,
        })
    }

    /// Detect Intel GPU generation from device ID
    fn detect_generation(device_id: u16) -> IntelGeneration {
        match device_id {
            // Gen6 (Sandy Bridge)
            0x0102..=0x016A => IntelGeneration::Gen6,
            // Gen7 (Ivy Bridge, Haswell)
            0x0162..=0x0426 => IntelGeneration::Gen7,
            // Gen8 (Broadwell)
            0x1602..=0x163E => IntelGeneration::Gen8,
            // Gen9 (Skylake, Kaby Lake, Coffee Lake)
            0x1902..=0x3EA9 => IntelGeneration::Gen9,
            // Gen11 (Ice Lake)
            0x8A50..=0x8A71 => IntelGeneration::Gen11,
            // Gen12 (Tiger Lake, Alder Lake, Raptor Lake)
            0x4C80..=0x4C9A | 0x9A40..=0x9AF8 | 0x4680..=0x46D2 | 0xA780..=0xA7A1 => IntelGeneration::Gen12,
            _ => IntelGeneration::Unknown,
        }
    }

    /// Read MMIO register
    unsafe fn read_mmio(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 {
            return 0;
        }
        let addr = (self.mmio_base + offset as u64) as *const u32;
        ptr::read_volatile(addr)
    }

    /// Write MMIO register
    unsafe fn write_mmio(&self, offset: u32, value: u32) {
        if self.mmio_base == 0 {
            return;
        }
        let addr = (self.mmio_base + offset as u64) as *mut u32;
        ptr::write_volatile(addr, value);
    }

    /// Get generation name
    pub fn generation_name(&self) -> &'static str {
        match self.generation {
            IntelGeneration::Gen6 => "Gen6 (Sandy Bridge)",
            IntelGeneration::Gen7 => "Gen7 (Ivy Bridge/Haswell)",
            IntelGeneration::Gen8 => "Gen8 (Broadwell)",
            IntelGeneration::Gen9 => "Gen9 (Skylake/Kaby Lake)",
            IntelGeneration::Gen11 => "Gen11 (Ice Lake)",
            IntelGeneration::Gen12 => "Gen12 (Tiger Lake+)",
            IntelGeneration::Unknown => "Unknown Generation",
        }
    }

    /// Initialize the GPU
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Enable bus mastering and memory access
        self.pci_device.enable_bus_mastering();
        self.pci_device.enable_memory_space();

        rinux_kernel::printk::printk("    Intel GPU Generation: ");
        rinux_kernel::printk::printk(self.generation_name());
        rinux_kernel::printk::printk("\n");

        // Basic initialization would go here
        // - Setup display pipes
        // - Configure displays
        // - Initialize graphics context
        // - Setup command ring buffers

        Ok(())
    }
}

/// Initialize Intel graphics device
pub fn init_device(pci_device: &PciDevice) -> Result<(), &'static str> {
    rinux_kernel::printk::printk("    Initializing Intel GPU...\n");

    let mut gpu = IntelGpu::new(pci_device)?;
    gpu.init()?;

    rinux_kernel::printk::printk("    Intel GPU initialized successfully\n");
    Ok(())
}

/// Common Intel GPU device IDs for reference
pub const INTEL_DEVICE_IDS: &[(u16, &str)] = &[
    // Skylake
    (0x1912, "Intel HD Graphics 530"),
    (0x191B, "Intel HD Graphics 530"),
    (0x1916, "Intel HD Graphics 520"),
    (0x1926, "Intel Iris Graphics 540"),
    // Kaby Lake
    (0x5912, "Intel HD Graphics 630"),
    (0x5916, "Intel HD Graphics 620"),
    (0x5926, "Intel Iris Plus Graphics 640"),
    // Coffee Lake
    (0x3E92, "Intel UHD Graphics 630"),
    (0x3E91, "Intel UHD Graphics 630"),
    // Ice Lake
    (0x8A52, "Intel Iris Plus Graphics G7"),
    (0x8A5C, "Intel Iris Plus Graphics G7"),
    // Tiger Lake
    (0x9A49, "Intel Iris Xe Graphics"),
    (0x9A40, "Intel UHD Graphics"),
    // Alder Lake
    (0x4680, "Intel UHD Graphics"),
    (0x4682, "Intel UHD Graphics"),
    (0x46A6, "Intel Iris Xe Graphics"),
    // Raptor Lake
    (0xA780, "Intel UHD Graphics 770"),
    (0xA781, "Intel UHD Graphics 770"),
];
