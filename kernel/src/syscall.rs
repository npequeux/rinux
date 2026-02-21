//! System Call Interface
//!
//! System call numbers and handler framework.

/// System call numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SyscallNumber {
    /// Read from file descriptor
    Read = 0,
    /// Write to file descriptor
    Write = 1,
    /// Open file
    Open = 2,
    /// Close file
    Close = 3,
    /// Get file status
    Stat = 4,
    /// Get file status (by fd)
    Fstat = 5,
    /// Create new process
    Fork = 57,
    /// Execute program
    Execve = 59,
    /// Exit process
    Exit = 60,
    /// Wait for process
    Wait4 = 61,
    /// Get process ID
    Getpid = 39,
    /// Get parent process ID
    Getppid = 110,
    /// Get user ID
    Getuid = 102,
    /// Get group ID
    Getgid = 104,
    /// Set user ID
    Setuid = 105,
    /// Set group ID
    Setgid = 106,
    /// Memory map
    Mmap = 9,
    /// Memory unmap
    Munmap = 11,
    /// Change memory protection
    Mprotect = 10,
    /// Yield CPU
    SchedYield = 24,
    /// Get time
    Time = 201,
    /// Unknown/invalid syscall
    Unknown = 0xFFFFFFFF,
}

impl From<u64> for SyscallNumber {
    fn from(num: u64) -> Self {
        match num {
            0 => SyscallNumber::Read,
            1 => SyscallNumber::Write,
            2 => SyscallNumber::Open,
            3 => SyscallNumber::Close,
            4 => SyscallNumber::Stat,
            5 => SyscallNumber::Fstat,
            57 => SyscallNumber::Fork,
            59 => SyscallNumber::Execve,
            60 => SyscallNumber::Exit,
            61 => SyscallNumber::Wait4,
            39 => SyscallNumber::Getpid,
            110 => SyscallNumber::Getppid,
            102 => SyscallNumber::Getuid,
            104 => SyscallNumber::Getgid,
            105 => SyscallNumber::Setuid,
            106 => SyscallNumber::Setgid,
            9 => SyscallNumber::Mmap,
            11 => SyscallNumber::Munmap,
            10 => SyscallNumber::Mprotect,
            24 => SyscallNumber::SchedYield,
            201 => SyscallNumber::Time,
            _ => SyscallNumber::Unknown,
        }
    }
}

/// System call result
pub type SyscallResult = Result<usize, isize>;

/// System call error codes
pub mod errno {
    /// Operation not permitted
    pub const EPERM: isize = -1;
    /// No such file or directory
    pub const ENOENT: isize = -2;
    /// No such process
    pub const ESRCH: isize = -3;
    /// Bad file descriptor
    pub const EBADF: isize = -9;
    /// Out of memory
    pub const ENOMEM: isize = -12;
    /// Permission denied
    pub const EACCES: isize = -13;
    /// Bad address
    pub const EFAULT: isize = -14;
    /// Invalid argument
    pub const EINVAL: isize = -22;
    /// Function not implemented
    pub const ENOSYS: isize = -38;
}

/// Handle a system call
///
/// # Arguments
///
/// * `syscall_num` - System call number
/// * `arg1` - First argument
/// * `arg2` - Second argument
/// * `arg3` - Third argument
/// * `arg4` - Fourth argument
/// * `arg5` - Fifth argument
/// * `arg6` - Sixth argument
pub fn handle_syscall(
    syscall_num: u64,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    let syscall = SyscallNumber::from(syscall_num);

    match syscall {
        SyscallNumber::Read => {
            // TODO: Implement read
            Err(errno::ENOSYS)
        }
        SyscallNumber::Write => {
            // TODO: Implement write
            Err(errno::ENOSYS)
        }
        SyscallNumber::Open => {
            // TODO: Implement open
            Err(errno::ENOSYS)
        }
        SyscallNumber::Close => {
            // TODO: Implement close
            Err(errno::ENOSYS)
        }
        SyscallNumber::Fork => {
            // TODO: Implement fork
            Err(errno::ENOSYS)
        }
        SyscallNumber::Execve => {
            // TODO: Implement execve
            Err(errno::ENOSYS)
        }
        SyscallNumber::Exit => {
            // TODO: Implement exit
            Err(errno::ENOSYS)
        }
        SyscallNumber::Getpid => {
            // TODO: Implement getpid
            use crate::process::sched;
            if let Some(pid) = sched::current_pid() {
                Ok(pid as usize)
            } else {
                Ok(0)
            }
        }
        SyscallNumber::SchedYield => {
            use crate::process::sched;
            sched::yield_now();
            Ok(0)
        }
        SyscallNumber::Unknown => {
            crate::printk::printk("Unknown syscall\n");
            Err(errno::ENOSYS)
        }
        _ => {
            crate::printk::printk("Unknown syscall\n");
            Err(errno::ENOSYS)
        }
    }
}

/// Initialize system call interface
pub fn init() {
    crate::printk::printk("  System call interface initialized\n");
    // TODO: Setup syscall entry point in IDT
}
