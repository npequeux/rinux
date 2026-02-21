# Quick Start Guide: Next Steps for Bootability

This document provides immediate next steps to achieve bootability on real hardware.

## Current Status Summary

✅ **Implemented:**
- Process management (fork, exec)
- Context switching
- System call infrastructure  
- AHCI storage driver with DMA
- ext2 filesystem
- PS/2 keyboard driver

⚠️ **Needs Work:**
- Build system (custom target issues)
- Page fault handling
- Interrupt routing
- Hardware testing

## Immediate Actions (This Week)

### 1. Fix Build Issues

**Problem:** Custom x86_64-unknown-rinux target compilation errors

**Solution:**
```bash
# Build Rust core library for custom target
cd /home/npequeux/code/Rinux/rinux
rustup component add rust-src --toolchain nightly

# Use cargo build with build-std
cargo build -Z build-std=core,alloc --target x86_64-unknown-rinux.json
```

**Files to check:**
- [Cargo.toml](../Cargo.toml) - Verify build-std configuration
- [x86_64-unknown-rinux.json](../x86_64-unknown-rinux.json) - Target spec

### 2. Complete Memory Management

**Priority:** CRITICAL - Required for process execution

**Add to mm/src/page_fault.rs:**
```rust
#[no_mangle]
pub extern "C" fn page_fault_handler(
    error_code: u64,
    fault_addr: u64,
) {
    // 1. Check if fault is in user space
    // 2. Check error code (present, write, user, etc.)
    // 3. Allocate page if needed
    // 4. Update page tables
    // 5. Return or kill process if invalid
}
```

**Enable paging in kernel initialization:**
```rust
// In arch/x86/src/paging.rs
pub fn enable_paging() {
    unsafe {
        // Set CR3 to page table
        // Set CR0.PG bit
        // Enable PAE if needed
    }
}
```

### 3. Test Process Creation

**Create test program:**
```rust
// In kernel/src/main.rs or test module
#[no_mangle]
pub fn test_fork() {
    match process::fork::do_fork() {
        Ok(child_pid) => {
            printk!("Child process created: PID {}\n", child_pid);
        }
        Err(e) => {
            printk!("Fork failed: {}\n", e);
        }
    }
}
```

### 4. Initialize AHCI in QEMU

**Test storage driver:**
```bash
# Run with AHCI disk
make run QEMU_EXTRA_ARGS="-drive file=test.img,if=none,id=disk0 \
  -device ahci,id=ahci -device ide-hd,drive=disk0,bus=ahci.0"
```

**Add initialization code in drivers/block/src/ahci.rs:**
```rust
pub fn init() {
    // Scan PCI for AHCI controllers
    if let Some(ahci_base) = scan_pci_for_ahci() {
        unsafe {
            let mut controller = AhciController::new(ahci_base);
            controller.init().expect("AHCI init failed");
            controller.probe_devices();
            
            // Register devices
            for device in controller.devices.iter() {
                register_device(device.clone());
            }
        }
    }
}
```

### 5. Mount Root Filesystem

**Add mount capability:**
```rust
// In kernel/src/fs.rs
pub fn mount_root(device: &str, fstype: &str) -> Result<(), &'static str> {
    match fstype {
        "ext2" => {
            let dev = get_block_device(device)?;
            let fs = ext2::Ext2Filesystem::mount(dev)?;
            vfs::set_root(fs.root());
            Ok(())
        }
        _ => Err("Unsupported filesystem")
    }
}
```

## Week 1-2 Goals

- [ ] Kernel builds successfully
- [ ] Can boot in QEMU
- [ ] Page faults handled correctly
- [ ] Fork creates child process
- [ ] AHCI detects disk
- [ ] Can mount ext2 filesystem

## Testing Checklist

### Boot Test
```bash
make run
# Expected: Kernel loads, prints messages, doesn't crash
```

### Process Test
```bash
# In kernel
test_fork()
# Expected: Child PID printed, no panic
```

### Storage Test
```bash
# Create test disk with ext2
dd if=/dev/zero of=test.img bs=1M count=64
mkfs.ext2 test.img

# Boot with disk
make run DISK=test.img
# Expected: Device detected, filesystem mounted
```

### Syscall Test
```rust
// User-space test (once init works)
fn main() {
    let pid = unsafe { syscall(SYS_GETPID) };
    println!("My PID: {}", pid);
    
    if unsafe { syscall(SYS_FORK) } == 0 {
        println!("I'm the child!");
    } else {
        println!("I'm the parent!");
    }
}
```

## Debug Tips

### Enable Debug Output
```rust
// In kernel/src/printk.rs
pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as usize, Ordering::SeqCst);
}

// In main
set_log_level(LogLevel::Debug);
```

### QEMU Debugging
```bash
# Run with GDB server
make run QEMU_EXTRA_ARGS="-s -S"

# In another terminal
gdb target/x86_64-unknown-rinux/debug/rinux
(gdb) target remote :1234
(gdb) break syscall_handler
(gdb) continue
```

### Serial Console
```bash
# Add to QEMU args
-serial stdio

# In kernel, use serial output
serial!("Debug: fork called\n");
```

## Common Issues & Solutions

### Issue: "error: can't find crate for `core`"
**Solution:** Use `cargo build -Z build-std=core,alloc`

### Issue: Triple fault on boot
**Solution:** Check GDT, IDT, and page tables are set up correctly

### Issue: Page fault immediately after enabling paging
**Solution:** Ensure kernel is identity-mapped or high-half mapped

### Issue: Syscall causes General Protection Fault
**Solution:** Verify MSR configuration (STAR, LSTAR) and segment selectors

### Issue: AHCI not detecting disk
**Solution:** Check PCI enumeration, HBA address, and QEMU disk configuration

## Resources

### Documentation
- [BOOTABILITY_IMPLEMENTATION.md](BOOTABILITY_IMPLEMENTATION.md) - Full implementation details
- [BOOTABILITY_ROADMAP.md](BOOTABILITY_ROADMAP.md) - Long-term plan
- [KERNEL_GAPS_ANALYSIS.md](KERNEL_GAPS_ANALYSIS.md) - Feature gaps

### Specifications
- Intel Software Developer Manual (for x86_64)
- AHCI Specification 1.3.1
- ext2 Filesystem Documentation
- Linux syscall interface documentation

### Example Code
- Linux kernel source (for reference)
- OSDev Wiki (https://wiki.osdev.org)
- Redox OS (Rust OS example)

## Getting Help

### Check Logs
```bash
# Kernel logs
dmesg

# QEMU logs
make run 2>&1 | tee boot.log
```

### Enable Verbose Output
```rust
// In kernel bootstrap
printk::set_log_level(LogLevel::Trace);
```

### Serial Debugging
- Use QEMU's `-serial stdio` for serial output
- Add `serial!()` macro calls at key points
- Monitor for crashes, hangs, or unexpected behavior

## Success Indicators

You'll know you're making progress when:
1. ✅ Kernel boots without panic
2. ✅ Console shows initialization messages
3. ✅ Keyboard input is received
4. ✅ Disk is detected and accessed
5. ✅ First process is created
6. ✅ System is stable for >1 minute

## Next Phase Preview

Once the immediate actions are complete:
- **Week 3-4:** Implement wait() and process cleanup
- **Month 2:** Add preemptive scheduling with timer
- **Month 3:** Create simple init process
- **Month 3-4:** Build basic shell and utilities
- **Month 4-6:** Test on real hardware

---

**Last Updated:** February 21, 2026  
**Next Review:** Weekly until Month 3, then monthly
