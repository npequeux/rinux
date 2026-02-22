//! Intel e1000/e1000e Network Driver
//!
//! This driver supports Intel 8254x (e1000) and 8257x (e1000e) network adapters.
//! It implements full TX/RX functionality with DMA descriptor rings.

#![allow(dead_code)] // Allow unused register constants for future use

use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use rinux_arch_x86::memory::{phys_to_virt, virt_to_phys};
use rinux_kernel::net::ethernet::MacAddress;
use rinux_kernel::net::netdev::{
    DeviceCapabilities, DeviceStats, LinkState, NetDevError, NetDevice,
};
use spin::Mutex;

use crate::pci::{PciAddress, PciDevice};

// ============================================================================
// Register Offsets
// ============================================================================

// Control Registers
const REG_CTRL: u32 = 0x0000; // Device Control
const REG_STATUS: u32 = 0x0008; // Device Status
const REG_EECD: u32 = 0x0010; // EEPROM Control
const REG_CTRL_EXT: u32 = 0x0018; // Extended Device Control

// Interrupt Registers
const REG_ICR: u32 = 0x00C0; // Interrupt Cause Read
const REG_ICS: u32 = 0x00C8; // Interrupt Cause Set
const REG_IMS: u32 = 0x00D0; // Interrupt Mask Set
const REG_IMC: u32 = 0x00D8; // Interrupt Mask Clear

// Receive Registers
const REG_RCTL: u32 = 0x0100; // Receive Control
const REG_RDBAL: u32 = 0x2800; // RX Descriptor Base Low
const REG_RDBAH: u32 = 0x2804; // RX Descriptor Base High
const REG_RDLEN: u32 = 0x2808; // RX Descriptor Length
const REG_RDH: u32 = 0x2810; // RX Descriptor Head
const REG_RDT: u32 = 0x2818; // RX Descriptor Tail
const REG_RDTR: u32 = 0x2820; // RX Delay Timer
const REG_RXDCTL: u32 = 0x2828; // RX Descriptor Control

// Transmit Registers
const REG_TCTL: u32 = 0x0400; // Transmit Control
const REG_TDBAL: u32 = 0x3800; // TX Descriptor Base Low
const REG_TDBAH: u32 = 0x3804; // TX Descriptor Base High
const REG_TDLEN: u32 = 0x3808; // TX Descriptor Length
const REG_TDH: u32 = 0x3810; // TX Descriptor Head
const REG_TDT: u32 = 0x3818; // TX Descriptor Tail
const REG_TIDV: u32 = 0x3820; // TX Interrupt Delay Value
const REG_TXDCTL: u32 = 0x3828; // TX Descriptor Control

// MAC Address Registers
const REG_RAL: u32 = 0x5400; // Receive Address Low
const REG_RAH: u32 = 0x5404; // Receive Address High

// MTA (Multicast Table Array)
const REG_MTA: u32 = 0x5200;

// ============================================================================
// Control Register Bits
// ============================================================================

// CTRL Register
const CTRL_FD: u32 = 1 << 0; // Full Duplex
const CTRL_LRST: u32 = 1 << 3; // Link Reset
const CTRL_ASDE: u32 = 1 << 5; // Auto-Speed Detection Enable
const CTRL_SLU: u32 = 1 << 6; // Set Link Up
const CTRL_RST: u32 = 1 << 26; // Device Reset
const CTRL_PHY_RST: u32 = 1 << 31; // PHY Reset

// STATUS Register
const STATUS_LU: u32 = 1 << 1; // Link Up

// RCTL Register
const RCTL_EN: u32 = 1 << 1; // Receive Enable
const RCTL_SBP: u32 = 1 << 2; // Store Bad Packets
const RCTL_UPE: u32 = 1 << 3; // Unicast Promiscuous Enable
const RCTL_MPE: u32 = 1 << 4; // Multicast Promiscuous Enable
const RCTL_LPE: u32 = 1 << 5; // Long Packet Enable
const RCTL_BAM: u32 = 1 << 15; // Broadcast Accept Mode
const RCTL_BSIZE_2048: u32 = 0 << 16; // Buffer Size 2048 bytes
const RCTL_BSEX: u32 = 1 << 25; // Buffer Size Extension
const RCTL_SECRC: u32 = 1 << 26; // Strip Ethernet CRC

// TCTL Register
const TCTL_EN: u32 = 1 << 1; // Transmit Enable
const TCTL_PSP: u32 = 1 << 3; // Pad Short Packets
const TCTL_CT_SHIFT: u32 = 4; // Collision Threshold
const TCTL_COLD_SHIFT: u32 = 12; // Collision Distance

// Interrupt Masks
const INT_TXDW: u32 = 1 << 0; // Transmit Descriptor Written Back
const INT_TXQE: u32 = 1 << 1; // Transmit Queue Empty
const INT_LSC: u32 = 1 << 2; // Link Status Change
const INT_RXSEQ: u32 = 1 << 3; // Receive Sequence Error
const INT_RXDMT0: u32 = 1 << 4; // Receive Descriptor Minimum Threshold
const INT_RXO: u32 = 1 << 6; // Receive Overrun
const INT_RXT0: u32 = 1 << 7; // Receiver Timer Interrupt

// RAH Register
const RAH_AV: u32 = 1 << 31; // Address Valid

// ============================================================================
// Descriptor Structures
// ============================================================================

/// Receive Descriptor
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
struct RxDesc {
    addr: u64,   // Physical address of buffer
    length: u16, // Length of data
    checksum: u16,
    status: u8, // Descriptor status
    errors: u8, // Descriptor errors
    special: u16,
}

impl RxDesc {
    const fn new() -> Self {
        Self {
            addr: 0,
            length: 0,
            checksum: 0,
            status: 0,
            errors: 0,
            special: 0,
        }
    }

    fn is_done(&self) -> bool {
        (self.status & 0x01) != 0
    }

    fn has_error(&self) -> bool {
        self.errors != 0
    }
}

/// Transmit Descriptor
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
struct TxDesc {
    addr: u64,   // Physical address of buffer
    length: u16, // Length of data
    cso: u8,     // Checksum Offset
    cmd: u8,     // Command byte
    status: u8,  // Descriptor status
    css: u8,     // Checksum Start
    special: u16,
}

impl TxDesc {
    const fn new() -> Self {
        Self {
            addr: 0,
            length: 0,
            cso: 0,
            cmd: 0,
            status: 0,
            css: 0,
            special: 0,
        }
    }

    fn is_done(&self) -> bool {
        (self.status & 0x01) != 0
    }
}

// TX Command bits
const TX_CMD_EOP: u8 = 0x01; // End of Packet
const TX_CMD_IFCS: u8 = 0x02; // Insert FCS
const TX_CMD_RS: u8 = 0x08; // Report Status

// ============================================================================
// Device IDs
// ============================================================================

const VENDOR_INTEL: u16 = 0x8086;

// Common e1000 device IDs
const DEVICE_82540EM: u16 = 0x100E; // 82540EM Gigabit Ethernet
const DEVICE_82545EM: u16 = 0x100F; // 82545EM Gigabit Ethernet
const DEVICE_82543GC: u16 = 0x1004; // 82543GC Gigabit Ethernet
const DEVICE_82544EI: u16 = 0x1008; // 82544EI Gigabit Ethernet
const DEVICE_82544GC: u16 = 0x100C; // 82544GC Gigabit Ethernet
const DEVICE_82547EI: u16 = 0x1019; // 82547EI Gigabit Ethernet
const DEVICE_82541EI: u16 = 0x1013; // 82541EI Gigabit Ethernet
const DEVICE_82541GI: u16 = 0x1076; // 82541GI Gigabit Ethernet
const DEVICE_82547GI: u16 = 0x1075; // 82547GI Gigabit Ethernet

// e1000e device IDs (82574 and newer)
const DEVICE_82574L: u16 = 0x10D3; // 82574L Gigabit Ethernet
const DEVICE_82571EB: u16 = 0x105E; // 82571EB Gigabit Ethernet
const DEVICE_82572EI: u16 = 0x107C; // 82572EI Gigabit Ethernet
const DEVICE_82573E: u16 = 0x108B; // 82573E Gigabit Ethernet
const DEVICE_82573L: u16 = 0x109A; // 82573L Gigabit Ethernet

// ============================================================================
// Ring Buffer Configuration
// ============================================================================

const RX_RING_SIZE: usize = 256;
const TX_RING_SIZE: usize = 256;
const RX_BUFFER_SIZE: usize = 2048;
const TX_BUFFER_SIZE: usize = 2048;

// ============================================================================
// Driver State
// ============================================================================

/// DMA buffer for packet data
struct DmaBuffer {
    virt_addr: u64,
    phys_addr: u64,
}

impl DmaBuffer {
    fn new(size: usize) -> Result<Self, NetDevError> {
        // Allocate physical frames for the buffer
        // TODO: Support multi-frame allocation for larger buffers
        // Currently only allocates a single frame regardless of size
        let _num_frames = size.div_ceil(4096);

        // Allocate first frame
        let frame = rinux_mm::frame::allocate_frame().ok_or(NetDevError::NoMemory)?;
        let phys_addr = frame.start_address();
        let virt_addr = phys_to_virt(phys_addr);

        // Initialize memory to zero
        unsafe {
            core::ptr::write_bytes(virt_addr as *mut u8, 0, size);
        }

        Ok(Self {
            virt_addr,
            phys_addr,
        })
    }

    fn as_ptr(&self) -> *mut u8 {
        self.virt_addr as *mut u8
    }

    fn phys_addr(&self) -> u64 {
        self.phys_addr
    }
}

/// E1000 Device State
struct E1000State {
    mmio_base: u64,
    mac_address: MacAddress,
    link_up: AtomicBool,
    device_up: AtomicBool,

    // RX Ring
    rx_descs: Vec<RxDesc>,
    rx_buffers: Vec<DmaBuffer>,
    rx_tail: AtomicU32,

    // TX Ring
    tx_descs: Vec<TxDesc>,
    tx_buffers: Vec<DmaBuffer>,
    tx_tail: AtomicU32,
    tx_head: AtomicU32,
}

impl E1000State {
    fn new(mmio_base: u64) -> Result<Self, NetDevError> {
        let mut rx_descs = Vec::new();
        let mut rx_buffers = Vec::new();
        let mut tx_descs = Vec::new();
        let mut tx_buffers = Vec::new();

        // Allocate RX descriptors and buffers
        for _ in 0..RX_RING_SIZE {
            rx_descs.push(RxDesc::new());
            rx_buffers.push(DmaBuffer::new(RX_BUFFER_SIZE)?);
        }

        // Allocate TX descriptors and buffers
        for _ in 0..TX_RING_SIZE {
            tx_descs.push(TxDesc::new());
            tx_buffers.push(DmaBuffer::new(TX_BUFFER_SIZE)?);
        }

        Ok(Self {
            mmio_base,
            mac_address: MacAddress::ZERO,
            link_up: AtomicBool::new(false),
            device_up: AtomicBool::new(false),
            rx_descs,
            rx_buffers,
            rx_tail: AtomicU32::new(0),
            tx_descs,
            tx_buffers,
            tx_tail: AtomicU32::new(0),
            tx_head: AtomicU32::new(0),
        })
    }

    // MMIO Read/Write
    fn read_reg(&self, offset: u32) -> u32 {
        unsafe { read_volatile((self.mmio_base + offset as u64) as *const u32) }
    }

    fn write_reg(&self, offset: u32, value: u32) {
        unsafe { write_volatile((self.mmio_base + offset as u64) as *mut u32, value) }
    }

    // Device Reset
    fn reset(&mut self) -> Result<(), NetDevError> {
        // Disable interrupts
        self.write_reg(REG_IMC, 0xFFFFFFFF);

        // Issue global reset
        self.write_reg(REG_CTRL, self.read_reg(REG_CTRL) | CTRL_RST);

        // Wait for reset to complete (spec says 1ms minimum)
        self.delay_ms(10);

        // Disable interrupts again after reset
        self.write_reg(REG_IMC, 0xFFFFFFFF);

        // Clear interrupt causes
        self.read_reg(REG_ICR);

        Ok(())
    }

    // Read MAC address from EEPROM/registers
    fn read_mac_address(&mut self) {
        let low = self.read_reg(REG_RAL);
        let high = self.read_reg(REG_RAH);

        let mut mac = [0u8; 6];
        mac[0] = (low & 0xFF) as u8;
        mac[1] = ((low >> 8) & 0xFF) as u8;
        mac[2] = ((low >> 16) & 0xFF) as u8;
        mac[3] = ((low >> 24) & 0xFF) as u8;
        mac[4] = (high & 0xFF) as u8;
        mac[5] = ((high >> 8) & 0xFF) as u8;

        // If MAC is all zeros, generate a local MAC address
        if mac.iter().all(|&b| b == 0) {
            mac = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56]; // QEMU default
        }

        self.mac_address = MacAddress::new(mac);
    }

    // Initialize RX ring
    fn init_rx(&mut self) -> Result<(), NetDevError> {
        // Setup RX descriptors
        for i in 0..RX_RING_SIZE {
            self.rx_descs[i].addr = self.rx_buffers[i].phys_addr();
            self.rx_descs[i].status = 0;
        }

        // Get physical address of descriptor ring
        let rx_desc_phys = {
            let rx_desc_virt = self.rx_descs.as_ptr() as u64;
            virt_to_phys(rx_desc_virt)
        };

        // Program descriptor base address
        self.write_reg(REG_RDBAL, (rx_desc_phys & 0xFFFFFFFF) as u32);
        self.write_reg(REG_RDBAH, (rx_desc_phys >> 32) as u32);

        // Set descriptor ring length
        self.write_reg(
            REG_RDLEN,
            (RX_RING_SIZE * core::mem::size_of::<RxDesc>()) as u32,
        );

        // Initialize head and tail
        self.write_reg(REG_RDH, 0);
        self.write_reg(REG_RDT, (RX_RING_SIZE - 1) as u32);
        self.rx_tail
            .store((RX_RING_SIZE - 1) as u32, Ordering::Release);

        // Enable receiver
        let mut rctl = RCTL_EN | RCTL_BAM | RCTL_BSIZE_2048 | RCTL_SECRC;
        rctl |= RCTL_UPE; // Unicast promiscuous for now
        self.write_reg(REG_RCTL, rctl);

        Ok(())
    }

    // Initialize TX ring
    fn init_tx(&mut self) -> Result<(), NetDevError> {
        // Setup TX descriptors
        for i in 0..TX_RING_SIZE {
            self.tx_descs[i].addr = self.tx_buffers[i].phys_addr();
            self.tx_descs[i].cmd = 0;
            self.tx_descs[i].status = 1; // Mark as done initially
        }

        // Get physical address of descriptor ring
        let tx_desc_phys = {
            let tx_desc_virt = self.tx_descs.as_ptr() as u64;
            virt_to_phys(tx_desc_virt)
        };

        // Program descriptor base address
        self.write_reg(REG_TDBAL, (tx_desc_phys & 0xFFFFFFFF) as u32);
        self.write_reg(REG_TDBAH, (tx_desc_phys >> 32) as u32);

        // Set descriptor ring length
        self.write_reg(
            REG_TDLEN,
            (TX_RING_SIZE * core::mem::size_of::<TxDesc>()) as u32,
        );

        // Initialize head and tail
        self.write_reg(REG_TDH, 0);
        self.write_reg(REG_TDT, 0);
        self.tx_head.store(0, Ordering::Release);
        self.tx_tail.store(0, Ordering::Release);

        // Enable transmitter
        let mut tctl = TCTL_EN | TCTL_PSP;
        tctl |= (0x0F << TCTL_CT_SHIFT) | (0x40 << TCTL_COLD_SHIFT);
        self.write_reg(REG_TCTL, tctl);

        Ok(())
    }

    // Check and update link status
    fn update_link_status(&self) {
        let status = self.read_reg(REG_STATUS);
        let link_up = (status & STATUS_LU) != 0;
        self.link_up.store(link_up, Ordering::Release);
    }

    // Simple delay (busy wait)
    fn delay_ms(&self, ms: u32) {
        for _ in 0..(ms * 10000) {
            core::hint::spin_loop();
        }
    }

    // Transmit packet
    fn transmit(&mut self, packet: &[u8], stats: &DeviceStats) -> Result<(), NetDevError> {
        if packet.len() > TX_BUFFER_SIZE {
            stats.inc_tx_errors();
            return Err(NetDevError::BufferTooSmall);
        }

        let tail = self.tx_tail.load(Ordering::Acquire) as usize;
        let head = self.read_reg(REG_TDH) as usize;

        // Check if ring is full
        let next_tail = (tail + 1) % TX_RING_SIZE;
        if next_tail == head {
            stats.inc_tx_dropped();
            return Err(NetDevError::Busy);
        }

        // Copy packet to buffer
        unsafe {
            core::ptr::copy_nonoverlapping(
                packet.as_ptr(),
                self.tx_buffers[tail].as_ptr(),
                packet.len(),
            );
        }

        // Setup descriptor
        self.tx_descs[tail].length = packet.len() as u16;
        self.tx_descs[tail].cmd = TX_CMD_EOP | TX_CMD_IFCS | TX_CMD_RS;
        self.tx_descs[tail].status = 0;

        // Update tail pointer
        self.tx_tail.store(next_tail as u32, Ordering::Release);
        self.write_reg(REG_TDT, next_tail as u32);

        // Update statistics
        stats.inc_tx_packets(1);
        stats.inc_tx_bytes(packet.len() as u32);

        Ok(())
    }

    // Receive packet
    fn receive(&mut self, buffer: &mut [u8], stats: &DeviceStats) -> Result<usize, NetDevError> {
        let tail = self.rx_tail.load(Ordering::Acquire) as usize;
        let next_tail = (tail + 1) % RX_RING_SIZE;

        // Check if packet is available
        if !self.rx_descs[next_tail].is_done() {
            return Err(NetDevError::WouldBlock);
        }

        let desc = &self.rx_descs[next_tail];

        // Check for errors
        if desc.has_error() {
            stats.inc_rx_errors();
            // Clear descriptor and move to next
            self.rx_descs[next_tail].status = 0;
            self.rx_tail.store(next_tail as u32, Ordering::Release);
            self.write_reg(REG_RDT, next_tail as u32);
            return Err(NetDevError::InvalidParam);
        }

        let length = desc.length as usize;

        if length > buffer.len() {
            stats.inc_rx_dropped();
            return Err(NetDevError::BufferTooSmall);
        }

        // Copy packet data to buffer
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.rx_buffers[next_tail].as_ptr(),
                buffer.as_mut_ptr(),
                length,
            );
        }

        // Clear descriptor and give buffer back to hardware
        self.rx_descs[next_tail].status = 0;
        self.rx_tail.store(next_tail as u32, Ordering::Release);
        self.write_reg(REG_RDT, next_tail as u32);

        // Update statistics
        stats.inc_rx_packets(1);
        stats.inc_rx_bytes(length as u32);

        Ok(length)
    }
}

// ============================================================================
// E1000 Device Driver
// ============================================================================

/// Intel E1000 Network Driver
pub struct E1000Driver {
    name: String,
    state: Mutex<E1000State>,
    stats: DeviceStats,
}

impl E1000Driver {
    /// Create a new E1000 driver instance
    pub fn new(pci_device: PciDevice) -> Result<Self, NetDevError> {
        // Get MMIO base address from BAR0
        let bar0 = pci_device.bars[0];
        if bar0 == 0 || (bar0 & 0x1) != 0 {
            return Err(NetDevError::InvalidParam);
        }

        let mmio_base_phys = (bar0 & !0xF) as u64;
        let mmio_base = phys_to_virt(mmio_base_phys);

        // Enable bus mastering and memory space
        pci_device.enable_bus_mastering();
        pci_device.enable_memory_space();

        // Create device state
        let mut state = E1000State::new(mmio_base)?;

        // Reset device
        state.reset()?;

        // Read MAC address
        state.read_mac_address();

        // Initialize RX and TX rings
        state.init_rx()?;
        state.init_tx()?;

        // Set link up
        let ctrl = state.read_reg(REG_CTRL);
        state.write_reg(REG_CTRL, ctrl | CTRL_SLU | CTRL_ASDE);

        // Enable interrupts
        state.write_reg(
            REG_IMS,
            INT_RXT0 | INT_TXDW | INT_LSC | INT_RXDMT0 | INT_RXO,
        );

        // Update link status
        state.update_link_status();

        // Create device name
        let name = String::from("eth0");

        Ok(Self {
            name,
            state: Mutex::new(state),
            stats: DeviceStats::new(),
        })
    }

    /// Handle device interrupt
    pub fn handle_interrupt(&self) {
        let state = self.state.lock();

        // Read interrupt cause
        let icr = state.read_reg(REG_ICR);

        if icr == 0 {
            return; // Not our interrupt
        }

        // Handle link status change
        if (icr & INT_LSC) != 0 {
            state.update_link_status();
        }

        // RX and TX interrupts are handled by polling for now
    }
}

impl NetDevice for E1000Driver {
    fn name(&self) -> &str {
        &self.name
    }

    fn mac_address(&self) -> MacAddress {
        let state = self.state.lock();
        state.mac_address
    }

    fn set_mac_address(&mut self, mac: MacAddress) -> Result<(), NetDevError> {
        let mut state = self.state.lock();

        let octets = mac.octets();
        let low = u32::from(octets[0])
            | (u32::from(octets[1]) << 8)
            | (u32::from(octets[2]) << 16)
            | (u32::from(octets[3]) << 24);
        let high = u32::from(octets[4]) | (u32::from(octets[5]) << 8) | RAH_AV;

        state.write_reg(REG_RAL, low);
        state.write_reg(REG_RAH, high);
        state.mac_address = mac;

        Ok(())
    }

    fn link_state(&self) -> LinkState {
        let state = self.state.lock();
        if state.link_up.load(Ordering::Acquire) {
            LinkState::Up
        } else {
            LinkState::Down
        }
    }

    fn mtu(&self) -> usize {
        1500
    }

    fn set_mtu(&mut self, _mtu: usize) -> Result<(), NetDevError> {
        Err(NetDevError::NotSupported)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            mtu: 1500,
            checksum_offload: false,
            scatter_gather: false,
            vlan_support: false,
        }
    }

    fn up(&mut self) -> Result<(), NetDevError> {
        let state = self.state.lock();
        state.device_up.store(true, Ordering::Release);
        Ok(())
    }

    fn down(&mut self) -> Result<(), NetDevError> {
        let state = self.state.lock();
        state.device_up.store(false, Ordering::Release);
        Ok(())
    }

    fn send(&mut self, packet: &[u8]) -> Result<(), NetDevError> {
        let mut state = self.state.lock();

        if !state.device_up.load(Ordering::Acquire) {
            return Err(NetDevError::DeviceDown);
        }

        state.transmit(packet, &self.stats)
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, NetDevError> {
        let mut state = self.state.lock();

        if !state.device_up.load(Ordering::Acquire) {
            return Err(NetDevError::DeviceDown);
        }

        state.receive(buffer, &self.stats)
    }

    fn stats(&self) -> &DeviceStats {
        &self.stats
    }
}

// ============================================================================
// Device Detection and Initialization
// ============================================================================

/// Check if a PCI device is a supported e1000 device
pub fn is_e1000_device(device: &PciDevice) -> bool {
    if device.vendor_id != VENDOR_INTEL {
        return false;
    }

    matches!(
        device.device_id,
        DEVICE_82540EM
            | DEVICE_82545EM
            | DEVICE_82543GC
            | DEVICE_82544EI
            | DEVICE_82544GC
            | DEVICE_82547EI
            | DEVICE_82541EI
            | DEVICE_82541GI
            | DEVICE_82547GI
            | DEVICE_82574L
            | DEVICE_82571EB
            | DEVICE_82572EI
            | DEVICE_82573E
            | DEVICE_82573L
    )
}

/// Get device name for a given device ID
pub fn device_name(device_id: u16) -> &'static str {
    match device_id {
        DEVICE_82540EM => "Intel 82540EM Gigabit Ethernet",
        DEVICE_82545EM => "Intel 82545EM Gigabit Ethernet",
        DEVICE_82543GC => "Intel 82543GC Gigabit Ethernet",
        DEVICE_82544EI => "Intel 82544EI Gigabit Ethernet",
        DEVICE_82544GC => "Intel 82544GC Gigabit Ethernet",
        DEVICE_82547EI => "Intel 82547EI Gigabit Ethernet",
        DEVICE_82541EI => "Intel 82541EI Gigabit Ethernet",
        DEVICE_82541GI => "Intel 82541GI Gigabit Ethernet",
        DEVICE_82547GI => "Intel 82547GI Gigabit Ethernet",
        DEVICE_82574L => "Intel 82574L Gigabit Ethernet",
        DEVICE_82571EB => "Intel 82571EB Gigabit Ethernet",
        DEVICE_82572EI => "Intel 82572EI Gigabit Ethernet",
        DEVICE_82573E => "Intel 82573E Gigabit Ethernet",
        DEVICE_82573L => "Intel 82573L Gigabit Ethernet",
        _ => "Unknown Intel e1000 Device",
    }
}

/// Initialize e1000 driver and detect devices
pub fn init() {
    rinux_kernel::printk::printk("Initializing Intel e1000 network driver...\n");

    let scanner = crate::pci::scanner();
    let mut device_count = 0;

    for i in 0..scanner.device_count() {
        if let Some(device) = scanner.get_device(i) {
            if is_e1000_device(device) {
                rinux_kernel::printk::printk("Found ");
                rinux_kernel::printk::printk(device_name(device.device_id));
                rinux_kernel::printk::printk(" at ");

                // Print PCI address
                let addr = device.address;
                print_pci_address(addr);
                rinux_kernel::printk::printk("\n");

                // Try to initialize the driver
                match E1000Driver::new(*device) {
                    Ok(driver) => {
                        rinux_kernel::printk::printk("  Initialized with MAC: ");
                        print_mac_address(driver.mac_address());
                        rinux_kernel::printk::printk("\n");

                        // Register with network stack
                        let driver = alloc::sync::Arc::new(Mutex::new(driver));
                        if let Err(_e) = rinux_kernel::net::netdev::register_device(driver) {
                            rinux_kernel::printk::printk("  Failed to register device\n");
                        } else {
                            device_count += 1;
                        }
                    }
                    Err(_) => {
                        rinux_kernel::printk::printk("  Failed to initialize driver\n");
                    }
                }
            }
        }
    }

    if device_count == 0 {
        rinux_kernel::printk::printk("No e1000 devices found\n");
    }
}

// Helper function to print PCI address
fn print_pci_address(addr: PciAddress) {
    print_hex_u8(addr.bus);
    rinux_kernel::printk::printk(":");
    print_hex_u8(addr.device);
    rinux_kernel::printk::printk(".");
    print_hex_u8(addr.function);
}

// Helper function to print MAC address
fn print_mac_address(mac: MacAddress) {
    let octets = mac.octets();
    for (i, octet) in octets.iter().enumerate() {
        if i > 0 {
            rinux_kernel::printk::printk(":");
        }
        print_hex_u8(*octet);
    }
}

// Helper function to print hex byte
fn print_hex_u8(n: u8) {
    const HEX: &[u8] = b"0123456789abcdef";
    let chars = [
        HEX[(n >> 4) as usize] as char,
        HEX[(n & 0xF) as usize] as char,
    ];
    for ch in chars {
        rinux_kernel::printk::printk(match ch {
            '0' => "0",
            '1' => "1",
            '2' => "2",
            '3' => "3",
            '4' => "4",
            '5' => "5",
            '6' => "6",
            '7' => "7",
            '8' => "8",
            '9' => "9",
            'a' => "a",
            'b' => "b",
            'c' => "c",
            'd' => "d",
            'e' => "e",
            'f' => "f",
            _ => "?",
        });
    }
}
