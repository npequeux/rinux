//! Init Process
//!
//! The first user-space process (PID 1) for the system.

use rinux_kernel::syscall;

/// Main entry point for init process
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Print welcome message
    syscall_write(1, b"Rinux Init Process Starting...\n");

    // Mount root filesystem (TODO: implement mount syscall)
    // mount("/dev/sda1", "/", "ext2", 0, null());

    // Set up basic environment
    syscall_write(1, b"Setting up basic environment...\n");

    // Spawn shell
    let pid = syscall_fork();
    if pid == 0 {
        // Child process - execute shell
        syscall_write(1, b"Starting shell...\n");
        // execve("/bin/sh", argv, envp);
        loop {}  // TODO: Replace with actual shell execution
    } else if pid > 0 {
        // Parent process - wait for children
        syscall_write(1, b"Init: Shell spawned with PID ");
        print_number(pid as u64);
        syscall_write(1, b"\n");

        // Reap zombie processes
        loop {
            // wait4(-1, &status, 0, null());
            // For now, just yield
            syscall_yield();
        }
    } else {
        // Fork failed
        syscall_write(1, b"Init: Failed to fork shell\n");
        loop {}
    }
}

/// Fork system call wrapper
fn syscall_fork() -> isize {
    let result: isize;
    unsafe {
        core::arch::asm!(
            "mov rax, 57",  // SYS_FORK
            "syscall",
            out("rax") result,
            options(nostack)
        );
    }
    result
}

/// Write system call wrapper
fn syscall_write(fd: usize, buf: &[u8]) -> isize {
    let result: isize;
    unsafe {
        core::arch::asm!(
            "mov rax, 1",   // SYS_WRITE
            "mov rdi, {0}", // fd
            "mov rsi, {1}", // buf
            "mov rdx, {2}", // count
            "syscall",
            in(reg) fd,
            in(reg) buf.as_ptr(),
            in(reg) buf.len(),
            out("rax") result,
            options(nostack)
        );
    }
    result
}

/// Yield system call wrapper
fn syscall_yield() {
    unsafe {
        core::arch::asm!(
            "mov rax, 24",  // SYS_SCHED_YIELD
            "syscall",
            out("rax") _,
            options(nostack)
        );
    }
}

/// Print a number in decimal
fn print_number(mut n: u64) {
    if n == 0 {
        syscall_write(1, b"0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    // Reverse and print
    for j in (0..i).rev() {
        syscall_write(1, &buf[j..j+1]);
    }
}

/// Panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    syscall_write(1, b"Init process panicked!\n");
    loop {}
}
