//! Ethernet Layer
//!
//! Ethernet frame handling and MAC address management.

use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};

/// Ethernet MAC address (6 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Broadcast MAC address (FF:FF:FF:FF:FF:FF)
    pub const BROADCAST: Self = Self([0xFF; 6]);

    /// Zero MAC address (00:00:00:00:00:00)
    pub const ZERO: Self = Self([0x00; 6]);

    /// Create new MAC address
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Create from individual octets
    pub const fn from_octets(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        Self([a, b, c, d, e, f])
    }

    /// Get octets
    pub const fn octets(&self) -> [u8; 6] {
        self.0
    }

    /// Check if broadcast address
    pub const fn is_broadcast(&self) -> bool {
        self.0[0] == 0xFF
            && self.0[1] == 0xFF
            && self.0[2] == 0xFF
            && self.0[3] == 0xFF
            && self.0[4] == 0xFF
            && self.0[5] == 0xFF
    }

    /// Check if multicast address
    pub const fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }

    /// Check if unicast address
    pub const fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }

    /// Check if locally administered address
    pub const fn is_local(&self) -> bool {
        self.0[0] & 0x02 != 0
    }

    /// Check if globally unique address
    pub const fn is_global(&self) -> bool {
        !self.is_local()
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// EtherType values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EtherType {
    /// IPv4
    Ipv4 = 0x0800,
    /// ARP
    Arp = 0x0806,
    /// IPv6
    Ipv6 = 0x86DD,
    /// VLAN-tagged frame (802.1Q)
    Vlan = 0x8100,
    /// Unknown type
    Unknown = 0xFFFF,
}

impl EtherType {
    /// Create EtherType from u16
    pub const fn from_u16(value: u16) -> Self {
        match value {
            0x0800 => Self::Ipv4,
            0x0806 => Self::Arp,
            0x86DD => Self::Ipv6,
            0x8100 => Self::Vlan,
            _ => Self::Unknown,
        }
    }

    /// Convert to u16
    pub const fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Ethernet frame header (14 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EthernetHeader {
    /// Destination MAC address
    pub dst: MacAddress,
    /// Source MAC address
    pub src: MacAddress,
    /// EtherType (network byte order)
    pub ethertype: u16,
}

impl EthernetHeader {
    /// Header size in bytes
    pub const SIZE: usize = 14;

    /// Create new Ethernet header
    pub const fn new(dst: MacAddress, src: MacAddress, ethertype: EtherType) -> Self {
        Self {
            dst,
            src,
            ethertype: ethertype.as_u16().to_be(),
        }
    }

    /// Parse header from bytes
    pub fn parse(data: &[u8]) -> Result<Self, EthernetError> {
        if data.len() < Self::SIZE {
            return Err(EthernetError::TooShort);
        }

        let mut dst = [0u8; 6];
        let mut src = [0u8; 6];
        dst.copy_from_slice(&data[0..6]);
        src.copy_from_slice(&data[6..12]);

        let ethertype = u16::from_be_bytes([data[12], data[13]]);

        Ok(Self {
            dst: MacAddress(dst),
            src: MacAddress(src),
            ethertype,
        })
    }

    /// Get EtherType
    pub fn get_ethertype(&self) -> EtherType {
        EtherType::from_u16(u16::from_be(self.ethertype))
    }

    /// Write header to buffer
    pub fn write_to(&self, buffer: &mut [u8]) -> Result<(), EthernetError> {
        if buffer.len() < Self::SIZE {
            return Err(EthernetError::BufferTooSmall);
        }

        buffer[0..6].copy_from_slice(&self.dst.0);
        buffer[6..12].copy_from_slice(&self.src.0);
        buffer[12..14].copy_from_slice(&self.ethertype.to_ne_bytes());

        Ok(())
    }
}

/// Ethernet frame
pub struct EthernetFrame<'a> {
    /// Frame data
    data: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    /// Minimum frame size (without FCS)
    pub const MIN_SIZE: usize = 60;

    /// Maximum frame size (without FCS)
    pub const MAX_SIZE: usize = 1518;

    /// Create frame from data
    pub fn new(data: &'a [u8]) -> Result<Self, EthernetError> {
        if data.len() < EthernetHeader::SIZE {
            return Err(EthernetError::TooShort);
        }

        Ok(Self { data })
    }

    /// Get header
    pub fn header(&self) -> Result<EthernetHeader, EthernetError> {
        EthernetHeader::parse(self.data)
    }

    /// Get payload
    pub fn payload(&self) -> &[u8] {
        &self.data[EthernetHeader::SIZE..]
    }

    /// Get destination MAC
    pub fn dst_mac(&self) -> MacAddress {
        let mut mac = [0u8; 6];
        mac.copy_from_slice(&self.data[0..6]);
        MacAddress(mac)
    }

    /// Get source MAC
    pub fn src_mac(&self) -> MacAddress {
        let mut mac = [0u8; 6];
        mac.copy_from_slice(&self.data[6..12]);
        MacAddress(mac)
    }

    /// Get EtherType
    pub fn ethertype(&self) -> EtherType {
        let et = u16::from_be_bytes([self.data[12], self.data[13]]);
        EtherType::from_u16(et)
    }

    /// Get frame length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if frame is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Ethernet frame builder
pub struct EthernetFrameBuilder {
    dst: MacAddress,
    src: MacAddress,
    ethertype: EtherType,
}

impl EthernetFrameBuilder {
    /// Create new builder
    pub const fn new() -> Self {
        Self {
            dst: MacAddress::BROADCAST,
            src: MacAddress::ZERO,
            ethertype: EtherType::Ipv4,
        }
    }

    /// Set destination MAC
    pub fn dst(mut self, mac: MacAddress) -> Self {
        self.dst = mac;
        self
    }

    /// Set source MAC
    pub fn src(mut self, mac: MacAddress) -> Self {
        self.src = mac;
        self
    }

    /// Set EtherType
    pub fn ethertype(mut self, ethertype: EtherType) -> Self {
        self.ethertype = ethertype;
        self
    }

    /// Build frame into buffer with payload
    pub fn build(self, buffer: &mut [u8], payload: &[u8]) -> Result<usize, EthernetError> {
        let total_len = EthernetHeader::SIZE + payload.len();
        if buffer.len() < total_len {
            return Err(EthernetError::BufferTooSmall);
        }

        let header = EthernetHeader::new(self.dst, self.src, self.ethertype);
        header.write_to(buffer)?;
        buffer[EthernetHeader::SIZE..total_len].copy_from_slice(payload);

        Ok(total_len)
    }
}

impl Default for EthernetFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Ethernet errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EthernetError {
    /// Frame too short
    TooShort,
    /// Buffer too small
    BufferTooSmall,
    /// Invalid frame
    InvalidFrame,
    /// Unknown EtherType
    UnknownEtherType,
}

/// Ethernet subsystem initialized flag
static ETHERNET_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize Ethernet subsystem
pub fn init() {
    if ETHERNET_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    ETHERNET_INITIALIZED.store(true, Ordering::Release);
}

/// Check if Ethernet subsystem is initialized
pub fn is_initialized() -> bool {
    ETHERNET_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address() {
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(mac.octets(), [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(!mac.is_broadcast());
        assert!(mac.is_unicast());
    }

    #[test]
    fn test_mac_broadcast() {
        let mac = MacAddress::BROADCAST;
        assert!(mac.is_broadcast());
        assert!(mac.is_multicast());
    }

    #[test]
    fn test_ethertype() {
        let ipv4 = EtherType::from_u16(0x0800);
        assert_eq!(ipv4, EtherType::Ipv4);
        assert_eq!(ipv4.as_u16(), 0x0800);

        let arp = EtherType::Arp;
        assert_eq!(arp.as_u16(), 0x0806);
    }

    #[test]
    fn test_ethernet_header() {
        let dst = MacAddress::BROADCAST;
        let src = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let header = EthernetHeader::new(dst, src, EtherType::Ipv4);

        assert_eq!(header.dst, dst);
        assert_eq!(header.src, src);
        assert_eq!(header.get_ethertype(), EtherType::Ipv4);
    }

    #[test]
    fn test_frame_builder() {
        let mut buffer = [0u8; 64];
        let payload = [0xAA, 0xBB, 0xCC, 0xDD];

        let len = EthernetFrameBuilder::new()
            .dst(MacAddress::BROADCAST)
            .src(MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]))
            .ethertype(EtherType::Ipv4)
            .build(&mut buffer, &payload)
            .unwrap();

        assert_eq!(len, EthernetHeader::SIZE + payload.len());

        let frame = EthernetFrame::new(&buffer[..len]).unwrap();
        assert_eq!(frame.dst_mac(), MacAddress::BROADCAST);
        assert_eq!(frame.ethertype(), EtherType::Ipv4);
        assert_eq!(frame.payload(), &payload);
    }
}
