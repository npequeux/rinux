# Rinux Roadmap

## Version 0.1.0 (Current)

- [x] Basic project structure
- [x] x86_64 architecture support
- [x] Console output
- [x] Memory management basics
- [x] Interrupt handling

## Version 0.2.0

### Core Features
- [ ] Complete memory allocator
- [ ] Process/thread management
- [ ] Basic scheduler
- [ ] System call interface
- [ ] User/kernel space separation

### Drivers
- [ ] Complete serial driver
- [ ] Keyboard input
- [ ] Timer/RTC
- [ ] PCI enumeration

## Version 0.3.0

### File Systems
- [ ] VFS layer
- [ ] Tmpfs/ramfs
- [ ] initramfs support
- [ ] Basic ext2 read support

### Networking
- [ ] Network stack skeleton
- [ ] Loopback interface
- [ ] Basic socket API

## Version 0.4.0

### Advanced Features
- [ ] Multi-core SMP support
- [ ] Advanced scheduler (CFS-like)
- [ ] Signal handling
- [ ] IPC mechanisms

### File Systems
- [ ] Ext4 support
- [ ] Proc filesystem
- [ ] Sysfs

## Version 0.5.0

### Networking
- [ ] TCP/IP stack
- [ ] Network drivers (e1000, virtio-net)
- [ ] Socket implementation

### Drivers
- [ ] Block device layer
- [ ] ATA/AHCI driver
- [ ] Virtio drivers

## Version 1.0.0

### Stability
- [ ] Full test coverage
- [ ] Security audit
- [ ] Performance optimization
- [ ] Complete documentation

### Features
- [ ] Full POSIX compatibility (subset)
- [ ] Self-hosting capability
- [ ] Bootable on real hardware

## Future (Post 1.0)

### Additional Architectures
- [ ] ARM64 support
- [ ] RISC-V support
- [ ] ARM32 support

### Advanced Features
- [ ] Kernel modules
- [ ] Live patching
- [ ] Containers/namespaces
- [ ] eBPF support

### Desktop Features
- [ ] Graphics support (Framebuffer)
- [ ] USB stack
- [ ] Audio support
- [ ] Input device support

### Ecosystem
- [ ] Package manager
- [ ] Standard utilities
- [ ] Shell
- [ ] Build system

## Long Term Vision

Rinux aims to be:
- **Safe**: Leveraging Rust's safety guarantees
- **Modern**: Using contemporary OS design principles
- **Fast**: Optimized for performance
- **Portable**: Supporting multiple architectures
- **Educational**: Well-documented and approachable
- **Practical**: Usable for real workloads
