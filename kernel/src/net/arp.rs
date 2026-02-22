//! Address Resolution Protocol (ARP)
//!
//! Maps IP addresses to MAC addresses on local networks.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use super::ethernet::{EtherType, EthernetFrameBuilder, MacAddress};
use super::ipv4::Ipv4Addr;

/// ARP hardware types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ArpHardwareType {
    /// Ethernet
    Ethernet = 1,
    /// Unknown
    Unknown = 0xFFFF,
}

impl ArpHardwareType {
    /// Create from u16
    pub const fn from_u16(value: u16) -> Self {
        match value {
            1 => Self::Ethernet,
            _ => Self::Unknown,
        }
    }

    /// Convert to u16
    pub const fn as_u16(self) -> u16 {
        self as u16
    }
}

/// ARP operation codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ArpOperation {
    /// ARP request
    Request = 1,
    /// ARP reply
    Reply = 2,
    /// Unknown
    Unknown = 0xFFFF,
}

impl ArpOperation {
    /// Create from u16
    pub const fn from_u16(value: u16) -> Self {
        match value {
            1 => Self::Request,
            2 => Self::Reply,
            _ => Self::Unknown,
        }
    }

    /// Convert to u16
    pub const fn as_u16(self) -> u16 {
        self as u16
    }
}

/// ARP packet header
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ArpHeader {
    /// Hardware type
    pub hardware_type: u16,
    /// Protocol type
    pub protocol_type: u16,
    /// Hardware address length
    pub hardware_len: u8,
    /// Protocol address length
    pub protocol_len: u8,
    /// Operation
    pub operation: u16,
    /// Sender hardware address (MAC)
    pub sender_hw_addr: [u8; 6],
    /// Sender protocol address (IP)
    pub sender_proto_addr: [u8; 4],
    /// Target hardware address (MAC)
    pub target_hw_addr: [u8; 6],
    /// Target protocol address (IP)
    pub target_proto_addr: [u8; 4],
}

impl ArpHeader {
    /// ARP header size
    pub const SIZE: usize = 28;

    /// Create new ARP header
    pub const fn new(
        operation: ArpOperation,
        sender_mac: MacAddress,
        sender_ip: Ipv4Addr,
        target_mac: MacAddress,
        target_ip: Ipv4Addr,
    ) -> Self {
        Self {
            hardware_type: ArpHardwareType::Ethernet.as_u16().to_be(),
            protocol_type: EtherType::Ipv4.as_u16().to_be(),
            hardware_len: 6,
            protocol_len: 4,
            operation: operation.as_u16().to_be(),
            sender_hw_addr: sender_mac.0,
            sender_proto_addr: sender_ip.0,
            target_hw_addr: target_mac.0,
            target_proto_addr: target_ip.0,
        }
    }

    /// Parse ARP header from bytes
    pub fn parse(data: &[u8]) -> Result<Self, ArpError> {
        if data.len() < Self::SIZE {
            return Err(ArpError::TooShort);
        }

        let hardware_type = u16::from_be_bytes([data[0], data[1]]);
        let protocol_type = u16::from_be_bytes([data[2], data[3]]);
        let hardware_len = data[4];
        let protocol_len = data[5];
        let operation = u16::from_be_bytes([data[6], data[7]]);

        let mut sender_hw_addr = [0u8; 6];
        let mut sender_proto_addr = [0u8; 4];
        let mut target_hw_addr = [0u8; 6];
        let mut target_proto_addr = [0u8; 4];

        sender_hw_addr.copy_from_slice(&data[8..14]);
        sender_proto_addr.copy_from_slice(&data[14..18]);
        target_hw_addr.copy_from_slice(&data[18..24]);
        target_proto_addr.copy_from_slice(&data[24..28]);

        Ok(Self {
            hardware_type: hardware_type.to_be(),
            protocol_type: protocol_type.to_be(),
            hardware_len,
            protocol_len,
            operation: operation.to_be(),
            sender_hw_addr,
            sender_proto_addr,
            target_hw_addr,
            target_proto_addr,
        })
    }

    /// Write header to buffer
    pub fn write_to(&self, buffer: &mut [u8]) -> Result<(), ArpError> {
        if buffer.len() < Self::SIZE {
            return Err(ArpError::BufferTooSmall);
        }

        let hw_type = u16::from_be(self.hardware_type);
        let proto_type = u16::from_be(self.protocol_type);
        let op = u16::from_be(self.operation);

        buffer[0..2].copy_from_slice(&hw_type.to_be_bytes());
        buffer[2..4].copy_from_slice(&proto_type.to_be_bytes());
        buffer[4] = self.hardware_len;
        buffer[5] = self.protocol_len;
        buffer[6..8].copy_from_slice(&op.to_be_bytes());
        buffer[8..14].copy_from_slice(&self.sender_hw_addr);
        buffer[14..18].copy_from_slice(&self.sender_proto_addr);
        buffer[18..24].copy_from_slice(&self.target_hw_addr);
        buffer[24..28].copy_from_slice(&self.target_proto_addr);

        Ok(())
    }

    /// Get operation
    pub fn get_operation(&self) -> ArpOperation {
        ArpOperation::from_u16(u16::from_be(self.operation))
    }

    /// Get sender MAC address
    pub fn sender_mac(&self) -> MacAddress {
        MacAddress(self.sender_hw_addr)
    }

    /// Get sender IP address
    pub fn sender_ip(&self) -> Ipv4Addr {
        Ipv4Addr(self.sender_proto_addr)
    }

    /// Get target MAC address
    pub fn target_mac(&self) -> MacAddress {
        MacAddress(self.target_hw_addr)
    }

    /// Get target IP address
    pub fn target_ip(&self) -> Ipv4Addr {
        Ipv4Addr(self.target_proto_addr)
    }
}

/// ARP cache entry
#[derive(Debug, Clone, Copy)]
pub struct ArpEntry {
    /// MAC address
    pub mac: MacAddress,
    /// IP address
    pub ip: Ipv4Addr,
    /// Time-to-live (in seconds)
    pub ttl: u32,
}

impl ArpEntry {
    /// Default TTL (5 minutes)
    pub const DEFAULT_TTL: u32 = 300;

    /// Create new ARP entry
    pub const fn new(mac: MacAddress, ip: Ipv4Addr) -> Self {
        Self {
            mac,
            ip,
            ttl: Self::DEFAULT_TTL,
        }
    }
}

/// ARP cache/table
pub struct ArpCache {
    /// IP to MAC mappings
    entries: BTreeMap<u32, ArpEntry>,
}

impl ArpCache {
    /// Create new ARP cache
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Insert entry
    pub fn insert(&mut self, ip: Ipv4Addr, mac: MacAddress) {
        let entry = ArpEntry::new(mac, ip);
        self.entries.insert(ip.as_u32(), entry);
    }

    /// Lookup MAC address by IP
    pub fn lookup(&self, ip: Ipv4Addr) -> Option<MacAddress> {
        self.entries.get(&ip.as_u32()).map(|entry| entry.mac)
    }

    /// Remove entry
    pub fn remove(&mut self, ip: Ipv4Addr) -> Option<ArpEntry> {
        self.entries.remove(&ip.as_u32())
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Update TTL for all entries (call periodically)
    pub fn update_ttl(&mut self, elapsed_secs: u32) {
        self.entries.retain(|_, entry| {
            if entry.ttl > elapsed_secs {
                entry.ttl -= elapsed_secs;
                true
            } else {
                false
            }
        });
    }
}

impl Default for ArpCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Global ARP cache
static ARP_CACHE: Mutex<ArpCache> = Mutex::new(ArpCache {
    entries: BTreeMap::new(),
});

/// Insert ARP entry into global cache
pub fn insert_entry(ip: Ipv4Addr, mac: MacAddress) {
    ARP_CACHE.lock().insert(ip, mac);
}

/// Lookup MAC address for IP in global cache
pub fn lookup_mac(ip: Ipv4Addr) -> Option<MacAddress> {
    ARP_CACHE.lock().lookup(ip)
}

/// Remove entry from global cache
pub fn remove_entry(ip: Ipv4Addr) -> Option<ArpEntry> {
    ARP_CACHE.lock().remove(ip)
}

/// Clear global ARP cache
pub fn clear_cache() {
    ARP_CACHE.lock().clear();
}

/// Build ARP request packet
pub fn build_request(
    buffer: &mut [u8],
    src_mac: MacAddress,
    src_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) -> Result<usize, ArpError> {
    // Build ARP header
    let arp_header = ArpHeader::new(
        ArpOperation::Request,
        src_mac,
        src_ip,
        MacAddress::ZERO,
        target_ip,
    );

    let mut arp_buf = [0u8; ArpHeader::SIZE];
    arp_header.write_to(&mut arp_buf)?;

    // Build Ethernet frame
    let len = EthernetFrameBuilder::new()
        .dst(MacAddress::BROADCAST)
        .src(src_mac)
        .ethertype(EtherType::Arp)
        .build(buffer, &arp_buf)
        .map_err(|_| ArpError::BufferTooSmall)?;

    Ok(len)
}

/// Build ARP reply packet
pub fn build_reply(
    buffer: &mut [u8],
    src_mac: MacAddress,
    src_ip: Ipv4Addr,
    target_mac: MacAddress,
    target_ip: Ipv4Addr,
) -> Result<usize, ArpError> {
    // Build ARP header
    let arp_header = ArpHeader::new(ArpOperation::Reply, src_mac, src_ip, target_mac, target_ip);

    let mut arp_buf = [0u8; ArpHeader::SIZE];
    arp_header.write_to(&mut arp_buf)?;

    // Build Ethernet frame
    let len = EthernetFrameBuilder::new()
        .dst(target_mac)
        .src(src_mac)
        .ethertype(EtherType::Arp)
        .build(buffer, &arp_buf)
        .map_err(|_| ArpError::BufferTooSmall)?;

    Ok(len)
}

/// Process incoming ARP packet
pub fn process_packet(
    data: &[u8],
    our_mac: MacAddress,
    our_ip: Ipv4Addr,
) -> Result<Option<Vec<u8>>, ArpError> {
    // Skip ethernet header
    if data.len() < 14 + ArpHeader::SIZE {
        return Err(ArpError::TooShort);
    }

    let arp_data = &data[14..];
    let header = ArpHeader::parse(arp_data)?;

    // Add sender to ARP cache
    insert_entry(header.sender_ip(), header.sender_mac());

    match header.get_operation() {
        ArpOperation::Request => {
            // Is this request for us?
            if header.target_ip() == our_ip {
                // Build and send reply
                let mut reply_buf = [0u8; 64];
                let len = build_reply(
                    &mut reply_buf,
                    our_mac,
                    our_ip,
                    header.sender_mac(),
                    header.sender_ip(),
                )?;
                Ok(Some(reply_buf[..len].to_vec()))
            } else {
                Ok(None)
            }
        }
        ArpOperation::Reply => {
            // Already added to cache above
            Ok(None)
        }
        ArpOperation::Unknown => Err(ArpError::InvalidOperation),
    }
}

/// ARP errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpError {
    /// Packet too short
    TooShort,
    /// Buffer too small
    BufferTooSmall,
    /// Invalid operation
    InvalidOperation,
    /// Entry not found
    NotFound,
}

/// ARP subsystem initialized flag
static ARP_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize ARP subsystem
pub fn init() {
    if ARP_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    ARP_INITIALIZED.store(true, Ordering::Release);
}

/// Check if ARP subsystem is initialized
pub fn is_initialized() -> bool {
    ARP_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arp_operation() {
        assert_eq!(ArpOperation::from_u16(1), ArpOperation::Request);
        assert_eq!(ArpOperation::from_u16(2), ArpOperation::Reply);
        assert_eq!(ArpOperation::Request.as_u16(), 1);
    }

    #[test]
    fn test_arp_cache() {
        let mut cache = ArpCache::new();
        let ip = Ipv4Addr::new(192, 168, 1, 1);
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

        cache.insert(ip, mac);
        assert_eq!(cache.lookup(ip), Some(mac));
        assert_eq!(cache.len(), 1);

        cache.remove(ip);
        assert_eq!(cache.lookup(ip), None);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_arp_header() {
        let sender_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let sender_ip = Ipv4Addr::new(192, 168, 1, 1);
        let target_mac = MacAddress::ZERO;
        let target_ip = Ipv4Addr::new(192, 168, 1, 2);

        let header = ArpHeader::new(
            ArpOperation::Request,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        );

        assert_eq!(header.get_operation(), ArpOperation::Request);
        assert_eq!(header.sender_mac(), sender_mac);
        assert_eq!(header.sender_ip(), sender_ip);
        assert_eq!(header.target_ip(), target_ip);
    }
}
