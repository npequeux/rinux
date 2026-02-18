# Building Rinux

## Prerequisites

### Required Tools

- **Rust nightly**: Install via rustup
  ```bash
  rustup install nightly
  rustup default nightly
  ```

- **QEMU**: For testing
  ```bash
  # Ubuntu/Debian
  sudo apt install qemu-system-x86

  # Fedora
  sudo dnf install qemu-system-x86

  # macOS
  brew install qemu
  ```

- **Build tools**:
  ```bash
  # Ubuntu/Debian
  sudo apt install build-essential

  # Fedora
  sudo dnf install gcc make

  # macOS
  xcode-select --install
  ```

### Rust Components

```bash
rustup component add rust-src
rustup component add llvm-tools-preview
```

## Building

### Build the kernel

```bash
make build
```

This will:
1. Compile all Rust code
2. Link with the linker script
3. Generate the kernel binary

### Run in QEMU

```bash
make run
```

### Debug in QEMU

```bash
make debug
```

Then in another terminal:
```bash
gdb -ex "target remote :1234" -ex "symbol-file target/x86_64-unknown-rinux/release/rinux"
```

## Build Options

### Architecture

```bash
make ARCH=x86_64 build
```

### Build Type

```bash
# Release build (default)
make build

# Debug build
cargo +nightly build --target x86_64-unknown-rinux.json
```

## Testing

```bash
make test
```

## Code Quality

### Format code

```bash
make fmt
```

### Run linter

```bash
make clippy
```

## Clean

```bash
make clean
```

## Documentation

Generate and view documentation:

```bash
make doc
```

## Troubleshooting

### Build fails with "linker not found"

Install LLVM tools:
```bash
rustup component add llvm-tools-preview
```

### Rust nightly issues

Update nightly:
```bash
rustup update nightly
```

### QEMU not found

Ensure QEMU is installed and in PATH:
```bash
which qemu-system-x86_64
```
