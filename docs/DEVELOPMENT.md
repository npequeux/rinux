# Rinux Development Guide

## Getting Started

Welcome to Rinux development! This guide will help you contribute to the project.

## Code Structure

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture information.

## Development Workflow

1. **Fork and Clone**
   ```bash
   git clone https://github.com/yourusername/rinux.git
   cd rinux
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

3. **Make Changes**
   - Write code
   - Add tests
   - Update documentation

4. **Test**
   ```bash
   make test
   make run
   ```

5. **Format and Lint**
   ```bash
   make fmt
   make clippy
   ```

6. **Commit and Push**
   ```bash
   git add .
   git commit -m "Description of changes"
   git push origin feature/my-feature
   ```

7. **Create Pull Request**

## Coding Standards

### Rust Style

- Follow Rust standard style guide
- Use `cargo fmt` for formatting
- Address all `clippy` warnings

### Documentation

- Document all public items
- Include examples where appropriate
- Keep README files up to date

### Comments

```rust
/// Brief description
///
/// Detailed explanation
///
/// # Examples
///
/// ```
/// // Example code
/// ```
///
/// # Safety
///
/// Explain safety requirements for unsafe code
pub fn example() {}
```

## Module Guidelines

### Adding a New Module

1. Create module file in appropriate directory
2. Add module declaration to parent
3. Export public interface
4. Document thoroughly
5. Add tests

### Architecture-Specific Code

Place in `arch/<arch>/` directory:
- Keep portable code in general modules
- Only architecture-specific code in arch modules

## Memory Safety

- Minimize `unsafe` code
- Document all unsafe blocks
- Use Rust's type system for safety
- Validate all unsafe assumptions

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
```

### Integration Tests

Run full kernel in QEMU and verify behavior.

## Debugging

### Print Debugging

```rust
use rinux_kernel::printk::printk;

printk("Debug message\n");
```

### GDB

```bash
make debug
# In another terminal:
gdb
```

## Common Tasks

### Adding a New System Call

1. Define in `kernel/src/syscall.rs`
2. Update syscall table
3. Implement handler
4. Add tests
5. Document

### Adding a Device Driver

1. Create driver in `drivers/src/`
2. Implement driver interface
3. Register with kernel
4. Test on hardware/emulator

### Porting to New Architecture

1. Create `arch/<arch>/` directory
2. Implement required interfaces
3. Add build configuration
4. Test thoroughly

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust OS Development Tutorial](https://os.phil-opp.com/)
- [OSDev Wiki](https://wiki.osdev.org/)
- Linux kernel source for reference

## Getting Help

- Check documentation
- Read existing code
- Ask questions in issues/discussions
