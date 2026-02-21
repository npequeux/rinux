//! NVMe Driver
//!
//! Non-Volatile Memory Express driver for modern SSDs

use super::block::{BlockDevice, BLOCK_SIZE};
use alloc::string::String;

/// NVMe controller capabilities
#[repr(C)]
pub struct NvmeCapabilities {
    pub mqes: u16,      // Maximum Queue Entries Supported
    pub cqr: bool,      // Contiguous Queues Required
    pub ams: u8,        // Arbitration Mechanism Supported
    pub to: u8,         // Timeout
    pub dstrd: u8,      // Doorbell Stride
    pub nssrs: bool,    // NVM Subsystem Reset Supported
    pub css: u8,        // Command Sets Supported
    pub mpsmin: u8,     // Memory Page Size Minimum
    pub mpsmax: u8,     // Memory Page Size Maximum
}

/// NVMe Admin Command Opcodes
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum NvmeAdminOp {
    CreateIOCQ = 0x05,
    CreateIOSQ = 0x01,
    Identify = 0x06,
    SetFeatures = 0x09,
    GetFeatures = 0x0A,
}

/// NVMe I/O Command Opcodes
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum NvmeIoOp {
    Flush = 0x00,
    Write = 0x01,
    Read = 0x02,
}

/// NVMe Command
#[repr(C)]
pub struct NvmeCommand {
    pub cdw0: u32,      // Command Dword 0
    pub nsid: u32,      // Namespace ID
    pub cdw2: u32,
    pub cdw3: u32,
    pub mptr: u64,      // Metadata Pointer
    pub prp1: u64,      // PRP Entry 1
    pub prp2: u64,      // PRP Entry 2
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

impl NvmeCommand {
    pub const fn new() -> Self {
        NvmeCommand {
            cdw0: 0,
            nsid: 0,
            cdw2: 0,
            cdw3: 0,
            mptr: 0,
            prp1: 0,
            prp2: 0,
            cdw10: 0,
            cdw11: 0,
            cdw12: 0,
            cdw13: 0,
            cdw14: 0,
            cdw15: 0,
        }
    }
}

/// NVMe Completion Queue Entry
#[repr(C)]
pub struct NvmeCompletion {
    pub result: u32,
    pub _reserved: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub cid: u16,
    pub status: u16,
}

/// NVMe namespace
pub struct NvmeNamespace {
    id: u32,
    size_blocks: u64,
    block_size: usize,
}

impl NvmeNamespace {
    pub const fn new(id: u32) -> Self {
        NvmeNamespace {
            id,
            size_blocks: 0,
            block_size: BLOCK_SIZE,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

/// NVMe device
pub struct NvmeDevice {
    name: String,
    namespace: NvmeNamespace,
    _base_address: u64,
}

impl NvmeDevice {
    /// Create a new NVMe device
    pub fn new(namespace_id: u32, base_address: u64) -> Self {
        NvmeDevice {
            name: alloc::format!("nvme{}n{}", 0, namespace_id),
            namespace: NvmeNamespace::new(namespace_id),
            _base_address: base_address,
        }
    }

    /// Identify namespace
    pub fn identify(&mut self) -> Result<(), &'static str> {
        // TODO: Send IDENTIFY NAMESPACE command
        // Parse the identify data to get:
        // - Namespace size
        // - Block size
        // - Features
        
        Err("Not implemented")
    }
}

impl BlockDevice for NvmeDevice {
    fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implement NVMe read
        // 1. Allocate PRP (Physical Region Pages) for buffer
        // 2. Create NVMe read command
        // 3. Submit to I/O submission queue
        // 4. Ring doorbell
        // 5. Wait for completion in completion queue
        // 6. Check status and copy data
        
        let _ = (start_block, buffer);
        Err("Not implemented")
    }

    fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implement NVMe write
        // Similar to read but with WRITE command
        
        let _ = (start_block, buffer);
        Err("Not implemented")
    }

    fn flush(&self) -> Result<(), &'static str> {
        // TODO: Send FLUSH command
        Ok(())
    }

    fn block_count(&self) -> u64 {
        self.namespace.size_blocks
    }

    fn block_size(&self) -> usize {
        self.namespace.block_size
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// NVMe controller
pub struct NvmeController {
    _base_address: u64,
    namespaces: alloc::vec::Vec<NvmeNamespace>,
}

impl NvmeController {
    /// Create a new NVMe controller
    pub fn new(base_address: u64) -> Self {
        NvmeController {
            _base_address: base_address,
            namespaces: alloc::vec::Vec::new(),
        }
    }

    /// Initialize NVMe controller
    pub fn init(&mut self) -> Result<(), &'static str> {
        // TODO: Full NVMe initialization sequence
        // 1. Read controller capabilities
        // 2. Configure admin queue
        // 3. Enable controller (CC.EN = 1)
        // 4. Wait for ready (CSTS.RDY = 1)
        // 5. Identify controller
        // 6. Set number of queues
        // 7. Create I/O queues
        // 8. Identify namespaces
        
        self.identify_namespaces()?;
        Ok(())
    }

    /// Identify all namespaces
    fn identify_namespaces(&mut self) -> Result<(), &'static str> {
        // TODO: Send IDENTIFY to get namespace list
        // For now, assume one namespace
        self.namespaces.push(NvmeNamespace::new(1));
        Ok(())
    }

    /// Get namespace by ID
    pub fn get_namespace(&self, ns_id: u32) -> Option<&NvmeNamespace> {
        self.namespaces.iter().find(|ns| ns.id() == ns_id)
    }
}

/// Initialize NVMe driver
pub fn init(pci_base_address: u64) -> Result<NvmeController, &'static str> {
    let mut controller = NvmeController::new(pci_base_address);
    controller.init()?;
    Ok(controller)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvme_device_creation() {
        let device = NvmeDevice::new(1, 0x1000);
        assert_eq!(device.name, "nvme0n1");
        assert_eq!(device.namespace.id(), 1);
    }

    #[test]
    fn test_nvme_command_size() {
        // NVMe command must be exactly 64 bytes
        assert_eq!(core::mem::size_of::<NvmeCommand>(), 64);
    }

    #[test]
    fn test_nvme_completion_size() {
        // NVMe completion must be exactly 16 bytes
        assert_eq!(core::mem::size_of::<NvmeCompletion>(), 16);
    }
}
