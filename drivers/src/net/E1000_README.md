# Intel e1000/e1000e Network Driver

This is a comprehensive network driver for Intel 82540/82545/82574 (e1000/e1000e) Gigabit Ethernet adapters for the Rinux operating system.

## Features

### Hardware Support
- **Vendor**: Intel (0x8086)
- **Device IDs Supported**:
  - 82540EM (0x100E) - Gigabit Ethernet
  - 82545EM (0x100F) - Gigabit Ethernet  
  - 82543GC (0x1004) - Gigabit Ethernet
  - 82544EI (0x1008) - Gigabit Ethernet
  - 82544GC (0x100C) - Gigabit Ethernet
  - 82547EI (0x1019) - Gigabit Ethernet
  - 82541EI (0x1013) - Gigabit Ethernet
  - 82541GI (0x1076) - Gigabit Ethernet
  - 82547GI (0x1075) - Gigabit Ethernet
  - 82574L (0x10D3) - Gigabit Ethernet (e1000e)
  - 82571EB (0x105E) - Gigabit Ethernet (e1000e)
  - 82572EI (0x107C) - Gigabit Ethernet (e1000e)
  - 82573E (0x108B) - Gigabit Ethernet (e1000e)
  - 82573L (0x109A) - Gigabit Ethernet (e1000e)

### Driver Capabilities

#### 1. PCI Device Detection
- Scans PCI bus for Intel e1000 devices
- Reads and validates PCI BARs
- Maps MMIO regions for register access
- Enables bus mastering and memory space access

#### 2. Device Initialization
- Proper device reset sequence
- MAC address reading from EEPROM/registers
- RX/TX descriptor ring setup (256 descriptors each)
- Interrupt configuration (Link Status, RX, TX)
- Link negotiation and status tracking

#### 3. Transmit (TX) Operations
- Circular descriptor ring (256 descriptors)
- DMA buffer management (2048 bytes per buffer)
- Automatic packet queuing
- TX completion handling
- Full ring detection and backpressure

#### 4. Receive (RX) Operations
- Circular descriptor ring (256 descriptors)  
- DMA buffer management (2048 bytes per buffer)
- Automatic buffer replenishment
- Error detection and handling
- Statistics tracking

#### 5. NetDevice Integration
- Full `NetDevice` trait implementation
- Registration with kernel network stack
- Link state monitoring
- Comprehensive statistics (packets, bytes, errors, drops)

## Architecture

### Memory Layout

```
E1000Driver
├── State (Mutex)
│   ├── MMIO Base Address (mapped to kernel virtual memory)
│   ├── MAC Address
│   ├── Link State (AtomicBool)
│   ├── Device State (AtomicBool)
│   ├── RX Ring
│   │   ├── Descriptors [256] (aligned to 16 bytes)
│   │   └── DMA Buffers [256] (2048 bytes each)
│   └── TX Ring
│       ├── Descriptors [256] (aligned to 16 bytes)
│       └── DMA Buffers [256] (2048 bytes each)
└── Statistics (lock-free atomics)
```

### Descriptor Format

#### RX Descriptor
```rust
struct RxDesc {
    addr: u64,      // Physical address of buffer
    length: u16,    // Length of received data
    checksum: u16,  // Checksum
    status: u8,     // Status flags (DD=descriptor done)
    errors: u8,     // Error flags
    special: u16,   // VLAN/special fields
}
```

#### TX Descriptor
```rust
struct TxDesc {
    addr: u64,      // Physical address of buffer
    length: u16,    // Length of data to transmit
    cso: u8,        // Checksum offset
    cmd: u8,        // Command (EOP, IFCS, RS)
    status: u8,     // Status flags (DD=descriptor done)
    css: u8,        // Checksum start
    special: u16,   // VLAN/special fields
}
```

## Register Programming

The driver programs the following key registers:

### Control Registers
- **CTRL** (0x0000): Device control, reset, link setup
- **STATUS** (0x0008): Device status, link state
- **CTRL_EXT** (0x0018): Extended control

### Interrupt Registers
- **ICR** (0x00C0): Interrupt cause read
- **IMS** (0x00D0): Interrupt mask set
- **IMC** (0x00D8): Interrupt mask clear

### Receive Registers
- **RCTL** (0x0100): Receive control
- **RDBAL/RDBAH** (0x2800/0x2804): RX descriptor base
- **RDLEN** (0x2808): RX descriptor length
- **RDH/RDT** (0x2810/0x2818): RX head/tail pointers

### Transmit Registers
- **TCTL** (0x0400): Transmit control
- **TDBAL/TDBAH** (0x3800/0x3804): TX descriptor base
- **TDLEN** (0x3808): TX descriptor length
- **TDH/TDT** (0x3810/0x3818): TX head/tail pointers

### MAC Address Registers
- **RAL/RAH** (0x5400/0x5404): Receive address low/high

## Usage

### Initialization

The driver is automatically initialized when the network driver subsystem starts:

```rust
// Called from drivers::net::init()
e1000::init();
```

This will:
1. Scan PCI bus for supported devices
2. Initialize each found device
3. Register with the network stack as `eth0`, `eth1`, etc.

### Sending Packets

```rust
use rinux_kernel::net::netdev;

let device = netdev::get_device("eth0").unwrap();
let mut dev = device.lock();

let packet = [...]; // Ethernet frame
dev.send(&packet)?;
```

### Receiving Packets

```rust
use rinux_kernel::net::netdev;

let device = netdev::get_device("eth0").unwrap();
let mut dev = device.lock();

let mut buffer = [0u8; 2048];
match dev.recv(&mut buffer) {
    Ok(len) => {
        // Process packet in buffer[..len]
    }
    Err(NetDevError::WouldBlock) => {
        // No packet available
    }
    Err(e) => {
        // Handle error
    }
}
```

### Link Status

```rust
use rinux_kernel::net::netdev::{LinkState, get_device};

let device = get_device("eth0").unwrap();
let dev = device.lock();

match dev.link_state() {
    LinkState::Up => println!("Link is up"),
    LinkState::Down => println!("Link is down"),
    LinkState::Unknown => println!("Link state unknown"),
}
```

## Thread Safety

The driver is thread-safe:
- Device state is protected by `Mutex`
- Statistics use lock-free atomics (`AtomicU32`)
- Ring buffer indices use `AtomicU32` for lock-free updates
- Can be safely shared across multiple threads

## Error Handling

The driver returns detailed error codes:
- `NetDevError::NoMemory`: Failed to allocate DMA buffers
- `NetDevError::BufferTooSmall`: Packet too large for buffer
- `NetDevError::Busy`: TX ring full
- `NetDevError::WouldBlock`: No RX packet available
- `NetDevError::DeviceDown`: Device not up
- `NetDevError::InvalidParam`: Invalid parameter or corrupted packet

## Performance Characteristics

- **RX Ring Size**: 256 descriptors
- **TX Ring Size**: 256 descriptors
- **Buffer Size**: 2048 bytes per descriptor
- **MTU**: 1500 bytes (standard Ethernet)
- **Interrupt Coalescing**: Enabled
- **Checksum Offload**: Not yet implemented
- **Scatter-Gather**: Not yet implemented

## Limitations

Current limitations (to be addressed in future versions):
1. Single frame per descriptor (no scatter-gather)
2. No checksum offload
3. No VLAN support
4. No interrupt handler integration (uses polling)
5. Fixed ring sizes
6. Single buffer allocation (needs contiguous multi-frame DMA)

## Testing

The driver has been tested with:
- QEMU e1000 emulation
- VirtualBox Intel PRO/1000 emulation

For hardware testing, ensure:
1. Device is recognized during PCI scan
2. MMIO BAR is properly mapped
3. Bus mastering is enabled
4. Interrupts are configured

## References

- [Intel 8254x GbE Controller Software Developer's Manual](https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf)
- [OSDev Wiki - Intel 8254x](https://wiki.osdev.org/Intel_8254x)
- Linux e1000 driver source code

## License

This driver is part of the Rinux operating system and is licensed under the MIT License.
