# GitHub Copilot Instructions for Rinux

## Project Overview

Rinux is a modern operating system kernel written in Rust, inspired by Linux's architecture and design principles. The project leverages Rust's safety guarantees to build a robust, secure, and performant kernel for x86_64 architecture.

## Project Structure

```
rinux/
├── arch/x86/           # x86_64 architecture-specific code
├── kernel/             # Core kernel functionality
├── mm/                 # Memory management subsystem
├── drivers/            # Device drivers
├── lib/                # Kernel utility libraries
└── src/                # Main kernel entry point
```

## Coding Standards

### Language and Style

- **Language**: Rust (nightly toolchain required)
- **Edition**: Rust 2021
- **Formatting**: Always use `cargo fmt` (standard Rust formatting)
- **Linting**: Address all `cargo clippy` warnings before committing
- **No Standard Library**: This is a `#![no_std]` environment

### Documentation

- Document all public items with doc comments (`///`)
- Include module-level documentation (`//!`)
- Provide examples in doc comments where appropriate
- For unsafe code, always include a `# Safety` section explaining invariants
- Keep documentation concise but complete

Example:
```rust
//! Module description
//!
//! More detailed explanation of the module's purpose.

/// Brief description of function
///
/// Detailed explanation.
///
/// # Safety
///
/// Explain safety requirements for unsafe code.
pub fn example() {}
```

### Naming Conventions

- **Modules**: `snake_case` (e.g., `memory.rs`, `page_table.rs`)
- **Types/Structs**: `PascalCase` (e.g., `PhysAddr`, `VirtAddr`, `PageTable`)
- **Functions**: `snake_case` (e.g., `init`, `allocate_frame`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `VGA_BUFFER`, `BUFFER_HEIGHT`)
- **Macros**: `snake_case` (e.g., `printk!`, `printkln!`)

### Type Definitions

Use newtype pattern for type safety:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }
    
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}
```

## Architecture Guidelines

### Memory Management

- Physical addresses: Use `PhysAddr` newtype
- Virtual addresses: Use `VirtAddr` newtype
- Frame allocation: Use frame allocator in `mm/frame.rs`
- Heap allocation: Initialize heap before use via `mm/heap.rs`

### Synchronization

- Use `spin::Mutex` for kernel-space locks (no std available)
- Use atomic operations from `core::sync::atomic` where appropriate
- Document lock ordering to prevent deadlocks

### Architecture-Specific Code

- Place all architecture-specific code in `arch/<arch>/`
- Keep portable code in general modules (`kernel/`, `mm/`, etc.)
- Use trait abstractions to support multiple architectures

### Interrupt Handling

- Interrupts are managed in `arch/x86/src/interrupts.rs`
- Use `#[repr(C)]` for interrupt frames
- Always disable interrupts during critical sections

## Memory Safety and Security

### Unsafe Code

- **Minimize** use of `unsafe` blocks
- Every `unsafe` block must have a comment explaining:
  - Why it's necessary
  - What invariants are being upheld
  - What could go wrong if invariants are violated
- Document safety requirements in `# Safety` sections

Example:
```rust
/// # Safety
///
/// The caller must ensure that `addr` points to valid, initialized memory.
pub unsafe fn read_from(addr: usize) -> u8 {
    // SAFETY: Caller guarantees addr points to valid memory
    unsafe { *(addr as *const u8) }
}
```

### Security Best Practices

- Validate all external inputs (from hardware, user space, etc.)
- Check bounds before array/buffer access
- Use Rust's type system to enforce invariants
- Minimize privileged code paths
- Never expose raw pointers in public APIs without careful documentation

### Panic Handling

- The kernel has a custom panic handler (no std panic available)
- Use `panic!` for unrecoverable errors
- For recoverable errors, use `Result<T, E>` types

## Module Organization

### Adding New Modules

1. Create module file in the appropriate directory
2. Add module declaration to parent `lib.rs` or `mod.rs`
3. Export public interface items
4. Write comprehensive documentation
5. Add unit tests if applicable

### Module Initialization

Follow this pattern for subsystem initialization:
```rust
use core::sync::atomic::{AtomicBool, Ordering};

static SUBSYSTEM_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    if SUBSYSTEM_INITIALIZED.load(Ordering::Acquire) {
        return;
    }
    
    // Initialize subsystem
    
    SUBSYSTEM_INITIALIZED.store(true, Ordering::Release);
}

pub fn is_initialized() -> bool {
    SUBSYSTEM_INITIALIZED.load(Ordering::Acquire)
}
```

## Testing

### Unit Tests

- Write unit tests in `#[cfg(test)]` modules
- Test edge cases and error conditions
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let addr = PhysAddr::new(0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
    }
}
```

### Integration Tests

- Run the kernel in QEMU to test real behavior
- Use `make run` for manual testing
- Use `make test` for automated tests

## Build System

### Building

```bash
make build      # Build the kernel
make run        # Run in QEMU
make test       # Run tests
make fmt        # Format code
make clippy     # Run linter
make clean      # Clean artifacts
```

### Target Specification

- Custom target: `x86_64-unknown-rinux.json`
- Uses nightly Rust with `-Z build-std`
- Requires specific features: `#![feature(abi_x86_interrupt)]`

## Common Patterns

### Printing to Console

```rust
use rinux_kernel::printk::printk;

printk("Hello from kernel\n");

// Or use the macro:
printk!("Value: {}\n", value);
```

### Inline Assembly

- Use Rust's `asm!` macro for inline assembly
- Always specify appropriate options: `options(nomem, nostack)`
- Document what the assembly does

```rust
#[inline(always)]
pub fn halt() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
```

### Color Enums

For enums representing hardware values, use explicit discriminants:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    // ...
}
```

## Dependencies

### External Crates

- `spin`: Spinlock implementation for no_std
- `bitflags`: Bitfield operations
- Avoid adding dependencies unless absolutely necessary

### Workspace Structure

- Main crate: `rinux`
- Sub-crates: `rinux-arch-x86`, `rinux-kernel`, `rinux-mm`, `rinux-drivers`, `rinux-lib`
- All sub-crates are in the workspace and use path dependencies

## Version Control

### Commit Messages

- Use clear, descriptive commit messages
- Reference issue numbers when applicable
- Follow conventional commit format when possible

### Pull Requests

1. Create a feature branch
2. Make focused changes
3. Format and lint code
4. Update documentation
5. Add/update tests
6. Submit PR with clear description

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust OS Development Tutorial](https://os.phil-opp.com/)
- [OSDev Wiki](https://wiki.osdev.org/)
- Project docs in `docs/` directory

## Current Limitations

⚠️ Rinux is in early development (v0.1.x):
- No user authentication or access control yet
- Limited hardware support
- Not suitable for production use
- API may change between versions

## When Writing Code

1. **Check existing patterns**: Look at similar code in the codebase
2. **Follow module structure**: Respect the separation between subsystems
3. **Document thoroughly**: Especially for unsafe code and public APIs
4. **Test locally**: Build and run in QEMU before committing
5. **Think about safety**: Leverage Rust's type system for correctness
6. **Consider portability**: Separate architecture-specific from portable code
