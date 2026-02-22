//! Network Device Framework
//!
//! Network device abstraction layer for device drivers.

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

use super::ethernet::MacAddress;

/// Maximum Transmission Unit (default)
pub const DEFAULT_MTU: usize = 1500;

/// Network device capabilities
#[derive(Debug, Clone, Copy)]
pub struct DeviceCapabilities {
    /// Maximum transmission unit
    pub mtu: usize,
    /// Supports checksum offload
    pub checksum_offload: bool,
    /// Supports scatter-gather I/O
    pub scatter_gather: bool,
    /// Supports VLAN tagging
    pub vlan_support: bool,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            mtu: DEFAULT_MTU,
            checksum_offload: false,
            scatter_gather: false,
            vlan_support: false,
        }
    }
}

/// Link state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// Link is down
    Down,
    /// Link is up
    Up,
    /// Link state unknown
    Unknown,
}

/// Network device statistics
#[derive(Debug, Default)]
pub struct DeviceStats {
    /// Packets received
    pub rx_packets: AtomicU32,
    /// Bytes received
    pub rx_bytes: AtomicU32,
    /// Receive errors
    pub rx_errors: AtomicU32,
    /// Receive drops
    pub rx_dropped: AtomicU32,
    /// Packets transmitted
    pub tx_packets: AtomicU32,
    /// Bytes transmitted
    pub tx_bytes: AtomicU32,
    /// Transmit errors
    pub tx_errors: AtomicU32,
    /// Transmit drops
    pub tx_dropped: AtomicU32,
}

impl DeviceStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment receive packets
    pub fn inc_rx_packets(&self, count: u32) {
        self.rx_packets.fetch_add(count, Ordering::Relaxed);
    }

    /// Increment receive bytes
    pub fn inc_rx_bytes(&self, count: u32) {
        self.rx_bytes.fetch_add(count, Ordering::Relaxed);
    }

    /// Increment receive errors
    pub fn inc_rx_errors(&self) {
        self.rx_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment receive drops
    pub fn inc_rx_dropped(&self) {
        self.rx_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment transmit packets
    pub fn inc_tx_packets(&self, count: u32) {
        self.tx_packets.fetch_add(count, Ordering::Relaxed);
    }

    /// Increment transmit bytes
    pub fn inc_tx_bytes(&self, count: u32) {
        self.tx_bytes.fetch_add(count, Ordering::Relaxed);
    }

    /// Increment transmit errors
    pub fn inc_tx_errors(&self) {
        self.tx_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment transmit drops
    pub fn inc_tx_dropped(&self) {
        self.tx_dropped.fetch_add(1, Ordering::Relaxed);
    }
}

/// Network device error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetDevError {
    /// Device not found
    NotFound,
    /// Device is down
    DeviceDown,
    /// Invalid parameters
    InvalidParam,
    /// Buffer too small
    BufferTooSmall,
    /// Operation would block
    WouldBlock,
    /// No memory available
    NoMemory,
    /// Device busy
    Busy,
    /// Not supported
    NotSupported,
}

/// Network device trait
pub trait NetDevice: Send + Sync {
    /// Get device name
    fn name(&self) -> &str;

    /// Get MAC address
    fn mac_address(&self) -> MacAddress;

    /// Set MAC address
    fn set_mac_address(&mut self, mac: MacAddress) -> Result<(), NetDevError>;

    /// Get link state
    fn link_state(&self) -> LinkState;

    /// Get MTU
    fn mtu(&self) -> usize;

    /// Set MTU
    fn set_mtu(&mut self, mtu: usize) -> Result<(), NetDevError>;

    /// Get device capabilities
    fn capabilities(&self) -> DeviceCapabilities;

    /// Bring interface up
    fn up(&mut self) -> Result<(), NetDevError>;

    /// Bring interface down
    fn down(&mut self) -> Result<(), NetDevError>;

    /// Send packet
    ///
    /// # Safety
    ///
    /// The packet data must be valid and properly formatted.
    fn send(&mut self, packet: &[u8]) -> Result<(), NetDevError>;

    /// Receive packet
    ///
    /// Returns the number of bytes received and writes them to the buffer.
    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, NetDevError>;

    /// Get device statistics
    fn stats(&self) -> &DeviceStats;
}

/// Network device registry
pub struct DeviceRegistry {
    devices: Vec<Arc<Mutex<dyn NetDevice>>>,
}

impl DeviceRegistry {
    /// Create new registry
    const fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    /// Register a network device
    pub fn register(&mut self, device: Arc<Mutex<dyn NetDevice>>) -> Result<(), NetDevError> {
        // Check if device with same name already exists
        let name = {
            let dev = device.lock();
            String::from(dev.name())
        };

        if self.devices.iter().any(|d| d.lock().name() == name) {
            return Err(NetDevError::InvalidParam);
        }

        self.devices.push(device);
        Ok(())
    }

    /// Unregister a network device
    pub fn unregister(&mut self, name: &str) -> Result<(), NetDevError> {
        let pos = self
            .devices
            .iter()
            .position(|d| d.lock().name() == name)
            .ok_or(NetDevError::NotFound)?;

        self.devices.remove(pos);
        Ok(())
    }

    /// Get device by name
    pub fn get_device(&self, name: &str) -> Option<Arc<Mutex<dyn NetDevice>>> {
        self.devices
            .iter()
            .find(|d| d.lock().name() == name)
            .cloned()
    }

    /// List all devices
    pub fn list_devices(&self) -> Vec<String> {
        self.devices
            .iter()
            .map(|d| String::from(d.lock().name()))
            .collect()
    }

    /// Get device count
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

/// Global device registry
static DEVICE_REGISTRY: Mutex<DeviceRegistry> = Mutex::new(DeviceRegistry::new());

/// Register a network device
pub fn register_device(device: Arc<Mutex<dyn NetDevice>>) -> Result<(), NetDevError> {
    DEVICE_REGISTRY.lock().register(device)
}

/// Unregister a network device
pub fn unregister_device(name: &str) -> Result<(), NetDevError> {
    DEVICE_REGISTRY.lock().unregister(name)
}

/// Get device by name
pub fn get_device(name: &str) -> Option<Arc<Mutex<dyn NetDevice>>> {
    DEVICE_REGISTRY.lock().get_device(name)
}

/// List all registered devices
pub fn list_devices() -> Vec<String> {
    DEVICE_REGISTRY.lock().list_devices()
}

/// Get device count
pub fn device_count() -> usize {
    DEVICE_REGISTRY.lock().device_count()
}

/// Network device subsystem initialized flag
static NETDEV_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize network device subsystem
pub fn init() {
    if NETDEV_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    NETDEV_INITIALIZED.store(true, Ordering::Release);
}

/// Check if network device subsystem is initialized
pub fn is_initialized() -> bool {
    NETDEV_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_capabilities() {
        let caps = DeviceCapabilities::default();
        assert_eq!(caps.mtu, DEFAULT_MTU);
        assert!(!caps.checksum_offload);
    }

    #[test]
    fn test_device_stats() {
        let stats = DeviceStats::new();
        stats.inc_rx_packets(10);
        assert_eq!(stats.rx_packets.load(Ordering::Relaxed), 10);

        stats.inc_tx_bytes(1500);
        assert_eq!(stats.tx_bytes.load(Ordering::Relaxed), 1500);
    }

    #[test]
    fn test_link_state() {
        assert_eq!(LinkState::Down, LinkState::Down);
        assert_ne!(LinkState::Up, LinkState::Down);
    }
}
