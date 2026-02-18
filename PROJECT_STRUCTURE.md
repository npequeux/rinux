# Rinux Project Structure

```
rinux/
│
├── src/                          # Main kernel entry
│   ├── main.rs                   # Kernel entry point (_start)
│   └── lib.rs                    # Main kernel library
│
├── arch/                         # Architecture-specific code
│   └── x86/                      # x86_64 support
│       ├── src/
│       │   ├── lib.rs            # Architecture module
│       │   ├── boot.rs           # Boot code & Multiboot header
│       │   ├── cpu.rs            # CPU detection, CPUID, MSR
│       │   ├── gdt.rs            # Global Descriptor Table
│       │   ├── idt.rs            # Interrupt Descriptor Table
│       │   ├── interrupts.rs     # Interrupt handling, PIC
│       │   ├── io.rs             # Port I/O operations
│       │   ├── memory.rs         # Memory detection
│       │   └── paging.rs         # Page table management
│       └── Cargo.toml
│
├── kernel/                       # Core kernel functionality
│   ├── src/
│   │   ├── lib.rs                # Kernel module
│   │   ├── init.rs               # Initialization routines
│   │   ├── printk.rs             # Kernel logging (VGA)
│   │   ├── panic.rs              # Panic handler
│   │   ├── types.rs              # Common types (PhysAddr, VirtAddr, Pid)
│   │   ├── process.rs            # Process management module
│   │   └── process/
│   │       ├── sched.rs          # Scheduler
│   │       ├── task.rs           # Task structure
│   │       └── pid.rs            # PID allocation
│   └── Cargo.toml
│
├── mm/                           # Memory management
│   ├── src/
│   │   ├── lib.rs                # MM module
│   │   ├── allocator.rs          # Global heap allocator
│   │   ├── frame.rs              # Physical frame allocator
│   │   ├── heap.rs               # Heap management
│   │   └── vmalloc.rs            # Virtual memory allocator
│   └── Cargo.toml
│
├── drivers/                      # Device drivers
│   ├── src/
│   │   ├── lib.rs                # Driver framework
│   │   ├── serial.rs             # Serial port driver
│   │   ├── keyboard.rs           # Keyboard driver
│   │   └── vga.rs                # VGA text mode
│   └── Cargo.toml
│
├── lib/                          # Kernel library
│   ├── src/
│   │   ├── lib.rs                # Lib module
│   │   ├── math.rs               # Math utilities
│   │   ├── string.rs             # String utilities
│   │   └── list.rs               # Linked list
│   └── Cargo.toml
│
├── docs/                         # Documentation
│   ├── ARCHITECTURE.md           # System architecture
│   ├── BUILD.md                  # Build instructions
│   ├── DEVELOPMENT.md            # Development guide
│   ├── ROADMAP.md                # Future plans
│   ├── SUMMARY.md                # Project summary
│   └── QUICKREF.md               # Quick reference
│
├── .cargo/
│   └── config.toml               # Cargo configuration
│
├── Cargo.toml                    # Workspace manifest
├── build.rs                      # Build script
├── Makefile                      # Build automation
├── linker.ld                     # Linker script
├── x86_64-unknown-rinux.json     # Custom target spec
├── Kconfig                       # Kernel configuration
├── .gitignore                    # Git ignore rules
│
├── README.md                     # Project overview
├── CHANGELOG.md                  # Version history
├── CONTRIBUTING.md               # Contribution guide
└── LICENSE                       # GPL-2.0 license
```

## Statistics

- **Total Files**: 54
- **Rust Source Files**: ~43
- **Configuration Files**: 7
- **Documentation Files**: 8
- **Crates**: 6 (workspace + 5 sub-crates)
- **Lines of Code**: ~2,500+ (Rust)

## Crate Dependency Graph

```
rinux (main)
├── rinux-arch-x86
├── rinux-kernel
├── rinux-mm
├── rinux-drivers
└── rinux-lib
```

## Key Components by Lines of Code (Approximate)

| Component | Lines | Description |
|-----------|-------|-------------|
| arch/x86 | ~800 | Architecture support |
| kernel | ~600 | Core kernel |
| mm | ~300 | Memory management |
| drivers | ~100 | Device drivers |
| lib | ~200 | Utilities |
| src | ~100 | Entry point |
| **Total** | **~2,500** | |

## Build Artifacts

When built, produces:
- `target/x86_64-unknown-rinux/release/rinux` - Kernel binary
- Compatible with QEMU and Multiboot bootloaders
