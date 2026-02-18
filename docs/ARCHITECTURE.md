# Rinux Architecture Guide

## Overview

Rinux follows a modular kernel architecture similar to Linux, with the following main components:

## Core Components

### 1. Architecture Support (`arch/`)

Architecture-specific code for different CPU architectures:

- **x86/**: x86_64 support
  - Boot process
  - CPU management (CPUID, MSR)
  - GDT/IDT setup
  - Interrupt handling
  - Memory management (paging)
  - I/O port access

Future architectures can be added here (ARM, RISC-V, etc.)

### 2. Kernel Core (`kernel/`)

Core kernel functionality:

- **init**: Kernel initialization
- **printk**: Kernel logging
- **panic**: Panic handling
- **types**: Common type definitions
- **process**: Process management (future)

### 3. Memory Management (`mm/`)

Physical and virtual memory management:

- **allocator**: Global heap allocator
- **frame**: Physical frame allocator
- **heap**: Kernel heap management
- **vmalloc**: Virtual memory allocator

### 4. Device Drivers (`drivers/`)

Device driver framework:

- **serial**: Serial port driver
- **keyboard**: Keyboard driver
- **vga**: VGA text mode driver

### 5. Library (`lib/`)

Common utilities and data structures:

- **math**: Mathematical utilities
- **string**: String operations
- **list**: Linked list implementation

## Boot Process

1. Bootloader loads kernel (Multiboot)
2. `_start` function in `main.rs`
3. Early architecture initialization
4. Memory management setup
5. Kernel subsystem initialization
6. Enter main kernel loop

## Memory Layout

```
0x0000000000000000 - 0x00007FFFFFFFFFFF: User space
0xFFFF800000000000 - 0xFFFFFF7FFFFFFFFF: Direct physical mapping
0xFFFFFF8000000000 - 0xFFFFFFFFFFFFFFFF: Kernel space
```

## Interrupt Handling

Interrupts are handled through the IDT (Interrupt Descriptor Table):

- Exceptions (0-31): CPU-generated exceptions
- IRQs (32-255): Hardware interrupts

## Building

The build process uses:

1. Cargo for Rust compilation
2. Custom target specification (`x86_64-unknown-rinux.json`)
3. Linker script (`linker.ld`)
4. Makefile for convenience

## Extension

To add support for a new architecture:

1. Create `arch/<arch>/` directory
2. Implement required architecture-specific functions
3. Update build system
4. Add architecture-specific configuration
