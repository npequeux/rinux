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

Rinux uses Rust's built-in testing framework for unit tests. Since Rinux is a `no_std` kernel, tests must be run with a standard target rather than the custom kernel target.

#### Running Tests

```bash
# Run all library tests
cd lib && cargo +nightly test --lib --target x86_64-unknown-linux-gnu

# Run all kernel tests
cd kernel && cargo +nightly test --lib --target x86_64-unknown-linux-gnu

# Run tests for a specific module
cd lib && cargo +nightly test --lib --target x86_64-unknown-linux-gnu math
```

#### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
    
    #[test]
    fn test_with_setup() {
        let value = calculate_something();
        assert!(value > 0);
    }
}
```

#### Test Coverage

Current test coverage:
- **lib/**: 27 tests covering math, string, list, and version functions
- **kernel/**: 34 tests covering types, PID allocation, and task management
- **Total**: 61 unit tests

To see which tests are available:
```bash
cargo +nightly test --lib --target x86_64-unknown-linux-gnu -- --list
```

### Integration Tests

Run full kernel in QEMU and verify behavior:
```bash
make run
```

### CI/CD Tests

Tests are automatically run in GitHub Actions on every push and pull request. See `.github/workflows/ci.yml` for details.

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
