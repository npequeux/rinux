//! User Datagram Protocol (UDP)
//!
//! UDP packet handling and port management.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use spin::Mutex;

use super::ipv4::{calculate_pseudo_header_checksum, IpProtocol, Ipv4Addr};
use super::socket::{SocketAddrV4, SocketError};

/// UDP header (8 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct UdpHeader {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Length (header + data)
    pub length: u16,
    /// Checksum
    pub checksum: u16,
}

impl UdpHeader {
    /// UDP header size
    pub const SIZE: usize = 8;

    /// Create new UDP header
    pub const fn new(src_port: u16, dst_port: u16, length: u16) -> Self {
        Self {
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
            length: length.to_be(),
            checksum: 0,
        }
    }

    /// Parse header from bytes
    pub fn parse(data: &[u8]) -> Result<Self, UdpError> {
        if data.len() < Self::SIZE {
            return Err(UdpError::TooShort);
        }

        Ok(Self {
            src_port: u16::from_be_bytes([data[0], data[1]]).to_be(),
            dst_port: u16::from_be_bytes([data[2], data[3]]).to_be(),
            length: u16::from_be_bytes([data[4], data[5]]).to_be(),
            checksum: u16::from_be_bytes([data[6], data[7]]).to_be(),
        })
    }

    /// Write header to buffer
    pub fn write_to(&self, buffer: &mut [u8]) -> Result<(), UdpError> {
        if buffer.len() < Self::SIZE {
            return Err(UdpError::BufferTooSmall);
        }

        buffer[0..2].copy_from_slice(&u16::from_be(self.src_port).to_be_bytes());
        buffer[2..4].copy_from_slice(&u16::from_be(self.dst_port).to_be_bytes());
        buffer[4..6].copy_from_slice(&u16::from_be(self.length).to_be_bytes());
        buffer[6..8].copy_from_slice(&u16::from_be(self.checksum).to_be_bytes());

        Ok(())
    }

    /// Get source port
    pub fn src_port(&self) -> u16 {
        u16::from_be(self.src_port)
    }

    /// Get destination port
    pub fn dst_port(&self) -> u16 {
        u16::from_be(self.dst_port)
    }

    /// Get length
    pub fn length(&self) -> u16 {
        u16::from_be(self.length)
    }

    /// Get checksum
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.checksum)
    }

    /// Calculate checksum
    pub fn calculate_checksum(&mut self, src_ip: Ipv4Addr, dst_ip: Ipv4Addr, payload: &[u8]) {
        // Zero out checksum field
        self.checksum = 0;

        // Build pseudo-header + UDP header + payload for checksum calculation
        let length = Self::SIZE + payload.len();
        let mut buffer = Vec::with_capacity(length);

        // Serialize UDP header
        let mut header_bytes = [0u8; Self::SIZE];
        let _ = self.write_to(&mut header_bytes);
        buffer.extend_from_slice(&header_bytes);

        // Add payload
        buffer.extend_from_slice(payload);

        // Calculate pseudo-header checksum
        let mut sum =
            calculate_pseudo_header_checksum(src_ip, dst_ip, IpProtocol::Udp, length as u16);

        // Add UDP header and payload checksum
        for chunk in buffer.chunks(2) {
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
        let checksum = !sum as u16;

        // UDP checksum of 0 means no checksum
        self.checksum = if checksum == 0 { 0xFFFF } else { checksum }.to_be();
    }

    /// Verify checksum
    pub fn verify_checksum(&self, src_ip: Ipv4Addr, dst_ip: Ipv4Addr, payload: &[u8]) -> bool {
        if self.checksum() == 0 {
            // Checksum is optional in UDP; 0 means no checksum
            return true;
        }

        let mut header = *self;
        header.calculate_checksum(src_ip, dst_ip, payload);
        header.checksum() == self.checksum()
    }
}

/// UDP packet
pub struct UdpPacket<'a> {
    /// Packet data
    data: &'a [u8],
}

impl<'a> UdpPacket<'a> {
    /// Create packet from data
    pub fn new(data: &'a [u8]) -> Result<Self, UdpError> {
        if data.len() < UdpHeader::SIZE {
            return Err(UdpError::TooShort);
        }

        Ok(Self { data })
    }

    /// Get header
    pub fn header(&self) -> Result<UdpHeader, UdpError> {
        UdpHeader::parse(self.data)
    }

    /// Get payload
    pub fn payload(&self) -> &[u8] {
        &self.data[UdpHeader::SIZE..]
    }

    /// Get source port
    pub fn src_port(&self) -> Result<u16, UdpError> {
        Ok(self.header()?.src_port())
    }

    /// Get destination port
    pub fn dst_port(&self) -> Result<u16, UdpError> {
        Ok(self.header()?.dst_port())
    }
}

/// UDP port range
pub const UDP_PORT_MIN: u16 = 1024;
pub const UDP_PORT_MAX: u16 = 65535;

/// UDP port manager
pub struct UdpPortManager {
    /// Next ephemeral port
    next_port: AtomicU16,
    /// Bound ports
    bound_ports: BTreeMap<u16, ()>,
}

impl UdpPortManager {
    /// Create new port manager
    pub fn new() -> Self {
        Self {
            next_port: AtomicU16::new(UDP_PORT_MIN),
            bound_ports: BTreeMap::new(),
        }
    }

    /// Allocate ephemeral port
    pub fn allocate_ephemeral(&mut self) -> Result<u16, UdpError> {
        let start = self.next_port.load(Ordering::Relaxed);
        let mut port = start;

        loop {
            if !self.bound_ports.contains_key(&port) {
                self.bound_ports.insert(port, ());
                self.next_port.store(
                    if port == UDP_PORT_MAX {
                        UDP_PORT_MIN
                    } else {
                        port + 1
                    },
                    Ordering::Relaxed,
                );
                return Ok(port);
            }

            port = if port == UDP_PORT_MAX {
                UDP_PORT_MIN
            } else {
                port + 1
            };

            if port == start {
                return Err(UdpError::NoPortsAvailable);
            }
        }
    }

    /// Bind specific port
    pub fn bind(&mut self, port: u16) -> Result<(), UdpError> {
        if self.bound_ports.contains_key(&port) {
            return Err(UdpError::PortInUse);
        }

        self.bound_ports.insert(port, ());
        Ok(())
    }

    /// Release port
    pub fn release(&mut self, port: u16) {
        self.bound_ports.remove(&port);
    }

    /// Check if port is bound
    pub fn is_bound(&self, port: u16) -> bool {
        self.bound_ports.contains_key(&port)
    }
}

impl Default for UdpPortManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global UDP port manager
static UDP_PORT_MANAGER: Mutex<UdpPortManager> = Mutex::new(UdpPortManager {
    next_port: AtomicU16::new(UDP_PORT_MIN),
    bound_ports: BTreeMap::new(),
});

/// Allocate ephemeral port
pub fn allocate_ephemeral_port() -> Result<u16, UdpError> {
    UDP_PORT_MANAGER.lock().allocate_ephemeral()
}

/// Bind port
pub fn bind_port(port: u16) -> Result<(), UdpError> {
    UDP_PORT_MANAGER.lock().bind(port)
}

/// Release port
pub fn release_port(port: u16) {
    UDP_PORT_MANAGER.lock().release(port);
}

/// Check if port is bound
pub fn is_port_bound(port: u16) -> bool {
    UDP_PORT_MANAGER.lock().is_bound(port)
}

/// UDP socket implementation
pub struct UdpSocket {
    /// Local address
    local_addr: Option<SocketAddrV4>,
    /// Remote address
    remote_addr: Option<SocketAddrV4>,
    /// Receive buffer
    recv_buffer: Vec<(Vec<u8>, SocketAddrV4)>,
}

impl UdpSocket {
    /// Create new UDP socket
    pub fn new() -> Result<Self, SocketError> {
        Ok(Self {
            local_addr: None,
            remote_addr: None,
            recv_buffer: Vec::new(),
        })
    }

    /// Bind socket to local address
    pub fn bind(&mut self, addr: SocketAddrV4) -> Result<(), SocketError> {
        if self.local_addr.is_some() {
            return Err(SocketError::AlreadyConnected);
        }

        bind_port(addr.port).map_err(|_| SocketError::AddrInUse)?;
        self.local_addr = Some(addr);
        Ok(())
    }

    /// Connect socket to remote address
    pub fn connect(&mut self, addr: SocketAddrV4) -> Result<(), SocketError> {
        // UDP connect is just setting the default destination
        self.remote_addr = Some(addr);

        // If not bound, bind to ephemeral port
        if self.local_addr.is_none() {
            let port = allocate_ephemeral_port().map_err(|_| SocketError::AddrNotAvail)?;
            self.local_addr = Some(SocketAddrV4 {
                ip: [0, 0, 0, 0], // Bind to any address
                port,
            });
        }

        Ok(())
    }

    /// Send data to address
    pub fn send_to(&mut self, data: &[u8], dst: SocketAddrV4) -> Result<usize, SocketError> {
        let local = self.local_addr.ok_or(SocketError::NotConnected)?;

        // Build UDP packet
        let mut buffer = [0u8; 1600];
        let _packet_len = build_udp_packet(
            &mut buffer,
            Ipv4Addr(local.ip),
            local.port,
            Ipv4Addr(dst.ip),
            dst.port,
            data,
        )
        .map_err(|_| SocketError::InvalidArg)?;

        // TODO: Send packet via network stack
        // For now, just return success
        Ok(data.len())
    }

    /// Send data to connected address
    pub fn send(&mut self, data: &[u8]) -> Result<usize, SocketError> {
        let remote = self.remote_addr.ok_or(SocketError::NotConnected)?;
        self.send_to(data, remote)
    }

    /// Receive data from any address
    pub fn recv_from(&mut self, buffer: &mut [u8]) -> Result<(usize, SocketAddrV4), SocketError> {
        if self.recv_buffer.is_empty() {
            return Err(SocketError::WouldBlock);
        }

        let (data, addr) = self.recv_buffer.remove(0);
        let len = data.len().min(buffer.len());
        buffer[..len].copy_from_slice(&data[..len]);
        Ok((len, addr))
    }

    /// Receive data from connected address
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, SocketError> {
        let (len, _addr) = self.recv_from(buffer)?;
        Ok(len)
    }

    /// Get local address
    pub fn local_addr(&self) -> Option<SocketAddrV4> {
        self.local_addr
    }

    /// Get remote address
    pub fn remote_addr(&self) -> Option<SocketAddrV4> {
        self.remote_addr
    }

    /// Close socket
    pub fn close(&mut self) -> Result<(), SocketError> {
        if let Some(addr) = self.local_addr {
            release_port(addr.port);
        }
        self.local_addr = None;
        self.remote_addr = None;
        self.recv_buffer.clear();
        Ok(())
    }
}

impl Default for UdpSocket {
    fn default() -> Self {
        Self {
            local_addr: None,
            remote_addr: None,
            recv_buffer: Vec::new(),
        }
    }
}

/// Build UDP packet
pub fn build_udp_packet(
    buffer: &mut [u8],
    src_ip: Ipv4Addr,
    src_port: u16,
    dst_ip: Ipv4Addr,
    dst_port: u16,
    payload: &[u8],
) -> Result<usize, UdpError> {
    let total_len = UdpHeader::SIZE + payload.len();
    if buffer.len() < total_len {
        return Err(UdpError::BufferTooSmall);
    }

    // Create UDP header
    let mut header = UdpHeader::new(src_port, dst_port, total_len as u16);
    header.calculate_checksum(src_ip, dst_ip, payload);

    // Write header
    header.write_to(buffer)?;

    // Write payload
    buffer[UdpHeader::SIZE..total_len].copy_from_slice(payload);

    Ok(total_len)
}

/// Process incoming UDP packet
pub fn process_packet(
    ip_payload: &[u8],
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
) -> Result<(), UdpError> {
    let packet = UdpPacket::new(ip_payload)?;
    let header = packet.header()?;

    // Verify checksum
    if !header.verify_checksum(src_ip, dst_ip, packet.payload()) {
        return Err(UdpError::InvalidChecksum);
    }

    // TODO: Deliver to appropriate socket
    // For now, just validate the packet
    Ok(())
}

/// UDP errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpError {
    /// Packet too short
    TooShort,
    /// Buffer too small
    BufferTooSmall,
    /// Invalid checksum
    InvalidChecksum,
    /// Port in use
    PortInUse,
    /// No ports available
    NoPortsAvailable,
    /// Socket not bound
    NotBound,
}

/// UDP subsystem initialized flag
static UDP_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize UDP subsystem
pub fn init() {
    if UDP_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    UDP_INITIALIZED.store(true, Ordering::Release);
}

/// Check if UDP subsystem is initialized
pub fn is_initialized() -> bool {
    UDP_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_header() {
        let header = UdpHeader::new(12345, 80, 100);
        assert_eq!(header.src_port(), 12345);
        assert_eq!(header.dst_port(), 80);
        assert_eq!(header.length(), 100);
    }

    #[test]
    fn test_udp_port_manager() {
        let mut manager = UdpPortManager::new();

        // Allocate ephemeral port
        let port1 = manager.allocate_ephemeral().unwrap();
        assert!(port1 >= UDP_PORT_MIN && port1 <= UDP_PORT_MAX);
        assert!(manager.is_bound(port1));

        // Bind specific port
        let port2 = 8080;
        manager.bind(port2).unwrap();
        assert!(manager.is_bound(port2));

        // Try to bind same port again
        assert_eq!(manager.bind(port2), Err(UdpError::PortInUse));

        // Release port
        manager.release(port1);
        assert!(!manager.is_bound(port1));
    }

    #[test]
    fn test_udp_checksum() {
        let src_ip = Ipv4Addr::new(192, 168, 1, 1);
        let dst_ip = Ipv4Addr::new(192, 168, 1, 2);
        let payload = b"Hello, UDP!";

        let mut header = UdpHeader::new(12345, 80, (UdpHeader::SIZE + payload.len()) as u16);
        header.calculate_checksum(src_ip, dst_ip, payload);

        assert_ne!(header.checksum(), 0);
        assert!(header.verify_checksum(src_ip, dst_ip, payload));
    }
}
