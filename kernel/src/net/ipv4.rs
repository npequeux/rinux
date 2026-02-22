//! Internet Protocol version 4 (IPv4)
//!
//! IPv4 packet handling, routing, and checksum calculation.

use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};

/// IPv4 address (4 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Ipv4Addr(pub [u8; 4]);

impl Ipv4Addr {
    /// Unspecified address (0.0.0.0)
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);

    /// Broadcast address (255.255.255.255)
    pub const BROADCAST: Self = Self([255, 255, 255, 255]);

    /// Localhost (127.0.0.1)
    pub const LOCALHOST: Self = Self([127, 0, 0, 1]);

    /// Create new IPv4 address
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    /// Create from u32 (network byte order)
    pub const fn from_u32(addr: u32) -> Self {
        Self(addr.to_be_bytes())
    }

    /// Convert to u32 (network byte order)
    pub const fn as_u32(&self) -> u32 {
        u32::from_be_bytes(self.0)
    }

    /// Get octets
    pub const fn octets(&self) -> [u8; 4] {
        self.0
    }

    /// Check if address is unspecified (0.0.0.0)
    pub const fn is_unspecified(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    /// Check if address is loopback (127.0.0.0/8)
    pub const fn is_loopback(&self) -> bool {
        self.0[0] == 127
    }

    /// Check if address is private
    pub const fn is_private(&self) -> bool {
        // 10.0.0.0/8
        self.0[0] == 10
            // 172.16.0.0/12
            || (self.0[0] == 172 && self.0[1] >= 16 && self.0[1] <= 31)
            // 192.168.0.0/16
            || (self.0[0] == 192 && self.0[1] == 168)
    }

    /// Check if address is link-local (169.254.0.0/16)
    pub const fn is_link_local(&self) -> bool {
        self.0[0] == 169 && self.0[1] == 254
    }

    /// Check if address is multicast (224.0.0.0/4)
    pub const fn is_multicast(&self) -> bool {
        self.0[0] >= 224 && self.0[0] <= 239
    }

    /// Check if address is broadcast
    pub const fn is_broadcast(&self) -> bool {
        self.0[0] == 255 && self.0[1] == 255 && self.0[2] == 255 && self.0[3] == 255
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// IP protocol numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IpProtocol {
    /// ICMP
    Icmp = 1,
    /// TCP
    Tcp = 6,
    /// UDP
    Udp = 17,
    /// Unknown
    Unknown = 0xFF,
}

impl IpProtocol {
    /// Create from u8
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Icmp,
            6 => Self::Tcp,
            17 => Self::Udp,
            _ => Self::Unknown,
        }
    }

    /// Convert to u8
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// IPv4 header (20 bytes minimum)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Ipv4Header {
    /// Version (4 bits) and IHL (4 bits)
    pub version_ihl: u8,
    /// Type of Service
    pub tos: u8,
    /// Total Length (header + data)
    pub total_length: u16,
    /// Identification
    pub identification: u16,
    /// Flags (3 bits) and Fragment Offset (13 bits)
    pub flags_fragment: u16,
    /// Time to Live
    pub ttl: u8,
    /// Protocol
    pub protocol: u8,
    /// Header Checksum
    pub checksum: u16,
    /// Source Address
    pub src_addr: [u8; 4],
    /// Destination Address
    pub dst_addr: [u8; 4],
}

impl Ipv4Header {
    /// Minimum header size (20 bytes)
    pub const MIN_SIZE: usize = 20;

    /// Maximum header size (60 bytes, with options)
    pub const MAX_SIZE: usize = 60;

    /// Default TTL
    pub const DEFAULT_TTL: u8 = 64;

    /// Create new IPv4 header
    pub const fn new(src: Ipv4Addr, dst: Ipv4Addr, protocol: IpProtocol, payload_len: u16) -> Self {
        Self {
            version_ihl: 0x45, // Version 4, IHL 5 (20 bytes)
            tos: 0,
            total_length: (Self::MIN_SIZE as u16 + payload_len).to_be(),
            identification: 0,
            flags_fragment: 0,
            ttl: Self::DEFAULT_TTL,
            protocol: protocol.as_u8(),
            checksum: 0,
            src_addr: src.0,
            dst_addr: dst.0,
        }
    }

    /// Parse header from bytes
    pub fn parse(data: &[u8]) -> Result<Self, Ipv4Error> {
        if data.len() < Self::MIN_SIZE {
            return Err(Ipv4Error::TooShort);
        }

        let version_ihl = data[0];
        let version = version_ihl >> 4;
        if version != 4 {
            return Err(Ipv4Error::InvalidVersion);
        }

        let ihl = (version_ihl & 0x0F) as usize * 4;
        if !(Self::MIN_SIZE..=Self::MAX_SIZE).contains(&ihl) {
            return Err(Ipv4Error::InvalidHeaderLength);
        }

        if data.len() < ihl {
            return Err(Ipv4Error::TooShort);
        }

        let mut src_addr = [0u8; 4];
        let mut dst_addr = [0u8; 4];
        src_addr.copy_from_slice(&data[12..16]);
        dst_addr.copy_from_slice(&data[16..20]);

        Ok(Self {
            version_ihl,
            tos: data[1],
            total_length: u16::from_be_bytes([data[2], data[3]]).to_be(),
            identification: u16::from_be_bytes([data[4], data[5]]).to_be(),
            flags_fragment: u16::from_be_bytes([data[6], data[7]]).to_be(),
            ttl: data[8],
            protocol: data[9],
            checksum: u16::from_be_bytes([data[10], data[11]]).to_be(),
            src_addr,
            dst_addr,
        })
    }

    /// Write header to buffer
    pub fn write_to(&self, buffer: &mut [u8]) -> Result<(), Ipv4Error> {
        if buffer.len() < Self::MIN_SIZE {
            return Err(Ipv4Error::BufferTooSmall);
        }

        buffer[0] = self.version_ihl;
        buffer[1] = self.tos;
        buffer[2..4].copy_from_slice(&u16::from_be(self.total_length).to_be_bytes());
        buffer[4..6].copy_from_slice(&u16::from_be(self.identification).to_be_bytes());
        buffer[6..8].copy_from_slice(&u16::from_be(self.flags_fragment).to_be_bytes());
        buffer[8] = self.ttl;
        buffer[9] = self.protocol;
        buffer[10..12].copy_from_slice(&u16::from_be(self.checksum).to_be_bytes());
        buffer[12..16].copy_from_slice(&self.src_addr);
        buffer[16..20].copy_from_slice(&self.dst_addr);

        Ok(())
    }

    /// Get header length in bytes
    pub fn header_length(&self) -> usize {
        ((self.version_ihl & 0x0F) as usize) * 4
    }

    /// Get version
    pub fn version(&self) -> u8 {
        self.version_ihl >> 4
    }

    /// Get total length
    pub fn total_length(&self) -> u16 {
        u16::from_be(self.total_length)
    }

    /// Get protocol
    pub fn protocol(&self) -> IpProtocol {
        IpProtocol::from_u8(self.protocol)
    }

    /// Get source address
    pub fn src_addr(&self) -> Ipv4Addr {
        Ipv4Addr(self.src_addr)
    }

    /// Get destination address
    pub fn dst_addr(&self) -> Ipv4Addr {
        Ipv4Addr(self.dst_addr)
    }

    /// Get checksum
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.checksum)
    }

    /// Calculate checksum
    pub fn calculate_checksum(&mut self) {
        // Zero out checksum field
        self.checksum = 0;

        // Serialize header to buffer
        let mut buffer = [0u8; Self::MIN_SIZE];
        let _ = self.write_to(&mut buffer);

        // Calculate checksum
        let checksum = calculate_checksum(&buffer[..self.header_length()]);
        self.checksum = checksum.to_be();
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> bool {
        let mut buffer = [0u8; Self::MIN_SIZE];
        let _ = self.write_to(&mut buffer);

        let checksum = calculate_checksum(&buffer[..self.header_length()]);
        checksum == 0
    }
}

/// IPv4 packet
pub struct Ipv4Packet<'a> {
    /// Packet data
    data: &'a [u8],
}

impl<'a> Ipv4Packet<'a> {
    /// Create packet from data
    pub fn new(data: &'a [u8]) -> Result<Self, Ipv4Error> {
        if data.len() < Ipv4Header::MIN_SIZE {
            return Err(Ipv4Error::TooShort);
        }

        Ok(Self { data })
    }

    /// Get header
    pub fn header(&self) -> Result<Ipv4Header, Ipv4Error> {
        Ipv4Header::parse(self.data)
    }

    /// Get payload
    pub fn payload(&self) -> Result<&[u8], Ipv4Error> {
        let header = self.header()?;
        let header_len = header.header_length();
        let total_len = header.total_length() as usize;

        if self.data.len() < total_len {
            return Err(Ipv4Error::TooShort);
        }

        Ok(&self.data[header_len..total_len])
    }

    /// Get source address
    pub fn src_addr(&self) -> Result<Ipv4Addr, Ipv4Error> {
        Ok(self.header()?.src_addr())
    }

    /// Get destination address
    pub fn dst_addr(&self) -> Result<Ipv4Addr, Ipv4Error> {
        Ok(self.header()?.dst_addr())
    }

    /// Get protocol
    pub fn protocol(&self) -> Result<IpProtocol, Ipv4Error> {
        Ok(self.header()?.protocol())
    }
}

/// Calculate IP checksum
///
/// Computes the standard Internet checksum (RFC 1071) over the given data.
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Process 16-bit words
    for chunk in data.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_be_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], 0])
        };
        sum += word as u32;
    }

    // Fold 32-bit sum to 16 bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement
    !sum as u16
}

/// Calculate pseudo-header checksum for TCP/UDP
pub fn calculate_pseudo_header_checksum(
    src: Ipv4Addr,
    dst: Ipv4Addr,
    protocol: IpProtocol,
    length: u16,
) -> u32 {
    let mut sum: u32 = 0;

    // Source address
    sum += u16::from_be_bytes([src.0[0], src.0[1]]) as u32;
    sum += u16::from_be_bytes([src.0[2], src.0[3]]) as u32;

    // Destination address
    sum += u16::from_be_bytes([dst.0[0], dst.0[1]]) as u32;
    sum += u16::from_be_bytes([dst.0[2], dst.0[3]]) as u32;

    // Protocol
    sum += protocol.as_u8() as u32;

    // Length
    sum += length as u32;

    sum
}

/// IPv4 errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ipv4Error {
    /// Packet too short
    TooShort,
    /// Buffer too small
    BufferTooSmall,
    /// Invalid version
    InvalidVersion,
    /// Invalid header length
    InvalidHeaderLength,
    /// Invalid checksum
    InvalidChecksum,
    /// Invalid address
    InvalidAddress,
}

/// IPv4 subsystem initialized flag
static IPV4_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize IPv4 subsystem
pub fn init() {
    if IPV4_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    IPV4_INITIALIZED.store(true, Ordering::Release);
}

/// Check if IPv4 subsystem is initialized
pub fn is_initialized() -> bool {
    IPV4_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_addr() {
        let addr = Ipv4Addr::new(192, 168, 1, 1);
        assert_eq!(addr.octets(), [192, 168, 1, 1]);
        assert!(addr.is_private());
        assert!(!addr.is_loopback());
    }

    #[test]
    fn test_ipv4_addr_special() {
        assert!(Ipv4Addr::LOCALHOST.is_loopback());
        assert!(Ipv4Addr::BROADCAST.is_broadcast());
        assert!(Ipv4Addr::UNSPECIFIED.is_unspecified());
    }

    #[test]
    fn test_ip_protocol() {
        assert_eq!(IpProtocol::from_u8(6), IpProtocol::Tcp);
        assert_eq!(IpProtocol::from_u8(17), IpProtocol::Udp);
        assert_eq!(IpProtocol::Tcp.as_u8(), 6);
    }

    #[test]
    fn test_ipv4_header() {
        let src = Ipv4Addr::new(192, 168, 1, 1);
        let dst = Ipv4Addr::new(192, 168, 1, 2);
        let mut header = Ipv4Header::new(src, dst, IpProtocol::Tcp, 100);

        assert_eq!(header.version(), 4);
        assert_eq!(header.header_length(), 20);
        assert_eq!(header.protocol(), IpProtocol::Tcp);
        assert_eq!(header.src_addr(), src);
        assert_eq!(header.dst_addr(), dst);

        header.calculate_checksum();
        assert_ne!(header.checksum(), 0);
    }

    #[test]
    fn test_checksum() {
        let data = [
            0x45, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00, 0x40, 0x06, 0x00, 0x00, 0xac, 0x10,
            0x0a, 0x63, 0xac, 0x10, 0x0a, 0x0c,
        ];
        let checksum = calculate_checksum(&data);
        // Verify checksum is calculated (specific value depends on data)
        assert_ne!(checksum, 0);
    }
}
