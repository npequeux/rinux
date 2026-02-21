# Quick Reference: New Features

## Testing Fork/Exec Chain

### Running Tests

```rust
// In kernel initialization
use rinux_kernel::tests;

// Run all tests
tests::run_all();

// Run specific test suite
tests::process_tests::run();
```

### Expected Output

```
=== Running Kernel Tests ===

--- Process Tests ---
  Testing Fork basic functionality: PASS
  Testing Fork PID allocation: PASS
  Testing Fork memory context: PASS
  Testing ELF header parsing: PASS
  Testing Exec context setup: PASS

Process Tests: 5 passed, 0 failed

=== All Tests Complete ===
```

## Mounting Filesystems

### Mount Root Filesystem

```rust
use rinux_drivers::fs::{ext2::Ext2Filesystem, mount};
use alloc::sync::Arc;

// Create ext2 filesystem (would normally pass block device)
let fs = Ext2Filesystem::mount()?;

// Set as root and mount at "/"
mount::set_root(Arc::new(fs))?;

// Now can access files through VFS
let root = mount::get_root_vnode().expect("No root fs");
```

### Mount Additional Filesystems

```rust
use rinux_drivers::fs::mount::{mount, MountFlags};

// Read-write mount
mount("/home", home_fs, MountFlags::new())?;

// Read-only mount
mount("/etc", config_fs, MountFlags::readonly())?;

// Custom flags
let flags = MountFlags {
    readonly: false,
    noexec: false,
    nodev: true,
    nosuid: true,
};
mount("/tmp", tmpfs, flags)?;
```

### Query Mount Points

```rust
// Get filesystem for a path
if let Some(fs) = mount::get_mount("/home/user/file.txt") {
    // Use filesystem
    let root = fs.root();
}

// List all mounts
for (path, fs_type) in mount::list_mounts() {
    printkln!("{} -> {}", path, fs_type);
}
```

### Unmounting

```rust
// Unmount a filesystem
mount::unmount("/tmp")?;
```

## Interrupt-Driven I/O

### Initialization

```rust
use rinux_drivers::block::ahci_irq;

// Initialize interrupt-driven I/O (call once during boot)
ahci_irq::init();
```

### AHCI Port Setup

```rust
// Enable interrupts for a port
let port_regs = ahci_device.get_port_registers();
ahci_irq::enable_port_interrupts(port_regs as *mut u8, port_number);
```

### Issuing I/O Operations

```rust
use rinux_drivers::block::ahci_irq::{add_pending_io, wait_for_completion};

// Create I/O completion tracker
let completion = add_pending_io(port, slot);

// Issue command to hardware (AHCI command issue)
issue_ahci_command(port, slot);

// Wait for completion (with 5 second timeout)
match wait_for_completion(&completion, 5000) {
    Ok(status) => {
        // I/O completed successfully
        printkln!("I/O complete with status: {}", status);
    }
    Err(_) => {
        // Timeout occurred
        printkln!("I/O timeout");
    }
}
```

### Interrupt Handler

The AHCI interrupt handler is automatically registered:

```rust
// Automatically called when IRQ 11 fires
fn ahci_interrupt_handler(irq: u8) {
    // Reads AHCI interrupt status
    // Marks pending I/O as complete
    // Clears interrupt
}
```

### Custom Interrupt Handlers

```rust
use rinux_drivers::block::ahci_irq::register_irq_handler;

// Register custom handler
fn my_irq_handler(_irq: u8) {
    printkln!("Custom IRQ handler called");
}

register_irq_handler(11, my_irq_handler);
```

## Integration Example

### Complete Boot Sequence

```rust
// 1. Initialize subsystems
rinux_kernel::init();

// 2. Initialize interrupt-driven I/O
rinux_drivers::block::ahci_irq::init();

// 3. Mount root filesystem
let ext2_fs = rinux_drivers::fs::ext2::Ext2Filesystem::mount()?;
rinux_drivers::fs::mount::set_root(Arc::new(ext2_fs))?;

// 4. Run tests
rinux_kernel::tests::run_all();

// 5. Start init process
// (would exec /sbin/init here)

// 6. Enter main loop
loop {
    // Schedule tasks
    // Handle interrupts
    // Process I/O
}
```

## Debugging

### Enable Verbose Logging

```rust
// In mount.rs
printkln!("Mounting {} at {}", fs_type, path);

// In ahci_irq.rs
printkln!("I/O completion: port={}, slot={}, status={}", 
          port, slot, status);

// In tests
printkln!("Test '{}': {}", name, result);
```

### Check Mount Table

```rust
let mounts = rinux_drivers::fs::mount::list_mounts();
printkln!("Active mounts: {}", mounts.len());
for (path, fs_type) in mounts {
    printkln!("  {} -> {}", path, fs_type);
}
```

### Monitor I/O Operations

```rust
// Check if I/O completed
{
    let comp = completion.lock();
    if comp.completed {
        printkln!("I/O finished: status={}", comp.status);
    } else {
        printkln!("I/O still pending");
    }
}
```

## Common Issues

### Mounting Fails

```rust
// Check if root already set
if mount::get_root().is_some() {
    printkln!("Root already mounted");
}

// Verify filesystem is valid
match Ext2Filesystem::mount() {
    Ok(fs) => printkln!("Filesystem OK"),
    Err(e) => printkln!("Mount failed: {:?}", e),
}
```

### I/O Timeout

```rust
// Increase timeout for slow devices
wait_for_completion(&completion, 10000)?; // 10 seconds

// Check if interrupts are enabled
ahci_irq::enable_port_interrupts(port_regs, port);

// Ensure IRQ is not masked in PIC
rinux_arch_x86::interrupts::enable_irq(11);
```

### Test Failures

```rust
// Run tests individually
use rinux_kernel::tests::process_tests::*;

test_fork_basic();
test_fork_pid_allocation();
test_fork_memory_context();
test_elf_parsing();
test_exec_context();
```

## Performance Tips

1. **Batch I/O Operations**: Queue multiple operations before waiting
2. **Use Async Where Possible**: Don't block on I/O completion
3. **Cache Mount Lookups**: Store frequent mount points
4. **Minimize Interrupts**: Use polling for short operations (<1ms)

## Security Considerations

1. **Mount Flags**: Use `readonly`, `noexec`, `nosuid` appropriately
2. **Path Validation**: Always validate mount paths
3. **Interrupt Handling**: Keep handlers fast and non-blocking
4. **Test Coverage**: Run tests regularly to catch regressions
