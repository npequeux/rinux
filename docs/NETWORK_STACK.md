# Rinux Network Stack Documentation

## Overview

The Rinux network stack provides a basic but functional TCP/IP implementation for the kernel. It follows a layered architecture similar to the OSI model and provides clean abstractions for network operations.

## Architecture

The network stack is organized into the following layers:

```
┌─────────────────────────────────────────┐
│         Socket Layer (BSD API)          │
├─────────────────────────────────────────┤
│      Transport Layer (UDP/TCP*)         │
├─────────────────────────────────────────┤
│       Network Layer (IPv4, ARP)         │
├─────────────────────────────────────────┤
│      Data Link Layer (Ethernet)         │
├─────────────────────────────────────────┤
│     Network Device Framework            │
└─────────────────────────────────────────┘

* TCP not yet implemented
```

## Components

### 1. Network Device Framework (`netdev.rs`)

The lowest layer provides an abstraction for network hardware devices.

**Key Features:**
- `NetDevice` trait for device drivers
- Device registration and management
- Link state tracking (Up/Down/Unknown)
- MTU (Maximum Transmission Unit) management
- Device statistics (packets, bytes, errors, drops)
- Device capabilities (checksum offload, scatter-gather, VLAN support)

**Usage Example:**
```rust
use rinux_kernel::net::netdev::{register_device, get_device};

// Register a network device
register_device(Arc::new(Mutex::new(my_device)))?;

// Get device by name
let device = get_device("eth0")?;
```

### 2. Ethernet Layer (`ethernet.rs`)

Handles Ethernet frame parsing and construction.

**Key Features:**
- MAC address representation and manipulation
- Ethernet frame parsing and building
- EtherType handling (IPv4, ARP, IPv6, VLAN)
- Frame validation (size checks)
- Broadcast/multicast/unicast address detection

**Data Structures:**
- `MacAddress`: 6-byte MAC address
- `EthernetHeader`: 14-byte Ethernet frame header
- `EthernetFrame`: Complete Ethernet frame parser
- `EthernetFrameBuilder`: Builder pattern for frame construction

**Usage Example:**
```rust
use rinux_kernel::net::ethernet::{MacAddress, EtherType, EthernetFrameBuilder};

let dst_mac = MacAddress::BROADCAST;
let src_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

let mut buffer = [0u8; 1500];
let len = EthernetFrameBuilder::new()
    .dst(dst_mac)
    .src(src_mac)
    .ethertype(EtherType::Ipv4)
    .build(&mut buffer, payload)?;
```

### 3. ARP Protocol (`arp.rs`)

Address Resolution Protocol for IP-to-MAC address mapping.

**Key Features:**
- ARP cache/table for storing IP-to-MAC mappings
- ARP request/reply packet building and parsing
- Automatic cache management with TTL
- ARP packet processing
- Global ARP cache with thread-safe access

**Data Structures:**
- `ArpHeader`: ARP packet header
- `ArpCache`: IP-to-MAC address mapping table
- `ArpEntry`: Single cache entry with TTL

**Usage Example:**
```rust
use rinux_kernel::net::arp::{lookup_mac, insert_entry, build_request};

// Lookup MAC address for IP
if let Some(mac) = lookup_mac(target_ip) {
    // Use MAC address
}

// Build ARP request
let mut buffer = [0u8; 64];
let len = build_request(&mut buffer, src_mac, src_ip, target_ip)?;
```

### 4. IPv4 Implementation (`ipv4.rs`)

Internet Protocol version 4 packet handling.

**Key Features:**
- IPv4 address representation with classification methods
- IPv4 packet header parsing and building
- Checksum calculation and verification
- Pseudo-header checksum for TCP/UDP
- Address validation (private, loopback, multicast, etc.)
- Protocol identification (TCP, UDP, ICMP)

**Data Structures:**
- `Ipv4Addr`: 4-byte IPv4 address
- `Ipv4Header`: 20-60 byte IPv4 packet header
- `Ipv4Packet`: Complete IPv4 packet parser
- `IpProtocol`: Protocol number enumeration

**Usage Example:**
```rust
use rinux_kernel::net::ipv4::{Ipv4Addr, Ipv4Header, IpProtocol};

let src = Ipv4Addr::new(192, 168, 1, 1);
let dst = Ipv4Addr::new(192, 168, 1, 2);

let mut header = Ipv4Header::new(src, dst, IpProtocol::Udp, payload_len);
header.calculate_checksum();
```

### 5. UDP Implementation (`udp.rs`)

User Datagram Protocol for connectionless communication.

**Key Features:**
- UDP packet parsing and building
- Port allocation and management
- UDP socket implementation
- Checksum calculation and verification (with pseudo-header)
- Port binding and ephemeral port allocation
- Datagram send/receive operations

**Data Structures:**
- `UdpHeader`: 8-byte UDP header
- `UdpPacket`: UDP packet parser
- `UdpSocket`: UDP socket implementation
- `UdpPortManager`: Port allocation manager

**Usage Example:**
```rust
use rinux_kernel::net::udp::UdpSocket;
use rinux_kernel::net::socket::SocketAddrV4;

let mut socket = UdpSocket::new()?;

// Bind to local address
let local_addr = SocketAddrV4 {
    ip: [0, 0, 0, 0],  // Bind to any
    port: 8080,
};
socket.bind(local_addr)?;

// Send data
let remote_addr = SocketAddrV4 {
    ip: [192, 168, 1, 2],
    port: 9090,
};
socket.send_to(b"Hello, UDP!", remote_addr)?;

// Receive data
let mut buffer = [0u8; 1500];
let (len, from) = socket.recv_from(&mut buffer)?;
```

### 6. Socket Layer (`socket.rs`)

BSD socket API implementation.

**Key Features:**
- Socket types (Stream, Datagram, Raw)
- Address families (AF_INET, AF_INET6, AF_UNIX)
- Socket operations (bind, connect, send, recv, sendto, recvfrom)
- Socket state management
- Socket options (reuse_addr, keep_alive, timeouts, etc.)
- File descriptor management

**Data Structures:**
- `Socket` trait: Common socket interface
- `SocketAddr`: Socket address (IPv4/IPv6/Unix)
- `SocketAddrV4`: IPv4 socket address (IP + port)
- `SocketError`: Socket error enumeration
- `SocketTable`: File descriptor to socket mapping

**Usage Example:**
```rust
use rinux_kernel::net::socket::{socket, bind, send, recv};
use rinux_kernel::net::socket::{SocketDomain, SocketType, SocketProtocol};
use rinux_kernel::net::socket::{SocketAddr, SocketAddrV4};

// Create UDP socket
let fd = socket(SocketDomain::Inet, SocketType::Dgram, SocketProtocol::Udp)?;

// Bind to address
let addr = SocketAddr::V4(SocketAddrV4 {
    ip: [0, 0, 0, 0],
    port: 8080,
});
bind(fd, addr)?;

// Send/receive data
send(fd, b"Hello", 0)?;
let mut buffer = [0u8; 1500];
let len = recv(fd, &mut buffer, 0)?;
```

## Design Principles

### 1. Safety First
- All modules use `#![no_std]` for kernel environment
- Extensive use of Rust's type system for correctness
- Clear documentation of safety requirements
- Minimal use of `unsafe` code

### 2. Layered Architecture
- Clean separation between layers
- Each layer only depends on layers below it
- Well-defined interfaces between layers

### 3. Thread Safety
- All shared state protected by `Mutex` or atomic operations
- Lock-free operations where possible (statistics, flags)
- No deadlock potential in current implementation

### 4. Resource Management
- Automatic port allocation for ephemeral ports
- Device registration/unregistration
- Proper cleanup on socket close

### 5. Error Handling
- Custom error types for each layer
- Clear error propagation
- No panics in normal operation

## Implementation Status

### ✅ Implemented
- Network device framework
- Ethernet frame handling
- ARP protocol (request/reply, cache)
- IPv4 packet handling
- UDP implementation
- Socket API (partial)
- Checksum calculation

### ⏳ Planned
- TCP implementation
- ICMP protocol
- IPv6 support
- Routing table
- NAT/firewall
- Socket options (full implementation)
- Network namespaces

### 🚧 Limitations
- No actual packet transmission (requires device driver integration)
- No receive packet processing loop
- No TCP support yet
- Limited socket options
- No routing beyond single subnet
- No fragmentation/reassembly

## Testing

Each module includes unit tests for core functionality:

```bash
# Run all kernel tests (including network stack)
cargo test --package rinux-kernel

# Run specific module tests
cargo test --package rinux-kernel --lib net::ethernet
cargo test --package rinux-kernel --lib net::ipv4
cargo test --package rinux-kernel --lib net::udp
```

## Integration with Device Drivers

To integrate a network device driver:

1. Implement the `NetDevice` trait for your device
2. Register the device with `register_device()`
3. Handle send/receive operations
4. Update device statistics

Example skeleton:

```rust
use rinux_kernel::net::netdev::{NetDevice, NetDevError, LinkState};
use rinux_kernel::net::ethernet::MacAddress;

struct MyNetworkDevice {
    mac: MacAddress,
    // ... device-specific fields
}

impl NetDevice for MyNetworkDevice {
    fn name(&self) -> &str {
        "eth0"
    }

    fn mac_address(&self) -> MacAddress {
        self.mac
    }

    fn link_state(&self) -> LinkState {
        LinkState::Up
    }

    fn send(&mut self, packet: &[u8]) -> Result<(), NetDevError> {
        // Send packet to hardware
        Ok(())
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, NetDevError> {
        // Receive packet from hardware
        Ok(0)
    }

    // ... implement other methods
}
```

## Security Considerations

⚠️ **Warning**: This is a basic implementation and has several security limitations:

1. **No Access Control**: All sockets accessible to kernel
2. **No Rate Limiting**: Susceptible to packet floods
3. **No Packet Filtering**: All packets accepted
4. **Limited Validation**: Minimal input validation
5. **No Encryption**: Plain text transmission

**Do not use in production without additional security measures.**

## Performance Considerations

The current implementation prioritizes correctness over performance:

- Uses locks for all shared state (Mutex)
- Allocates memory for packet buffers
- No zero-copy operations
- No packet batching
- No hardware offload support (planned)

Future optimizations:
- Lock-free data structures where possible
- Memory pool for packet buffers
- DMA support for device drivers
- Hardware checksum offload
- Interrupt coalescing

## Future Work

1. **TCP Implementation**
   - Connection management
   - Reliable delivery
   - Flow control
   - Congestion control

2. **Routing**
   - Routing table
   - Default gateway
   - Multi-interface support

3. **Advanced Features**
   - Socket filters (BPF)
   - Raw sockets
   - Packet capture
   - Network statistics

4. **IPv6 Support**
   - IPv6 addressing
   - Neighbor Discovery (NDP)
   - ICMPv6

## Contributing

When contributing to the network stack:

1. Follow Rinux coding standards
2. Add comprehensive documentation
3. Include unit tests for new functionality
4. Test with actual network devices when possible
5. Consider security implications
6. Update this documentation

## References

- [RFC 791](https://tools.ietf.org/html/rfc791) - Internet Protocol (IPv4)
- [RFC 768](https://tools.ietf.org/html/rfc768) - User Datagram Protocol (UDP)
- [RFC 826](https://tools.ietf.org/html/rfc826) - Address Resolution Protocol (ARP)
- [RFC 1071](https://tools.ietf.org/html/rfc1071) - Computing the Internet Checksum
- [RFC 894](https://tools.ietf.org/html/rfc894) - IP over Ethernet
