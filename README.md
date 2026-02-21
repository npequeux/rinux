# Rinux

[![CI](https://github.com/npequeux/rinux/actions/workflows/ci.yml/badge.svg)](https://github.com/npequeux/rinux/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey.svg)](https://github.com/npequeux/rinux)

Rinux is a modern operating system kernel written in Rust, inspired by Linux's architecture and design principles. The project aims to leverage Rust's safety guarantees to build a robust, secure, and performant kernel.

## Features

### Currently Implemented ✅

- **x86_64 Architecture Support**: Full bootloader, interrupt handling, paging, and exception management
- **Memory Management**: Slab allocator, page fault handling, frame allocation, and virtual memory
- **VFS Layer**: POSIX-compliant virtual filesystem with working TmpFS
- **Device Drivers**: Serial (COM1-4), keyboard with LED control, VGA text mode, framebuffer graphics
- **GPU Support**: Detection and initialization for Intel i915, AMD (RDNA/GCN), NVIDIA (Maxwell+)
- **Storage Framework**: AHCI and NVMe driver structures with partition table support (GPT/MBR)
- **Console**: VGA text mode and multi-port serial console with configurable parameters

### Partially Implemented ⚠️

- **Process Management**: Task structures and scheduler framework (fork/exec not yet functional)
- **Block Storage**: Device detection and abstraction layer (DMA operations pending)
- **Filesystems**: ext2/ext4 structures defined (disk I/O not implemented)
- **System Calls**: Infrastructure and dispatcher (most syscalls are stubs)
- **USB Stack**: xHCI host controller framework (device enumeration pending)

### Planned 📋

- **Task Switching**: Full context switching and preemption
- **Network Stack**: TCP/IP implementation
- **Complete Storage**: DMA operations and interrupt-driven I/O
- **Additional Filesystems**: Complete ext2/ext4 with disk I/O
- **Multi-architecture**: ARM64 and RISC-V support

## Building

### Prerequisites

- Rust nightly toolchain
- QEMU (for testing)
- NASM assembler
- GNU Make

### Build Commands

```bash
# Build the kernel
make build

# Run in QEMU
make run

# Run tests
make test

# Clean build artifacts
make clean
```

## Project Structure

```
rinux/
├── arch/           # Architecture-specific code
│   └── x86/        # x86_64 support
├── kernel/         # Core kernel functionality
├── mm/             # Memory management
├── drivers/        # Device drivers
├── fs/             # File systems
├── net/            # Network stack
├── lib/            # Kernel libraries
└── src/            # Main kernel entry point
```

## License

Rinux is licensed under the MIT License.

## Documentation

### Overview & Testing
- [Test Coverage](docs/TEST_COVERAGE.md) - Unit test coverage report (61 tests, 100% pass rate)
- [Linux Kernel Coverage Comparison](docs/LINUX_COVERAGE.md) - Detailed comparison with Linux kernel
- [Coverage Summary](docs/COVERAGE_SUMMARY.md) - Quick reference for feature coverage

### Development & Architecture
- [Architecture Guide](docs/ARCHITECTURE.md) - System architecture overview
- [Development Guide](docs/DEVELOPMENT.md) - Developer documentation including testing
- [Roadmap](docs/ROADMAP.md) - Development roadmap and future plans

### Hardware Support
- [Hardware Support](docs/HARDWARE_SUPPORT.md) - Comprehensive hardware support documentation
- [Driver Development Guide](docs/DRIVER_DEVELOPMENT.md) - Guide for writing device drivers

## Contributing

Contributions are welcome! Please read CONTRIBUTING.md for details.
