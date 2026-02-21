# Rinux Quick Reference Card

**Version**: 0.2.0  
**Target**: x86_64  
**Status**: Development (Alpha)

---

## What Works ✅

### Memory Management
```rust
// Frame allocation
let frame = allocate_frame()?;
deallocate_frame(frame);

// Heap allocation
let vec = Vec::new();  // Works!
let string = String::from("hello");  // Works!

// Slab allocator
// Automatic for all kernel allocations
// Size classes: 8B, 16B, 32B, 64B, 128B, 256B, 512B, 1KB, 2KB, 4KB
```

### File System (TmpFS)
```rust
// TmpFS is fully functional
use rinux_drivers::fs::tmpfs::TmpFs;

let fs = TmpFs::new();
let root = fs.root();

// Create files and directories
root.create("file.txt", FileType::Regular)?;
root.create("mydir", FileType::Directory)?;

// Read/write
let file = root.lookup("file.txt")?;
file.write(b"Hello, world!")?;
let mut buf = [0u8; 13];
file.read(&mut buf)?;
```

### Console Output
```rust
// Macro form (preferred)
printk!("Hello, {}\n", "world");
printkln!("With automatic newline");

// Function form
use rinux_kernel::printk::printk;
printk("Simple string\n");
```

### Keyboard Input
```rust
use rinux_drivers::keyboard;

// Read key (blocking)
if let Some(key) = keyboard::read_key() {
    printkln!("Pressed: {}", key);
}

// Modifiers work: Shift, Ctrl, Alt
// Lock keys work: Caps Lock, Num Lock, Scroll Lock
// LEDs indicate lock state
```

### Serial Port
```rust
use rinux_drivers::serial;

// Write to serial (COM1)
serial::write_str("Debug message\n");
serial::write_byte(b'X');

// Read from serial (non-blocking)
if let Some(byte) = serial::read_byte() {
    printkln!("Received: {}", byte as char);
}
```

### Graphics
```rust
use rinux_drivers::graphics::framebuffer::{Framebuffer, Color};

let fb = Framebuffer::init()?;

// Draw primitives
fb.draw_line(10, 10, 100, 100, Color::rgb(255, 0, 0));
fb.draw_circle(200, 200, 50, Color::rgb(0, 255, 0));
fb.draw_rect(300, 300, 100, 50, Color::rgb(0, 0, 255));

// Text rendering
fb.draw_char(10, 10, 'A', Color::rgb(255, 255, 255));
fb.draw_string(10, 30, "Hello!", Color::rgb(255, 255, 255));
```

### Process Information
```rust
use rinux_kernel::process;

// Get current PID
let pid = process::current_pid();

// Yield CPU
process::yield_now();
```

### Timer
```rust
use rinux_drivers::timer;

// Get system uptime
let ticks = timer::get_ticks();
let ms = timer::get_uptime_ms();
let secs = timer::get_uptime_secs();
```

### Real-Time Clock
```rust
use rinux_drivers::rtc;

let datetime = rtc::read_datetime();
printkln!("Date: {}-{}-{}", datetime.year, datetime.month, datetime.day);
printkln!("Time: {}:{}:{}", datetime.hour, datetime.minute, datetime.second);
```

---

## What Doesn't Work Yet ❌

### Process Management
```rust
// These exist but are NOT functional:
fork();      // Framework only
exec();      // Framework only
wait();      // Framework only
// Context switching not implemented
```

### Block Storage
```rust
// AHCI and NVMe drivers detect devices but:
// - No DMA operations
// - No interrupt-driven I/O
// - Cannot read/write blocks
```

### Filesystems
```rust
// ext2/ext4 have structures but:
// - No block device I/O
// - Cannot mount real disks
// - Only TmpFS is functional
```

### Networking
```rust
// Not implemented at all:
// - No network drivers
// - No TCP/IP stack
// - No sockets
```

---

## Architecture Features

### x86_64 Support ✅
- GDT, IDT, TSS configured
- All CPU exceptions handled
- PIC and APIC interrupt controllers
- Page tables (4-level paging)
- System calls (syscall/sysret)
- FPU/SSE state management

### Multi-core (SMP) ⚠️
- Bootstrap processor works
- Framework exists for APs
- Scheduling not SMP-aware yet

---

## Building and Testing

### Build Commands
```bash
make build      # Build kernel
make run        # Run in QEMU
make test       # Run unit tests
make clean      # Clean artifacts
make clippy     # Lint code
make fmt        # Format code
```

### Development
```bash
# Build for development
cargo +nightly build

# Build for release
cargo +nightly build --release

# Run tests (lib only)
cd lib && cargo +nightly test
```

---

## Memory Layout

```
Virtual Memory (x86_64):
┌────────────────────┐ 0xFFFFFFFFFFFFFFFF
│ Kernel (higher)    │
├────────────────────┤ 0xFFFF800000000000
│ (Canonical hole)   │
├────────────────────┤ 0x00007FFFFFFFFFFF
│ User space         │
└────────────────────┘ 0x0000000000000000

Kernel Heap:
- Default: 1 MB
- Allocator: Slab allocator
- Growth: Not yet implemented
```

---

## Key Data Structures

### Task (Process)
```rust
pub struct Task {
    pub pid: u32,
    pub state: TaskState,      // Running, Sleeping, Zombie
    pub priority: u8,          // 0-255
    pub parent_pid: Option<u32>,
    pub exit_code: i32,
}
```

### VNode (File)
```rust
pub struct VNode {
    pub vtype: VNodeType,      // Regular, Directory, Symlink
    pub mode: u16,             // Permissions
    pub size: usize,
    // + operations table
}
```

### PhysAddr, VirtAddr
```rust
#[repr(transparent)]
pub struct PhysAddr(pub u64);

#[repr(transparent)]
pub struct VirtAddr(pub u64);
```

---

## Error Handling

### Error Codes
- Uses result types: `Result<T, ErrorCode>`
- POSIX-compatible error codes
- No panics in normal operation (only for kernel bugs)

### Common Errors
- `ENOMEM` - Out of memory
- `ENOENT` - File not found
- `EBADF` - Bad file descriptor
- `EINVAL` - Invalid argument
- `ENOSYS` - Function not implemented

---

## Performance Notes

### Fast Operations (O(1))
- Frame allocation/deallocation
- Slab allocation (for cached sizes)
- TmpFS file operations (mostly)
- Page table lookups (with TLB)

### Slow Operations
- PID allocation (O(n) bitmap scan)
- Path resolution (O(n) components)
- Directory listing (O(n) entries)

---

## Debugging

### Serial Output
- COM1 at 0x3F8
- 38400 baud by default
- Use for kernel debugging

### VGA Output
- 80x25 text mode
- 16 colors
- Main console

### QEMU Debugging
```bash
# Run with serial output
qemu-system-x86_64 -kernel rinux -serial stdio

# Run with GDB
qemu-system-x86_64 -kernel rinux -s -S
# Then: gdb rinux, (gdb) target remote :1234
```

---

## Security Status ⚠️

### Current State
- ✅ Memory safe (Rust ownership)
- ✅ No buffer overflows in safe code
- ✅ CodeQL scan passes
- ❌ No user authentication
- ❌ No access control enforcement
- ❌ No privilege separation
- ❌ No encryption

**Warning**: Educational kernel only. Do not use in production.

---

## Resource Limits

### Current Limits
- Max frames: 8192 (32 MB physical RAM tracked)
- Max PIDs: 65536
- Max open files per process: Not enforced
- Max tasks: Not enforced
- Kernel heap: 1 MB default

---

## Useful Links

- **Documentation**: `docs/`
- **Features**: `FEATURES.md`
- **Changelog**: `CHANGELOG.md`
- **Release Notes**: `RELEASE_NOTES_v0.2.0.md`
- **GitHub**: https://github.com/npequeux/rinux
- **Issues**: https://github.com/npequeux/rinux/issues

---

## Getting Help

### Common Issues

**Build fails**: Ensure nightly Rust installed
```bash
rustup install nightly
rustup component add --toolchain nightly rust-src llvm-tools-preview
```

**QEMU won't start**: Install QEMU
```bash
# Ubuntu/Debian
sudo apt-get install qemu-system-x86

# macOS
brew install qemu
```

**Tests fail**: Only lib tests work currently
```bash
cd lib
cargo +nightly test --target x86_64-unknown-linux-gnu
```

---

**Last Updated**: v0.2.0 (February 21, 2026)  
**License**: MIT  
**Maintainer**: Nicolas Péqueux (@npequeux)
