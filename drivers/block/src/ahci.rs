//! AHCI (Advanced Host Controller Interface) Driver
//!
//! Driver for SATA devices using AHCI with interrupt support

use crate::device::{BlockDevice, BlockDeviceError};
use crate::ahci_irq::{add_pending_io, wait_for_completion, enable_port_interrupts};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

/// AHCI PCI Class/Subclass
pub const AHCI_PCI_CLASS: u8 = 0x01;  // Mass Storage Controller
pub const AHCI_PCI_SUBCLASS: u8 = 0x06;  // SATA Controller

/// PCI BAR memory/IO space indicator
const PCI_BAR_MEMORY_SPACE: u32 = 0x1;

/// Maximum PCI bus number to scan (avoid excessive boot delay)
const MAX_PCI_BUS: u16 = 256;

/// AHCI HBA (Host Bus Adapter) Registers
#[repr(C)]
struct HbaRegisters {
    capability: u32,
    global_host_control: u32,
    interrupt_status: u32,
    ports_implemented: u32,
    version: u32,
    ccc_control: u32,
    ccc_ports: u32,
    em_location: u32,
    em_control: u32,
    capability2: u32,
    bohc: u32,
}

/// AHCI Port Registers
#[repr(C)]
struct PortRegisters {
    command_list_base: u32,
    command_list_base_upper: u32,
    fis_base: u32,
    fis_base_upper: u32,
    interrupt_status: u32,
    interrupt_enable: u32,
    command_and_status: u32,
    _reserved: u32,
    task_file_data: u32,
    signature: u32,
    sata_status: u32,
    sata_control: u32,
    sata_error: u32,
    sata_active: u32,
    command_issue: u32,
    sata_notification: u32,
    fis_based_switching: u32,
}

/// FIS Type
#[repr(u8)]
#[allow(dead_code)]
enum FisType {
    RegH2D = 0x27,      // Register FIS - host to device
    RegD2H = 0x34,      // Register FIS - device to host
    DmaActivate = 0x39, // DMA activate FIS
    DmaSetup = 0x41,    // DMA setup FIS
    Data = 0x46,        // Data FIS
    Bist = 0x58,        // BIST activate FIS
    PioSetup = 0x5F,    // PIO setup FIS
    SetDevBits = 0xA1,  // Set device bits FIS
}

/// Command FIS (Frame Information Structure)
#[repr(C, packed)]
struct CommandFis {
    fis_type: u8,    // FisType::RegH2D
    flags: u8,       // Bit 7: Command (1) / Control (0)
    command: u8,     // ATA command
    features_low: u8,
    
    lba_0: u8,       // LBA bits 0-7
    lba_1: u8,       // LBA bits 8-15
    lba_2: u8,       // LBA bits 16-23
    device: u8,      // Device register
    
    lba_3: u8,       // LBA bits 24-31
    lba_4: u8,       // LBA bits 32-39
    lba_5: u8,       // LBA bits 40-47
    features_high: u8,
    
    count_low: u8,   // Sector count low
    count_high: u8,  // Sector count high
    icc: u8,         // Isochronous command completion
    control: u8,
    
    _reserved: [u8; 4],
}

/// Command Header (one per command slot, 32 bytes)
#[repr(C, packed)]
struct CommandHeader {
    flags: u16,              // Command flags (DW0)
    prdtl: u16,              // Physical Region Descriptor Table Length
    prdbc: u32,              // Physical Region Descriptor Byte Count
    ctba: u32,               // Command Table Base Address (lower 32-bit)
    ctba_upper: u32,         // Command Table Base Address (upper 32-bit)
    _reserved: [u32; 4],
}

/// Physical Region Descriptor Table Entry (PRDT entry, 16 bytes)
#[repr(C, packed)]
struct PrdtEntry {
    dba: u32,        // Data Base Address (lower 32-bit)
    dba_upper: u32,  // Data Base Address (upper 32-bit)
    _reserved: u32,
    dbc: u32,        // Data Byte Count (bit 0-21), Interrupt on completion (bit 31)
}

/// Command Table (128 bytes aligned, variable size with PRDT)
#[repr(C, packed)]
struct CommandTable {
    cfis: [u8; 64],           // Command FIS
    acmd: [u8; 16],           // ATAPI Command
    _reserved: [u8; 48],
    // PRDT entries follow (variable size)
}

/// Command Header flags
const CMD_HEADER_FLAG_FIS_LENGTH: u16 = 5; // FIS length in DWORDs (5 * 4 = 20 bytes for H2D FIS)
const CMD_HEADER_FLAG_WRITE: u16 = 1 << 6; // Write (H2D)
const CMD_HEADER_FLAG_PREFETCHABLE: u16 = 1 << 7;
const CMD_HEADER_FLAG_CLEAR_BUSY: u16 = 1 << 10;

/// PRDT entry flags
const PRDT_INTERRUPT_ON_COMPLETION: u32 = 1 << 31;

/// Allocate aligned memory for DMA
fn allocate_aligned(size: usize, alignment: usize) -> Option<*mut u8> {
    use alloc::alloc::{alloc, Layout};
    
    let layout = Layout::from_size_align(size, alignment).ok()?;
    unsafe {
        let ptr = alloc(layout);
        if ptr.is_null() {
            None
        } else {
            // Zero the memory for DMA buffers
            core::ptr::write_bytes(ptr, 0, size);
            Some(ptr)
        }
    }
}

/// Get physical address from virtual address
/// 
/// # Safety
/// 
/// This function currently assumes identity mapping in kernel space.
/// In a production kernel with virtual memory, this would need to walk
/// the page tables to translate virtual to physical addresses.
fn virt_to_phys(virt: *const u8) -> u64 {
    // TODO: Implement proper page table walking for virtual-to-physical translation
    // For now, we assume identity mapping for kernel DMA buffers
    virt as u64
}

/// Build a READ DMA EXT command FIS
fn build_read_fis(lba: u64, count: u16) -> CommandFis {
    CommandFis {
        fis_type: FisType::RegH2D as u8,
        flags: 0x80, // Command bit set
        command: 0x25, // READ DMA EXT
        features_low: 0,
        
        lba_0: (lba & 0xFF) as u8,
        lba_1: ((lba >> 8) & 0xFF) as u8,
        lba_2: ((lba >> 16) & 0xFF) as u8,
        device: 0x40, // LBA mode
        
        lba_3: ((lba >> 24) & 0xFF) as u8,
        lba_4: ((lba >> 32) & 0xFF) as u8,
        lba_5: ((lba >> 40) & 0xFF) as u8,
        features_high: 0,
        
        count_low: (count & 0xFF) as u8,
        count_high: ((count >> 8) & 0xFF) as u8,
        icc: 0,
        control: 0,
        
        _reserved: [0; 4],
    }
}

/// Build a WRITE DMA EXT command FIS
fn build_write_fis(lba: u64, count: u16) -> CommandFis {
    CommandFis {
        fis_type: FisType::RegH2D as u8,
        flags: 0x80,
        command: 0x35, // WRITE DMA EXT
        features_low: 0,
        
        lba_0: (lba & 0xFF) as u8,
        lba_1: ((lba >> 8) & 0xFF) as u8,
        lba_2: ((lba >> 16) & 0xFF) as u8,
        device: 0x40,
        
        lba_3: ((lba >> 24) & 0xFF) as u8,
        lba_4: ((lba >> 32) & 0xFF) as u8,
        lba_5: ((lba >> 40) & 0xFF) as u8,
        features_high: 0,
        
        count_low: (count & 0xFF) as u8,
        count_high: ((count >> 8) & 0xFF) as u8,
        icc: 0,
        control: 0,
        
        _reserved: [0; 4],
    }
}

/// Port command and status register bits
const PORT_CMD_START: u32 = 1 << 0;        // Start (ST)
const PORT_CMD_FIS_RX_ENABLE: u32 = 1 << 4; // FIS Receive Enable (FRE)
const PORT_CMD_FIS_RX_RUNNING: u32 = 1 << 14; // FIS Receive Running (FR)
const PORT_CMD_CR: u32 = 1 << 15;          // Command List Running (CR)

/// AHCI Device
pub struct AhciDevice {
    name: String,
    port: usize,
    block_size: usize,
    num_blocks: u64,
    hba: *mut HbaRegisters,
}

unsafe impl Send for AhciDevice {}
unsafe impl Sync for AhciDevice {}

impl AhciDevice {
    /// Create a new AHCI device
    pub fn new(name: String, port: usize, hba: *mut HbaRegisters) -> Self {
        AhciDevice {
            name,
            port,
            block_size: 512,  // Standard sector size
            num_blocks: 0,    // Will be detected
            hba,
        }
    }

    /// Identify the device and get capacity
    fn identify(&mut self) -> Result<(), BlockDeviceError> {
        // This would send an ATA IDENTIFY command to the device
        // For now, we'll just set a default capacity
        self.num_blocks = 1024 * 1024 * 1024 / 512;  // 1GB default
        Ok(())
    }

    /// Issue a read command to the device
    fn read_dma(&self, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        // Get port registers
        let port = self.get_port_registers();
        
        // Stop command engine to set up new command
        self.stop_cmd(port)?;
        
        // Build READ DMA EXT command (0x25)
        let command_fis = build_read_fis(lba, count);
        
        // Set up command header and table
        let dma_buffer = self.setup_command_read(port, &command_fis, buffer.len())?;
        
        // Start command engine
        self.start_cmd(port)?;
        
        // Issue command
        unsafe {
            // Set command issue bit for slot 0
            core::ptr::write_volatile(&mut (*port).command_issue as *mut u32, 1);
        }
        
        // Wait for completion - only copy data on success
        self.wait_for_completion(port)?;
        
        // Copy data from DMA buffer to user buffer
        unsafe {
            core::ptr::copy_nonoverlapping(dma_buffer, buffer.as_mut_ptr(), buffer.len());
        }
        
        Ok(())
    }

    /// Issue a write command to the device
    fn write_dma(&self, lba: u64, count: u16, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        let port = self.get_port_registers();
        
        // Stop command engine to set up new command
        self.stop_cmd(port)?;
        
        // Build WRITE DMA EXT command
        let command_fis = build_write_fis(lba, count);
        
        // Set up command
        self.setup_command_write(port, &command_fis, buffer)?;
        
        // Start command engine
        self.start_cmd(port)?;
        
        // Issue command
        unsafe {
            core::ptr::write_volatile(&mut (*port).command_issue as *mut u32, 1);
        }
        
        // Wait for completion
        self.wait_for_completion(port)?;
        
        Ok(())
    }
    
    /// Get port registers for this device
    fn get_port_registers(&self) -> *mut PortRegisters {
        unsafe {
            let hba_mem = self.hba as *mut u8;
            // Ports start at offset 0x100, each port is 0x80 bytes
            let port_offset = 0x100 + (self.port * 0x80);
            hba_mem.add(port_offset) as *mut PortRegisters
        }
    }
    
    /// Start command engine on port
    fn start_cmd(&self, port: *mut PortRegisters) -> Result<(), BlockDeviceError> {
        unsafe {
            // Wait until CR (bit 15) is cleared
            let mut timeout = 1000;
            while (core::ptr::read_volatile(&(*port).command_and_status as *const u32) & PORT_CMD_CR) != 0 {
                timeout -= 1;
                if timeout == 0 {
                    return Err(BlockDeviceError::Timeout);
                }
                // Yield CPU to reduce busy-waiting overhead
                core::hint::spin_loop();
            }
            
            // Set FRE (bit 4) and ST (bit 0)
            let cmd = core::ptr::read_volatile(&(*port).command_and_status as *const u32);
            core::ptr::write_volatile(
                &mut (*port).command_and_status as *mut u32,
                cmd | PORT_CMD_FIS_RX_ENABLE | PORT_CMD_START,
            );
            
            Ok(())
        }
    }
    
    /// Stop command engine on port
    fn stop_cmd(&self, port: *mut PortRegisters) -> Result<(), BlockDeviceError> {
        unsafe {
            // Clear ST (bit 0)
            let mut cmd = core::ptr::read_volatile(&(*port).command_and_status as *const u32);
            cmd &= !PORT_CMD_START;
            core::ptr::write_volatile(&mut (*port).command_and_status as *mut u32, cmd);
            
            // Wait until FR (bit 14), CR (bit 15) are cleared
            let mut timeout = 1000;
            loop {
                let status = core::ptr::read_volatile(&(*port).command_and_status as *const u32);
                if (status & PORT_CMD_FIS_RX_RUNNING) == 0 && (status & PORT_CMD_CR) == 0 {
                    break;
                }
                timeout -= 1;
                if timeout == 0 {
                    return Err(BlockDeviceError::Timeout);
                }
                // Yield CPU to reduce busy-waiting overhead
                core::hint::spin_loop();
            }
            
            // Clear FRE (bit 4)
            cmd = core::ptr::read_volatile(&(*port).command_and_status as *const u32);
            cmd &= !PORT_CMD_FIS_RX_ENABLE;
            core::ptr::write_volatile(&mut (*port).command_and_status as *mut u32, cmd);
            
            Ok(())
        }
    }
    
    /// Set up command for DMA read transfer
    fn setup_command_read(
        &self,
        port: *mut PortRegisters,
        fis: &CommandFis,
        buffer_len: usize,
    ) -> Result<*mut u8, BlockDeviceError> {
        // Validate buffer length
        if buffer_len == 0 {
            return Err(BlockDeviceError::InvalidOffset);
        }
        
        unsafe {
            // Allocate command list (1K aligned, minimum 1024 bytes for 32 command slots)
            let cmd_list = allocate_aligned(1024, 1024).ok_or(BlockDeviceError::IoError)?;
            let cmd_header = cmd_list as *mut CommandHeader;
            
            // Allocate command table (128-byte aligned)
            let cmd_table_size = core::mem::size_of::<CommandTable>() + core::mem::size_of::<PrdtEntry>();
            let cmd_table_ptr = allocate_aligned(cmd_table_size, 128).ok_or(BlockDeviceError::IoError)?;
            let cmd_table = cmd_table_ptr as *mut CommandTable;
            
            // Allocate DMA buffer for data transfer (aligned to sector size)
            let dma_buffer = allocate_aligned(buffer_len, 512).ok_or(BlockDeviceError::IoError)?;
            
            // Fill in the Command FIS in the command table
            core::ptr::copy_nonoverlapping(
                fis as *const CommandFis as *const u8,
                (*cmd_table).cfis.as_mut_ptr(),
                core::mem::size_of::<CommandFis>(),
            );
            
            // Set up PRDT entry (located after CommandTable)
            let prdt_entry = (cmd_table_ptr as usize + core::mem::size_of::<CommandTable>()) as *mut PrdtEntry;
            let phys_addr = virt_to_phys(dma_buffer);
            
            core::ptr::write_volatile(&mut (*prdt_entry).dba as *mut u32, (phys_addr & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*prdt_entry).dba_upper as *mut u32, ((phys_addr >> 32) & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*prdt_entry)._reserved as *mut u32, 0);
            
            // Data byte count - 1 (0-based), with interrupt on completion
            let dbc = ((buffer_len - 1) as u32) | PRDT_INTERRUPT_ON_COMPLETION;
            core::ptr::write_volatile(&mut (*prdt_entry).dbc as *mut u32, dbc);
            
            // Fill in command header (read, so no write flag)
            let flags = CMD_HEADER_FLAG_FIS_LENGTH | CMD_HEADER_FLAG_PREFETCHABLE | CMD_HEADER_FLAG_CLEAR_BUSY;
            
            core::ptr::write_volatile(&mut (*cmd_header).flags as *mut u16, flags);
            core::ptr::write_volatile(&mut (*cmd_header).prdtl as *mut u16, 1);
            core::ptr::write_volatile(&mut (*cmd_header).prdbc as *mut u32, 0);
            
            // Set command table base address
            let cmd_table_phys = virt_to_phys(cmd_table_ptr);
            core::ptr::write_volatile(&mut (*cmd_header).ctba as *mut u32, (cmd_table_phys & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*cmd_header).ctba_upper as *mut u32, ((cmd_table_phys >> 32) & 0xFFFFFFFF) as u32);
            
            // Clear reserved fields
            for i in 0..4 {
                core::ptr::write_volatile(&mut (*cmd_header)._reserved[i] as *mut u32, 0);
            }
            
            // Set command list base address in port registers
            let cmd_list_phys = virt_to_phys(cmd_list);
            core::ptr::write_volatile(&mut (*port).command_list_base as *mut u32, (cmd_list_phys & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*port).command_list_base_upper as *mut u32, ((cmd_list_phys >> 32) & 0xFFFFFFFF) as u32);
            
            // Allocate and set up received FIS buffer (256 bytes, 256-byte aligned)
            let fis_buffer = allocate_aligned(256, 256).ok_or(BlockDeviceError::IoError)?;
            let fis_buffer_phys = virt_to_phys(fis_buffer);
            core::ptr::write_volatile(&mut (*port).fis_base as *mut u32, (fis_buffer_phys & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*port).fis_base_upper as *mut u32, ((fis_buffer_phys >> 32) & 0xFFFFFFFF) as u32);
            
            // TODO: Track allocated memory (cmd_list, cmd_table_ptr, dma_buffer, fis_buffer)
            // for cleanup after command completion to prevent memory leaks
            
            Ok(dma_buffer)
        }
    }
    
    /// Set up command for DMA write transfer
    fn setup_command_write(
        &self,
        port: *mut PortRegisters,
        fis: &CommandFis,
        buffer: &[u8],
    ) -> Result<(), BlockDeviceError> {
        // Validate buffer length
        if buffer.is_empty() {
            return Err(BlockDeviceError::InvalidOffset);
        }
        
        unsafe {
            // Allocate command list (1K aligned, minimum 1024 bytes for 32 command slots)
            let cmd_list = allocate_aligned(1024, 1024).ok_or(BlockDeviceError::IoError)?;
            let cmd_header = cmd_list as *mut CommandHeader;
            
            // Allocate command table (128-byte aligned)
            let cmd_table_size = core::mem::size_of::<CommandTable>() + core::mem::size_of::<PrdtEntry>();
            let cmd_table_ptr = allocate_aligned(cmd_table_size, 128).ok_or(BlockDeviceError::IoError)?;
            let cmd_table = cmd_table_ptr as *mut CommandTable;
            
            // Allocate DMA buffer for data transfer (aligned to sector size)
            let dma_buffer = allocate_aligned(buffer.len(), 512).ok_or(BlockDeviceError::IoError)?;
            
            // Copy data to DMA buffer for write
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), dma_buffer, buffer.len());
            
            // Fill in the Command FIS in the command table
            core::ptr::copy_nonoverlapping(
                fis as *const CommandFis as *const u8,
                (*cmd_table).cfis.as_mut_ptr(),
                core::mem::size_of::<CommandFis>(),
            );
            
            // Set up PRDT entry (located after CommandTable)
            let prdt_entry = (cmd_table_ptr as usize + core::mem::size_of::<CommandTable>()) as *mut PrdtEntry;
            let phys_addr = virt_to_phys(dma_buffer);
            
            core::ptr::write_volatile(&mut (*prdt_entry).dba as *mut u32, (phys_addr & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*prdt_entry).dba_upper as *mut u32, ((phys_addr >> 32) & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*prdt_entry)._reserved as *mut u32, 0);
            
            // Data byte count - 1 (0-based), with interrupt on completion
            let dbc = ((buffer.len() - 1) as u32) | PRDT_INTERRUPT_ON_COMPLETION;
            core::ptr::write_volatile(&mut (*prdt_entry).dbc as *mut u32, dbc);
            
            // Fill in command header (write, so set write flag)
            let flags = CMD_HEADER_FLAG_FIS_LENGTH | CMD_HEADER_FLAG_WRITE | CMD_HEADER_FLAG_PREFETCHABLE | CMD_HEADER_FLAG_CLEAR_BUSY;
            
            core::ptr::write_volatile(&mut (*cmd_header).flags as *mut u16, flags);
            core::ptr::write_volatile(&mut (*cmd_header).prdtl as *mut u16, 1);
            core::ptr::write_volatile(&mut (*cmd_header).prdbc as *mut u32, 0);
            
            // Set command table base address
            let cmd_table_phys = virt_to_phys(cmd_table_ptr);
            core::ptr::write_volatile(&mut (*cmd_header).ctba as *mut u32, (cmd_table_phys & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*cmd_header).ctba_upper as *mut u32, ((cmd_table_phys >> 32) & 0xFFFFFFFF) as u32);
            
            // Clear reserved fields
            for i in 0..4 {
                core::ptr::write_volatile(&mut (*cmd_header)._reserved[i] as *mut u32, 0);
            }
            
            // Set command list base address in port registers
            let cmd_list_phys = virt_to_phys(cmd_list);
            core::ptr::write_volatile(&mut (*port).command_list_base as *mut u32, (cmd_list_phys & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*port).command_list_base_upper as *mut u32, ((cmd_list_phys >> 32) & 0xFFFFFFFF) as u32);
            
            // Allocate and set up received FIS buffer (256 bytes, 256-byte aligned)
            let fis_buffer = allocate_aligned(256, 256).ok_or(BlockDeviceError::IoError)?;
            let fis_buffer_phys = virt_to_phys(fis_buffer);
            core::ptr::write_volatile(&mut (*port).fis_base as *mut u32, (fis_buffer_phys & 0xFFFFFFFF) as u32);
            core::ptr::write_volatile(&mut (*port).fis_base_upper as *mut u32, ((fis_buffer_phys >> 32) & 0xFFFFFFFF) as u32);
            
            // TODO: Track allocated memory (cmd_list, cmd_table_ptr, dma_buffer, fis_buffer)
            // for cleanup after command completion to prevent memory leaks
            
            Ok(())
        }
    }
    
    /// Wait for command completion (interrupt-driven)
    fn wait_for_completion(&self, port: *mut PortRegisters) -> Result<(), BlockDeviceError> {
        // Create I/O completion tracker
        let completion = add_pending_io(self.port, 0);
        
        // Enable port interrupts
        enable_port_interrupts(self.hba as *mut u8, self.port);
        
        // Wait for completion with timeout (5 seconds = 5000ms)
        match wait_for_completion(&completion, 5000) {
            Ok(_) => Ok(()),
            Err(_) => Err(BlockDeviceError::Timeout),
        }
    }
}

impl BlockDevice for AhciDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn block_size(&self) -> usize {
        self.block_size
    }

    fn num_blocks(&self) -> u64 {
        self.num_blocks
    }

    fn read_blocks(&self, block_offset: u64, buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
        if block_offset >= self.num_blocks {
            return Err(BlockDeviceError::InvalidOffset);
        }

        let blocks_to_read = (buffer.len() / self.block_size).min((self.num_blocks - block_offset) as usize);
        if blocks_to_read == 0 {
            return Ok(0);
        }

        self.read_dma(block_offset, blocks_to_read as u16, buffer)?;
        Ok(blocks_to_read)
    }

    fn write_blocks(&self, block_offset: u64, buffer: &[u8]) -> Result<usize, BlockDeviceError> {
        if block_offset >= self.num_blocks {
            return Err(BlockDeviceError::InvalidOffset);
        }

        let blocks_to_write = (buffer.len() / self.block_size).min((self.num_blocks - block_offset) as usize);
        if blocks_to_write == 0 {
            return Ok(0);
        }

        self.write_dma(block_offset, blocks_to_write as u16, buffer)?;
        Ok(blocks_to_write)
    }

    fn flush(&self) -> Result<(), BlockDeviceError> {
        // Issue FLUSH CACHE command
        Ok(())
    }

    fn model(&self) -> Option<&str> {
        Some("AHCI SATA Device")
    }
}

/// AHCI Controller
pub struct AhciController {
    hba: *mut HbaRegisters,
    devices: Vec<Arc<AhciDevice>>,
}

impl AhciController {
    /// Create a new AHCI controller
    ///
    /// # Safety
    ///
    /// The caller must ensure that `hba_base` points to valid AHCI MMIO registers
    pub unsafe fn new(hba_base: usize) -> Self {
        AhciController {
            hba: hba_base as *mut HbaRegisters,
            devices: Vec::new(),
        }
    }

    /// Initialize the controller
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Reset HBA
        // Enable AHCI mode
        // Detect ports with devices attached
        // Initialize each port
        
        // For now, this is a stub
        Ok(())
    }

    /// Probe for devices on all ports
    pub fn probe_devices(&mut self) {
        // Check each port (typically 0-31)
        // For each port that has a device:
        //   1. Initialize the port
        //   2. Identify the device
        //   3. Create an AhciDevice and register it
        
        // Stub: assume port 0 has a device
        let device = AhciDevice::new(
            String::from("sda"),
            0,
            self.hba,
        );
        self.devices.push(Arc::new(device));
    }
}

static AHCI_CONTROLLERS: Mutex<Vec<AhciController>> = Mutex::new(Vec::new());

/// Initialize AHCI driver
pub fn init() {
    // Initialize interrupt-driven I/O
    crate::ahci_irq::init();
    
    // Scan PCI for AHCI controllers
    let controllers = scan_pci_for_ahci();
    
    if controllers.is_empty() {
        // No AHCI controllers found
        return;
    }
    
    // Initialize each controller
    let mut ctrl_list = AHCI_CONTROLLERS.lock();
    for hba_base in controllers {
        unsafe {
            let mut controller = AhciController::new(hba_base);
            if controller.init().is_ok() {
                controller.probe_devices();
                ctrl_list.push(controller);
            }
        }
    }
}

/// Scan PCI bus for AHCI controllers
fn scan_pci_for_ahci() -> Vec<usize> {
    let mut controllers = Vec::new();
    let mut empty_buses = 0;
    const MAX_EMPTY_BUSES: u16 = 8; // Stop after 8 consecutive empty buses
    
    // Scan all PCI buses, devices, and functions
    for bus in 0..MAX_PCI_BUS {
        let mut bus_has_devices = false;
        
        for device in 0..32u8 {
            for function in 0..8u8 {
                // Read vendor ID
                let vendor_device = read_pci_config_u16(bus as u8, device, function, 0);
                let vendor_id = vendor_device & 0xFFFF;
                
                // Skip if no device present (vendor ID 0xFFFF)
                if vendor_id == 0xFFFF {
                    continue;
                }
                
                bus_has_devices = true;
                
                // Read class/subclass
                let class_reg = read_pci_config_u16(bus as u8, device, function, 0x0A);
                let subclass = (class_reg >> 8) as u8;
                let class = (class_reg & 0xFF) as u8;
                
                // Check for SATA controller (class 0x01, subclass 0x06)
                if class == AHCI_PCI_CLASS && subclass == AHCI_PCI_SUBCLASS {
                    // Read programming interface
                    let prog_if = read_pci_config_u8(bus as u8, device, function, 0x09);
                    
                    // Check for AHCI (programming interface 0x01)
                    if prog_if == 0x01 {
                        // Read BAR5 (AHCI Base Address Register)
                        let bar5 = read_pci_config_u32(bus as u8, device, function, 0x24);
                        if bar5 != 0 && (bar5 & PCI_BAR_MEMORY_SPACE) == 0 {
                            // Valid memory BAR
                            let hba_base = (bar5 & !0xFFF) as usize;
                            controllers.push(hba_base);
                        }
                    }
                }
            }
        }
        
        // Early termination: stop if we've seen many consecutive empty buses
        if !bus_has_devices {
            empty_buses += 1;
            if empty_buses >= MAX_EMPTY_BUSES {
                break;
            }
        } else {
            empty_buses = 0;
        }
    }
    
    controllers
}

/// Read PCI configuration word (16-bit)
fn read_pci_config_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = read_pci_config_u32(bus, device, function, offset & 0xFC);
    let shift = ((offset & 0x2) * 8) as u32;
    ((value >> shift) & 0xFFFF) as u16
}

/// Read PCI configuration byte (8-bit)
fn read_pci_config_u8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let value = read_pci_config_u32(bus, device, function, offset & 0xFC);
    let shift = ((offset & 0x3) * 8) as u32;
    ((value >> shift) & 0xFF) as u8
}

/// Read PCI configuration dword (32-bit)
fn read_pci_config_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = 0x80000000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    
    unsafe {
        // Write address to CONFIG_ADDRESS port (0xCF8)
        core::arch::asm!(
            "out dx, eax",
            in("dx") 0xCF8u16,
            in("eax") address,
            options(nomem, nostack)
        );
        
        // Read data from CONFIG_DATA port (0xCFC)
        let mut data: u32;
        core::arch::asm!(
            "in eax, dx",
            out("eax") data,
            in("dx") 0xCFCu16,
            options(nomem, nostack)
        );
        data
    }
}

/// Write PCI configuration dword (32-bit)
fn write_pci_config_u32(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = 0x80000000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    
    unsafe {
        // Write address to CONFIG_ADDRESS port (0xCF8)
        core::arch::asm!(
            "out dx, eax",
            in("dx") 0xCF8u16,
            in("eax") address,
            options(nomem, nostack)
        );
        
        // Write data to CONFIG_DATA port (0xCFC)
        core::arch::asm!(
            "out dx, eax",
            in("dx") 0xCFCu16,
            in("eax") value,
            options(nomem, nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ahci_device_creation() {
        let hba = core::ptr::null_mut();
        let device = AhciDevice::new(String::from("sda"), 0, hba);
        assert_eq!(device.name(), "sda");
        assert_eq!(device.block_size(), 512);
    }
}
