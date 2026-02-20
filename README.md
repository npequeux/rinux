# Rinux

[![CI](https://github.com/npequeux/rinux/actions/workflows/ci.yml/badge.svg)](https://github.com/npequeux/rinux/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey.svg)](https://github.com/npequeux/rinux)

Rinux is a modern operating system kernel written in Rust, inspired by Linux's architecture and design principles. The project aims to leverage Rust's safety guarantees to build a robust, secure, and performant kernel.

## Features

- **x86_64 Architecture Support**: Initial focus on PC hardware with x86_64 architecture
- **Memory Management**: Advanced memory allocation and paging
- **Process Management**: Multi-tasking and scheduling
- **Device Drivers**: Modular driver architecture
- **File Systems**: VFS abstraction layer
- **Network Stack**: TCP/IP implementation
- **Extensible Architecture**: Support for additional architectures planned

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

- [Test Coverage](docs/TEST_COVERAGE.md) - Unit test coverage report (61 tests, 100% pass rate)
- [Linux Kernel Coverage Comparison](docs/LINUX_COVERAGE.md) - Detailed comparison with Linux kernel
- [Coverage Summary](docs/COVERAGE_SUMMARY.md) - Quick reference for feature coverage
- [Architecture Guide](docs/ARCHITECTURE.md) - System architecture overview
- [Development Guide](docs/DEVELOPMENT.md) - Developer documentation including testing
- [Roadmap](docs/ROADMAP.md) - Development roadmap and future plans

## Contributing

Contributions are welcome! Please read CONTRIBUTING.md for details.
