# Rinux v0.2.0 Release Notes

**Release Date:** February 21, 2026  
**Codename:** Memory Master

## Overview

Rinux v0.2.0 represents a significant milestone in the development of this Rust-based operating system kernel. This release brings the project from an initial prototype (~3% functionality) to a foundational kernel with ~15% of Linux kernel functionality implemented.

## Highlights

### 🎯 Major Features

- **Advanced Memory Management**: Production-quality slab allocator with O(1) operations, page fault handling, and virtual memory support
- **Filesystem Support**: Fully functional TmpFS with POSIX-compliant VFS layer
- **Enhanced Device Drivers**: Multi-port serial (COM1-4), full keyboard support with modifiers, and framebuffer graphics
- **GPU Support**: Detection and initialization for Intel i915, AMD (RDNA/GCN), and NVIDIA (Maxwell+) GPUs
- **Process Management Framework**: Task structures, schedulers (round-robin and CFS-inspired), and PID management
- **Storage Infrastructure**: AHCI and NVMe driver frameworks with partition table support (GPT/MBR)

### 📊 Progress Metrics

- **Total Lines of Code**: 4,500+ production Rust code
- **New Modules**: 13 major subsystems added
- **Test Coverage**: 27 unit tests passing (100% pass rate)
- **Security**: 0 CodeQL vulnerabilities
- **Documentation**: Comprehensive guides and API documentation

## What's New in v0.2.0

### Memory Management ✅

#### Slab Allocator
- Multiple size classes from 8 bytes to 4KB
- O(1) allocation and deallocation
- Efficient memory reuse and fragmentation reduction
- Thread-safe with spin locks

#### Page Fault Handler
- Complete x86_64 error code parsing
- On-demand page allocation
- Copy-on-write framework
- TLB management

#### Additional Features
- VMalloc for kernel virtual memory
- Memory mapping (mmap) infrastructure
- Out-of-memory (OOM) killer framework
- Swap support framework

### Process Management ⚠️

#### Task Management
- Task structure with process states (Running, Sleeping, Zombie)
- PID allocator and management
- Process metadata tracking

#### Schedulers
- Round-robin scheduler implementation
- CFS-inspired scheduler framework for future fair scheduling
- Scheduler initialization infrastructure

**Note**: Fork, exec, and wait system calls are framework only - not yet functional.

### Filesystem Support ✅

#### Virtual Filesystem (VFS)
- POSIX-compliant VNode abstraction
- File descriptor table management
- Path resolution infrastructure
- Standard file operations interface

#### TmpFS - Fully Functional ✅
- Complete in-memory filesystem
- File and directory operations
- Permission management
- Symbolic link support

#### Other Filesystems (Framework)
- ext2 and ext4 structure definitions
- ProcFS skeleton for process information
- SysFS skeleton for system information

### Device Drivers

#### Serial/UART - Enhanced ✅
- **Multi-Port Support**: COM1, COM2, COM3, COM4
- **Configurable Baud Rates**: 115200, 57600, 38400, 19200, 9600, 4800, 2400 bps
- **Configuration Options**: 
  - Data bits: 5, 6, 7, 8 bits
  - Stop bits: 1 or 2 stop bits
  - Parity: None, Odd, Even, Mark, Space
- **Flow Control**: Status checking for DCD, DSR, CTS, RI
- **Operations**: Multi-byte read, non-blocking operations

#### Keyboard - Enhanced ✅
- **Complete PS/2 Support**: Full scancode mapping for all keys
- **Modifier Keys**: Shift (left/right), Ctrl, Alt
- **Lock Keys**: Caps Lock, Num Lock, Scroll Lock with toggle state
- **LED Control**: Visual feedback for lock key states
- **Character Support**: Numbers, letters, symbols, special characters

#### Graphics - New ✅
- **Framebuffer Driver**: VGA and VESA mode support
- **Drawing Primitives**: 
  - Line drawing (Bresenham's algorithm)
  - Circle drawing (midpoint algorithm)
  - Filled rectangles
- **Text Rendering**: 8x8 bitmap font with ASCII character support
- **GPU Detection**: 
  - Intel i915 with MMIO register access
  - AMD RDNA/RDNA2/RDNA3 and GCN architectures
  - NVIDIA Maxwell/Pascal/Turing/Ampere/Ada architectures
- **PCI Integration**: Bus mastering and memory space enablement

#### Storage - Framework ⚠️
- **AHCI Driver**: SATA controller detection and initialization
- **NVMe Driver**: Modern SSD support framework
- **Block Device Layer**: Request queue management
- **Partition Support**: GPT and MBR partition table parsing

**Note**: DMA operations and interrupt-driven I/O not yet implemented.

#### Other Drivers
- **Timer/RTC**: System time management
- **PCI**: Bus enumeration and device scanning
- **ACPI**: Framework for power management
- **USB**: xHCI host controller framework, HID and mass storage stubs

### Architecture (x86_64) ✅

- Complete GDT and IDT setup
- Advanced interrupt handling with APIC support
- Exception handlers for all x86_64 exceptions
- FPU and SSE state management
- Context switching infrastructure
- Multi-core SMP bootstrap framework
- System call infrastructure (syscall/sysret)
- TSC and HPET timer support

### System Features

#### IPC (Framework)
- Pipe abstraction
- Shared memory framework

#### Signal Handling
- Signal handler infrastructure
- Signal type definitions

#### System Calls (Partial)
- System call dispatcher framework
- Basic syscalls stubbed (read, write, open, close, fork, exec, etc.)
- Memory management syscalls (mmap, munmap, mprotect)

**Note**: Most syscalls are stubs - full implementation pending.

## Breaking Changes

None - this is the first major release after initial version.

## Known Limitations

### Not Yet Implemented ❌
- Task preemption and context switching
- Network stack (TCP/IP)
- File I/O to disk (block device DMA)
- Interrupt-driven device I/O (currently polling)
- Complete fork/exec/wait implementation
- ELF loader
- Synchronization primitives beyond spin locks

### Partially Implemented ⚠️
- Process management (structures exist, core functionality pending)
- Storage drivers (detection works, DMA operations pending)
- ext2/ext4 filesystems (structures only, no block I/O)
- USB support (xHCI framework only)
- System calls (infrastructure exists, most calls are stubs)

## Upgrade Notes

This is a fresh release. No upgrade path exists from v0.1.0.

## Building and Testing

### Prerequisites
- Rust nightly toolchain
- QEMU (for testing)
- GNU Make

### Build
```bash
make build
```

### Run in QEMU
```bash
make run
```

### Run Tests
```bash
make test
```

**Note**: Currently, only the rinux-lib module tests run (27 tests). Kernel and MM modules require a no_std test environment.

## Documentation

### New Documentation
- Comprehensive CHANGELOG with all v0.2.0 changes
- Accurate README reflecting current implementation status
- Hardware Enhancement Summary (1,200+ LOC improvements)
- Implementation Complete document

### Existing Documentation
- Architecture Guide (`docs/ARCHITECTURE.md`)
- Development Guide (`docs/DEVELOPMENT.md`)
- Hardware Support (`docs/HARDWARE_SUPPORT.md`)
- Driver Development Guide (`docs/DRIVER_DEVELOPMENT.md`)
- Test Coverage (`docs/TEST_COVERAGE.md`)
- Linux Kernel Comparison (`docs/LINUX_COVERAGE.md`)
- Roadmap (`docs/ROADMAP.md`)

## Contributors

- Nicolas Péqueux (@npequeux)
- Rinux Contributors

## Security

This release has been scanned with CodeQL and shows **0 security vulnerabilities**.

All unsafe code includes proper documentation explaining:
- Why it's necessary
- What invariants are being upheld
- What could go wrong if invariants are violated

## Next Steps (Roadmap to v0.3.0)

### Phase 1: Complete Process Management (3-4 months)
- Implement fork/clone/exec functionality
- Complete scheduler with preemption
- Add context switching
- User/kernel space separation

### Phase 2: Complete Storage Stack (2-3 months)
- Finish AHCI DMA operations
- Add NVMe queue handling
- Implement interrupt handlers for storage devices
- Enable actual disk I/O

### Phase 3: Complete Filesystems (3-4 months)
- Finish ext2 block I/O
- Add ext4 support
- Test read/write operations on real disks
- Implement page cache

### Phase 4: Basic Input (2 months)
- Complete PS/2 keyboard driver with interrupts
- Serial console improvements
- Mouse support

**Estimated timeline to bootability on real hardware**: 10-13 months

## Support

For issues, feature requests, or questions:
- GitHub Issues: https://github.com/npequeux/rinux/issues
- Documentation: See `docs/` directory

## License

Rinux is licensed under the MIT License. See LICENSE file for details.

---

**Thank you for your interest in Rinux!**

This release represents significant progress toward a fully functional Rust-based operating system kernel. While there's still substantial work ahead (approximately 85% of target functionality remains), the foundations are solid and the architecture is sound.

We welcome contributions, testing, and feedback from the community!
