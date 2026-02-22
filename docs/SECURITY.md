# Security Subsystem Implementation

This document describes the security features implemented in the Rinux kernel.

## Overview

The security subsystem provides five main components:

1. **Privilege Levels** (`privilege.rs`) - Ring 0/3 separation
2. **Capabilities** (`capabilities.rs`) - Fine-grained privilege control
3. **Access Control** (`access.rs`) - File permission checks
4. **ASLR** (`aslr.rs`) - Address space randomization
5. **Validation** (`validation.rs`) - Syscall parameter validation

## 1. Privilege Levels (privilege.rs)

### Features
- **Ring-based security**: Support for x86 protection rings (Ring 0 = kernel, Ring 3 = user)
- **Privilege tracking**: Per-CPU current privilege level tracking
- **Privilege transitions**: Safe helpers for entering/exiting userspace
- **Context preservation**: Save and restore privilege contexts

### Key Functions
```rust
// Check current privilege
pub fn current_privilege() -> PrivilegeLevel;
pub fn in_kernel_mode() -> bool;
pub fn in_user_mode() -> bool;

// Require specific privilege
pub fn require_kernel() -> Result<(), &'static str>;
pub fn require_user() -> Result<(), &'static str>;

// Privilege transitions
pub unsafe fn enter_userspace() -> PrivilegeContext;
pub fn return_to_kernel() -> PrivilegeContext;

// Execute with specific privilege
pub unsafe fn with_privilege<F, R>(level: PrivilegeLevel, f: F) -> R;
```

### Usage Example
```rust
use rinux_kernel::security::privilege;

// Check if in kernel mode
if privilege::in_kernel_mode() {
    // Perform privileged operation
}

// Execute function with temporary privilege
unsafe {
    privilege::with_user_privilege(|| {
        // This code runs in user mode
    });
}
```

## 2. Capabilities (capabilities.rs)

### Features
- **Linux-compatible capabilities**: 30 capability types (CAP_CHOWN, CAP_NET_ADMIN, etc.)
- **Per-process capability sets**: Permitted, effective, and inheritable capabilities
- **Atomic operations**: Thread-safe capability manipulation
- **Flexible capability checks**: Fine-grained privilege control

### Capability Types
- `CapDacOverride` - Override file permissions
- `CapChown` - Change file ownership
- `CapNetBindService` - Bind to privileged ports (<1024)
- `CapNetAdmin` - Network administration
- `CapSysAdmin` - System administration
- `CapSetuid` / `CapSetgid` - Change UID/GID
- And 24 more...

### Key Types and Functions
```rust
// Capability set operations
pub struct CapabilitySet;
impl CapabilitySet {
    pub const fn has(&self, cap: Capability) -> bool;
    pub fn add(&mut self, cap: Capability);
    pub fn remove(&mut self, cap: Capability);
}

// Per-process capabilities
pub struct ProcessCapabilities;
impl ProcessCapabilities {
    pub const fn new() -> Self;
    pub const fn root() -> Self;  // All capabilities
    pub fn has_capability(&self, cap: Capability) -> bool;
    pub fn add_capability(&self, cap: Capability);
    pub fn remove_capability(&self, cap: Capability);
}
```

### Usage Example
```rust
use rinux_kernel::security::capabilities::{ProcessCapabilities, Capability};

// Create root capabilities
let caps = ProcessCapabilities::root();

// Check capability
if caps.has_capability(Capability::CapNetAdmin) {
    // Perform network admin operation
}

// Add specific capability
caps.add_capability(Capability::CapChown);
```

## 3. Access Control (access.rs)

### Features
- **Unix-style permissions**: Owner/group/other with read/write/execute bits
- **Special bits**: setuid, setgid, sticky bit support
- **Root bypass**: Root (uid 0) bypasses most checks
- **Comprehensive checks**: Functions for all common permission checks

### Permission Bits
- Owner: read (0o400), write (0o200), execute (0o100)
- Group: read (0o040), write (0o020), execute (0o010)
- Other: read (0o004), write (0o002), execute (0o001)
- Special: setuid (0o4000), setgid (0o2000), sticky (0o1000)

### Key Functions
```rust
// Permission checks
pub fn check_permission(
    uid: Uid, gid: Gid,
    file_uid: Uid, file_gid: Gid,
    perms: FilePermissions,
    mode: AccessMode
) -> bool;

pub fn can_read(...) -> bool;
pub fn can_write(...) -> bool;
pub fn can_execute(...) -> bool;

// Ownership checks
pub fn can_chown(uid: Uid, file_uid: Uid) -> bool;
pub fn can_chmod(uid: Uid, file_uid: Uid) -> bool;
pub fn can_delete(...) -> bool;
```

### Usage Example
```rust
use rinux_kernel::security::access::{FilePermissions, can_read, can_write};

// Create permissions (0o755)
let perms = FilePermissions::from_mode(0o755);

// Check if user 1000 in group 1000 can read file owned by user 1000
if can_read(1000, 1000, 1000, 1000, perms) {
    // Grant read access
}

// Check write permission
if can_write(1000, 1000, 1000, 1000, perms) {
    // Grant write access
}
```

## 4. ASLR (aslr.rs)

### Features
- **Address randomization**: Randomize stack, heap, mmap, and PIE regions
- **Configurable entropy**: Adjustable randomization bits per region
- **Pseudo-random number generation**: Built-in PRNG (LCG-based)
- **Flexible configuration**: Enable/disable per-region randomization

### Default Randomization
- Stack: 28 bits (256MB range)
- Heap: 28 bits (256MB range)
- Mmap: 28 bits (256MB range)
- PIE: 28 bits (256MB range)

### Key Functions
```rust
// Initialize ASLR
pub fn init();

// Randomize different regions
pub fn randomize_stack() -> u64;
pub fn randomize_heap() -> u64;
pub fn randomize_mmap() -> u64;
pub fn randomize_pie() -> u64;

// Generic randomization
pub fn randomize_address(base: VirtAddr, size: u64, random_bits: u32) -> VirtAddr;

// Configuration
pub struct AslrConfig;
impl AslrConfig {
    pub const fn default() -> Self;  // All enabled
    pub const fn disabled() -> Self; // All disabled
    pub fn apply_stack(&self, base: VirtAddr, size: u64) -> VirtAddr;
}
```

### Usage Example
```rust
use rinux_kernel::security::aslr::{self, AslrConfig};
use rinux_kernel::types::VirtAddr;

// Initialize ASLR
aslr::init();

// Randomize stack
let stack_offset = aslr::randomize_stack();
let stack_addr = VirtAddr::new(0x7FFF_0000_0000 + stack_offset);

// Use configuration
let config = AslrConfig::default();
let heap_addr = config.apply_heap(VirtAddr::new(0x1000_0000), 0x1000_0000);
```

## 5. Parameter Validation (validation.rs)

### Features
- **User/kernel space separation**: Validate addresses are in correct space
- **Bounds checking**: Validate buffer sizes and ranges
- **Safe copy functions**: Copy data between user and kernel space
- **String validation**: Null-termination and length checks

### Address Ranges
- User space: `0x0000_0000_0000_0000` to `0x0000_7FFF_FFFF_FFFF`
- Kernel space: `0xFFFF_8000_0000_0000` to `0xFFFF_FFFF_FFFF_FFFF`

### Key Functions
```rust
// Address validation
pub fn is_user_address(addr: VirtAddr) -> bool;
pub fn is_kernel_address(addr: VirtAddr) -> bool;
pub fn is_user_range(addr: VirtAddr, size: usize) -> bool;

// Pointer validation
pub fn validate_user_ptr(ptr: *const u8) -> Result<(), &'static str>;
pub fn validate_user_buffer(ptr: *const u8, size: usize) -> Result<(), &'static str>;
pub unsafe fn validate_user_string(ptr: *const u8, max_len: usize) -> Result<usize, &'static str>;

// Safe copy functions
pub unsafe fn copy_from_user(user_ptr: *const u8, kernel_buf: &mut [u8]) -> Result<(), &'static str>;
pub unsafe fn copy_to_user(kernel_buf: &[u8], user_ptr: *mut u8) -> Result<(), &'static str>;

// Type-safe operations
pub unsafe fn read_user<T: Copy>(user_ptr: *const T) -> Result<T, &'static str>;
pub unsafe fn write_user<T: Copy>(user_ptr: *mut T, value: T) -> Result<(), &'static str>;
```

### Usage Example
```rust
use rinux_kernel::security::validation;

// Validate user pointer before use
fn handle_syscall(user_buf: *const u8, size: usize) -> Result<(), &'static str> {
    // Validate the buffer
    validation::validate_user_buffer(user_buf, size)?;
    
    // Safe to copy from user space
    let mut kernel_buf = vec![0u8; size];
    unsafe {
        validation::copy_from_user(user_buf, &mut kernel_buf)?;
    }
    
    // Process kernel_buf...
    Ok(())
}
```

## Integration with Task Structure

The security subsystem is integrated with the process management:

```rust
// In kernel/src/process/task.rs
pub struct Task {
    pub uid: Uid,
    pub gid: Gid,
    pub capabilities: Arc<ProcessCapabilities>,
    // ... other fields
}

impl Task {
    pub fn is_root(&self) -> bool;
    pub fn has_capability(&self, cap: Capability) -> bool;
}
```

## Initialization

The security subsystem is initialized early in the kernel boot:

```rust
// In kernel/src/lib.rs
pub fn init() {
    // ... other subsystems
    security::init();
    // ... more subsystems
}
```

## Thread Safety

All security components are designed to be thread-safe:
- **Atomic operations** for capability sets and privilege levels
- **Lock-free reads** where possible
- **No mutable global state** without proper synchronization

## Testing

Comprehensive unit tests are included for all modules:
- `privilege.rs`: 8 tests covering privilege transitions
- `capabilities.rs`: 7 tests covering capability operations
- `access.rs`: 10 tests covering permission checks
- `aslr.rs`: 9 tests covering randomization
- `validation.rs`: 7 tests covering pointer validation

Run tests with:
```bash
make test
```

## Future Enhancements

Potential improvements for the security subsystem:

1. **SELinux/AppArmor-style MAC**: Mandatory access control policies
2. **Audit logging**: Security event logging and audit trails
3. **Secure boot**: Boot chain verification
4. **Memory protection**: W^X, stack canaries, guard pages
5. **Sandboxing**: Process isolation and containment
6. **Cryptographic RNG**: Replace PRNG with hardware RNG (RDRAND)
7. **Per-CPU privilege tracking**: True per-CPU privilege state
8. **Page table isolation**: Kernel page table isolation (KPTI)

## Security Considerations

### Current Limitations
- **Placeholder implementations**: Some functions are placeholders (e.g., `cap_capable`)
- **Simple PRNG**: ASLR uses LCG instead of cryptographic RNG
- **No page table checks**: Validation doesn't verify page mappings
- **No fault handling**: Copy functions don't handle page faults gracefully

### Best Practices
1. **Always validate user pointers** before dereferencing
2. **Check capabilities** before privileged operations
3. **Use ASLR** for all new processes
4. **Minimize time in privileged mode**
5. **Audit security-critical operations**

## References

- [Linux Capabilities Man Page](https://man7.org/linux/man-pages/man7/capabilities.7.html)
- [x86 Protection Rings](https://en.wikipedia.org/wiki/Protection_ring)
- [ASLR](https://en.wikipedia.org/wiki/Address_space_layout_randomization)
- [Unix File Permissions](https://en.wikipedia.org/wiki/File-system_permissions)
