//! AHCI/SATA Driver
//!
//! Advanced Host Controller Interface for SATA drives

use super::block::{BlockDevice, BLOCK_SIZE};
use alloc::string::String;

/// AHCI HBA (Host Bus Adapter) registers
#[repr(C)]
pub struct AhciHba {
    // Generic Host Control
    cap: u32,      // Host Capabilities
    ghc: u32,      // Global Host Control
    is: u32,       // Interrupt Status
    pi: u32,       // Ports Implemented
    vs: u32,       // Version
    ccc_ctl: u32,  // Command Completion Coalescing Control
    ccc_ports: u32, // Command Completion Coalescing Ports
    em_loc: u32,   // Enclosure Management Location
    em_ctl: u32,   // Enclosure Management Control
    cap2: u32,     // Host Capabilities Extended
    bohc: u32,     // BIOS/OS Handoff Control and Status
}

/// AHCI port registers
#[repr(C)]
pub struct AhciPort {
    clb: u64,       // Command List Base Address
    fb: u64,        // FIS Base Address
    is: u32,        // Interrupt Status
    ie: u32,        // Interrupt Enable
    cmd: u32,       // Command and Status
    _reserved0: u32,
    tfd: u32,       // Task File Data
    sig: u32,       // Signature
    ssts: u32,      // Serial ATA Status
    sctl: u32,      // Serial ATA Control
    serr: u32,      // Serial ATA Error
    sact: u32,      // Serial ATA Active
    ci: u32,        // Command Issue
    sntf: u32,      // Serial ATA Notification
    fbs: u32,       // FIS-based Switching Control
}

/// SATA device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SataDeviceType {
    None,
    SATA,
    SATAPI,
    PM,      // Port Multiplier
    SEMB,    // Enclosure Management Bridge
}

/// AHCI SATA device
pub struct AhciDevice {
    name: String,
    port: usize,
    device_type: SataDeviceType,
    sector_count: u64,
    base_address: u64,
}

impl AhciDevice {
    /// Create a new AHCI device
    pub fn new(port: usize, base_address: u64) -> Self {
        AhciDevice {
            name: alloc::format!("sd{}", (b'a' + port as u8) as char),
            port,
            device_type: SataDeviceType::None,
            sector_count: 0,
            base_address,
        }
    }

    /// Probe and identify the device
    pub fn probe(&mut self) -> Result<(), &'static str> {
        // TODO: Implement actual AHCI probing
        // This would involve:
        // 1. Check port signature to identify device type
        // 2. Send IDENTIFY command
        // 3. Parse IDENTIFY data to get sector count and features
        // 4. Set up command list and FIS structures
        
        // For now, stub implementation
        self.device_type = SataDeviceType::SATA;
        self.sector_count = 0; // Unknown
        
        Ok(())
    }

    /// Check if device is present
    pub fn is_present(&self) -> bool {
        self.device_type != SataDeviceType::None
    }
}

impl BlockDevice for AhciDevice {
    fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.is_present() {
            return Err("Device not present");
        }

        // TODO: Implement actual AHCI read
        // This would involve:
        // 1. Set up command FIS (Frame Information Structure)
        // 2. Set up PRDT (Physical Region Descriptor Table)
        // 3. Issue command to port
        // 4. Wait for completion
        // 5. Copy data from DMA buffer to user buffer
        
        // For now, return error
        let _ = (start_block, buffer);
        Err("Not implemented")
    }

    fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<usize, &'static str> {
        if !self.is_present() {
            return Err("Device not present");
        }

        // TODO: Implement actual AHCI write
        // Similar to read but with WRITE DMA command
        
        let _ = (start_block, buffer);
        Err("Not implemented")
    }

    fn flush(&self) -> Result<(), &'static str> {
        if !self.is_present() {
            return Err("Device not present");
        }

        // TODO: Send FLUSH CACHE command
        Ok(())
    }

    fn block_count(&self) -> u64 {
        self.sector_count
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// AHCI controller
pub struct AhciController {
    base_address: u64,
    ports: alloc::vec::Vec<Option<AhciDevice>>,
}

impl AhciController {
    /// Create a new AHCI controller
    pub fn new(base_address: u64) -> Self {
        AhciController {
            base_address,
            ports: alloc::vec::Vec::new(),
        }
    }

    /// Initialize AHCI controller
    pub fn init(&mut self) -> Result<(), &'static str> {
        // TODO: Implement full AHCI initialization
        // 1. Enable AHCI mode (set GHC.AE)
        // 2. Reset HBA (set GHC.HR)
        // 3. Wait for reset to complete
        // 4. Enable interrupts (set GHC.IE)
        // 5. Probe all implemented ports
        
        self.probe_ports()?;
        Ok(())
    }

    /// Probe all ports
    fn probe_ports(&mut self) -> Result<(), &'static str> {
        // TODO: Read PI (Ports Implemented) register
        // For now, assume 6 ports (common AHCI config)
        for port in 0..6 {
            let mut device = AhciDevice::new(port, self.base_address);
            if device.probe().is_ok() && device.is_present() {
                self.ports.push(Some(device));
            } else {
                self.ports.push(None);
            }
        }
        Ok(())
    }

    /// Get device on port
    pub fn get_device(&self, port: usize) -> Option<&AhciDevice> {
        self.ports.get(port).and_then(|d| d.as_ref())
    }

    /// Get mutable device on port
    pub fn get_device_mut(&mut self, port: usize) -> Option<&mut AhciDevice> {
        self.ports.get_mut(port).and_then(|d| d.as_mut())
    }
}

/// Initialize AHCI driver
pub fn init(pci_base_address: u64) -> Result<AhciController, &'static str> {
    let mut controller = AhciController::new(pci_base_address);
    controller.init()?;
    Ok(controller)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ahci_device_creation() {
        let device = AhciDevice::new(0, 0x1000);
        assert_eq!(device.name, "sda");
        assert_eq!(device.port, 0);
    }

    #[test]
    fn test_device_type() {
        let mut device = AhciDevice::new(0, 0x1000);
        assert_eq!(device.device_type, SataDeviceType::None);
        assert!(!device.is_present());
    }
}
