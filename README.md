# Rinux

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

Rinux is licensed under GPL-2.0, similar to Linux.

## Contributing

Contributions are welcome! Please read CONTRIBUTING.md for details.
