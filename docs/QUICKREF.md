# Rinux Quick Reference

## Build Commands

```bash
# Build the kernel
make build

# Run in QEMU
make run

# Debug in QEMU (starts paused, waiting for GDB)
make debug

# Run tests
make test

# Format code
make fmt

# Run Clippy linter
make clippy

# Generate documentation
make doc

# Clean build artifacts
make clean

# Show help
make help
```

## Project Structure

```
rinux/
├── arch/x86/          # x86_64 architecture code
├── kernel/            # Core kernel functionality
├── mm/                # Memory management
├── drivers/           # Device drivers
├── lib/               # Kernel libraries
├── src/               # Main entry point
└── docs/              # Documentation
```

## Key Files

- `src/main.rs` - Kernel entry point (`_start`)
- `linker.ld` - Linker script
- `Makefile` - Build automation
- `x86_64-unknown-rinux.json` - Target specification

## Important Functions

### Kernel Entry
```rust
// src/main.rs
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Kernel initialization
}
```

### Print to Console
```rust
use rinux_kernel::printk::printk;

printk("Hello, Rinux!\n");
```

### Halt CPU
```rust
use rinux_arch_x86::halt;

halt(); // Infinite loop with HLT
```

### Memory Allocation
```rust
// Uses global allocator
let vec = alloc::vec![1, 2, 3];
```

## Architecture APIs

### x86_64 (`rinux_arch_x86`)

```rust
// CPU operations
x86::halt()                    // Halt CPU
x86::enable_interrupts()       // STI
x86::disable_interrupts()      // CLI
x86::interrupts_enabled()      // Check IF flag

// I/O ports
x86::io::inb(port)             // Read byte
x86::io::outb(port, value)     // Write byte
x86::io::inw(port)             // Read word
x86::io::outw(port, value)     // Write word
x86::io::inl(port)             // Read dword
x86::io::outl(port, value)     // Write dword

// MSR access
x86::cpu::rdmsr(msr)           // Read MSR
x86::cpu::wrmsr(msr, value)    // Write MSR

// Paging
x86::paging::read_cr3()        // Get page table
x86::paging::write_cr3(addr)   // Set page table
x86::paging::flush_tlb(addr)   // Flush TLB entry
x86::paging::flush_tlb_all()   // Flush all TLB
```

## Memory Management APIs

### Frame Allocator (`rinux_mm::frame`)

```rust
use rinux_mm::frame;

// Allocate physical frame
let frame = frame::allocate_frame()?;

// Deallocate frame
frame::deallocate_frame(frame);

// Frame address
let addr = frame.start_address();
```

### Heap Allocator

```rust
// Automatic through global allocator
use alloc::vec::Vec;
use alloc::boxed::Box;

let v = Vec::new();
let b = Box::new(42);
```

## Kernel APIs

### Initialization (`rinux_kernel::init`)

```rust
use rinux_kernel::init;

init::early_init();  // Early initialization
init::main_init();   // Main initialization
init::late_init();   // Late initialization
```

### Types (`rinux_kernel::types`)

```rust
use rinux_kernel::types::{PhysAddr, VirtAddr, Pid};

let phys = PhysAddr::new(0x1000);
let virt = VirtAddr::new(0xFFFF800000001000);
let pid: Pid = 1;
```

## Common Patterns

### Spinlock Usage

```rust
use spin::Mutex;

static DATA: Mutex<u32> = Mutex::new(0);

let mut data = DATA.lock();
*data += 1;
```

### Interrupt Handler

```rust
extern "x86-interrupt" fn handler() {
    // Handle interrupt
    x86::interrupts::send_eoi(irq);
}
```

### Panic

```rust
panic!("Something went wrong: {}", error);
```

## Debugging

### Print Debugging

```rust
printk("Value: {}\n", value);  // TODO: Format not yet implemented
printk("Debug point reached\n");
```

### GDB Debugging

```bash
# Terminal 1
make debug

# Terminal 2
gdb -ex "target remote :1234"
```

## Configuration

Edit `Kconfig` for kernel configuration options.

## Documentation

```bash
# View architecture docs
cargo doc --package rinux-arch-x86 --open

# View kernel docs
cargo doc --package rinux-kernel --open

# View all docs
make doc
```

## Testing

```bash
# Run unit tests
make test

# Run in QEMU
make run
```

## Code Style

```bash
# Auto-format
make fmt

# Check with Clippy
make clippy
```

## Common Issues

### Build fails
```bash
# Update Rust nightly
rustup update nightly

# Ensure components installed
rustup component add rust-src
rustup component add llvm-tools-preview
```

### QEMU not found
```bash
# Install QEMU
sudo apt install qemu-system-x86  # Ubuntu
```

### Linker errors
```bash
# Check linker script
cat linker.ld

# Verify target spec
cat x86_64-unknown-rinux.json
```

## Resources

- [Architecture Guide](ARCHITECTURE.md)
- [Build Guide](BUILD.md)
- [Development Guide](DEVELOPMENT.md)
- [Roadmap](ROADMAP.md)
- [Summary](SUMMARY.md)
