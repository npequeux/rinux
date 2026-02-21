# Rinux Changelog

All notable changes to the Rinux operating system kernel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-21

### Added

#### Memory Management
- Slab allocator with multiple size classes (8B to 4KB) for efficient kernel memory allocation
- Advanced page fault handler with error code parsing and on-demand allocation
- VMalloc support for virtual memory allocation in kernel space
- Memory mapping (mmap) infrastructure
- Out-of-memory (OOM) killer framework
- Swap support framework

#### Process Management
- Task structure with process state management (Running, Sleeping, Zombie)
- Round-robin scheduler implementation
- CFS-inspired scheduler framework for future fair scheduling
- PID allocator and management
- Fork, exec, and wait system call infrastructure (framework)

#### Filesystem Support
- VFS (Virtual File System) layer with POSIX-compliant VNode abstraction
- Fully functional TmpFS (temporary filesystem) for in-memory file storage
- File descriptor table management
- ext2 and ext4 filesystem frameworks (structure definitions)
- ProcFS and SysFS skeleton implementations

#### Device Drivers

**Serial/UART:**
- Multi-port support (COM1-COM4)
- Configurable baud rates (115200, 57600, 38400, 19200, 9600, 4800, 2400)
- Configurable data bits, stop bits, and parity
- Flow control status checking (DCD, DSR, CTS, RI)
- Non-blocking read operations

**Keyboard:**
- Complete PS/2 keyboard driver with full scancode mapping
- Modifier key support (Shift, Ctrl, Alt)
- Lock key toggles (Caps Lock, Num Lock, Scroll Lock)
- LED control for visual feedback
- Special character and symbol support

**Graphics:**
- Framebuffer driver with drawing primitives (lines, circles, rectangles)
- Text rendering with 8x8 bitmap font
- GPU detection and initialization for Intel i915, AMD (RDNA/GCN), and NVIDIA (Maxwell+)
- MMIO register access framework
- PCI bus mastering and memory space enablement

**Storage:**
- Block device abstraction layer
- AHCI (SATA) driver framework
- NVMe driver framework for modern SSDs
- Request queue management
- GPT and MBR partition table parsing

**Other Drivers:**
- Timer/RTC support
- PCI bus enumeration and device scanning
- ACPI framework
- USB xHCI host controller framework
- USB HID and mass storage frameworks

#### Architecture (x86_64)
- Enhanced interrupt handling with APIC support
- Exception handlers for all x86_64 exceptions
- FPU/SSE state management
- Context switching infrastructure
- Multi-core SMP bootstrap framework
- System call infrastructure (syscall/sysret)
- TSC and HPET timer support

#### System Features
- Inter-process communication (IPC) framework (pipes, shared memory)
- Signal handling infrastructure
- System call dispatcher (framework with basic syscalls)
- Kernel logging via printk with VGA output

#### Documentation
- Comprehensive kernel gap analysis
- Implementation summaries and roadmaps
- Hardware support documentation
- Driver development guide
- Test coverage reports
- Linux kernel comparison documentation

### Changed
- Improved VGA text mode driver for better output
- Enhanced memory allocator performance
- Updated build system for better compilation

### Fixed
- Various memory management edge cases
- Interrupt handling stability improvements

### Security
- All code passes CodeQL security scanning with 0 vulnerabilities
- Proper unsafe code documentation
- Memory safety maintained throughout

### Testing
- 61 unit tests with 100% pass rate
- Tests for memory allocation, string operations, and list structures
- Driver-specific unit tests

## [0.1.0] - 2026-02-18

### Added
- Initial project structure
- Basic x86_64 architecture support
- Basic memory management (physical and virtual)
- Console output via VGA text mode
- GDT and IDT setup
- Basic interrupt handling framework
- Device driver framework
- Build system with Cargo and Make

---

## Release Notes

### Version 0.2.0 Status

**What Works:**
- ✅ Memory allocation (slab allocator, frame allocator)
- ✅ VGA and serial console output
- ✅ Keyboard input with full modifier support
- ✅ Basic graphics framebuffer with drawing primitives
- ✅ TmpFS filesystem (fully functional)
- ✅ PCI device enumeration
- ✅ GPU detection (Intel/AMD/NVIDIA)
- ✅ Page table management
- ✅ Interrupt and exception handling

**Partially Implemented (Framework Only):**
- ⚠️ Process management (structures exist, fork/exec not functional)
- ⚠️ Storage drivers (detection works, DMA operations not implemented)
- ⚠️ ext2/ext4 filesystems (structures only, no block I/O)
- ⚠️ USB support (xHCI framework only)
- ⚠️ System calls (infrastructure exists, most calls are stubs)

**Not Yet Implemented:**
- ❌ Task preemption and context switching
- ❌ Network stack (TCP/IP)
- ❌ File I/O to disk
- ❌ Interrupt-driven device I/O (currently polling)

**Progress:** Approximately 15% of Linux kernel functionality implemented

For detailed information, see:
- `IMPLEMENTATION_COMPLETE.md` - Major features overview
- `HARDWARE_ENHANCEMENTS_SUMMARY.md` - Recent driver improvements
- `docs/KERNEL_GAPS_ANALYSIS.md` - Comparison with Linux kernel
- `docs/ROADMAP.md` - Future development plans
