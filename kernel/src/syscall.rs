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
    arg1: usize,
    arg2: usize,
    arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SyscallResult {
    let syscall = SyscallNumber::from(syscall_num);

    match syscall {
        SyscallNumber::Read => {
            // arg1: fd, arg2: buf ptr, arg3: count
            let _fd = arg1 as i32;
            let _buf = arg2 as *mut u8;
            let _count = arg3;
            // TODO: Implement proper read with buffer safety checks
            Err(errno::ENOSYS)
        }
        SyscallNumber::Write => {
            // arg1: fd, arg2: buf ptr, arg3: count
            let _fd = arg1 as i32;
            let _buf = arg2 as *const u8;
            let _count = arg3;
            // TODO: Implement proper write with buffer safety checks
            Err(errno::ENOSYS)
        }
        SyscallNumber::Open => {
            // TODO: Implement open
            Err(errno::ENOSYS)
        }
        SyscallNumber::Close => {
            // arg1: fd
            let fd = arg1 as i32;
            match crate::fs::fd::free_fd(fd) {
                Ok(()) => Ok(0),
                Err(()) => Err(errno::EBADF),
            }
        }
        SyscallNumber::Fork => {
            // TODO: Implement fork - create child process
            Err(errno::ENOSYS)
        }
        SyscallNumber::Execve => {
            // TODO: Implement execve - replace process image
            Err(errno::ENOSYS)
        }
        SyscallNumber::Exit => {
            // arg1: exit code
            let exit_code = arg1 as i32;
            // TODO: Mark current process as exited
            if let Some(pid) = crate::process::sched::current_pid() {
                crate::process::sched::remove_task(pid);
            }
            Ok(exit_code as usize)
        }
        SyscallNumber::Wait4 => {
            // TODO: Implement wait4 - wait for process
            Err(errno::ENOSYS)
        }
        SyscallNumber::Getpid => {
            use crate::process::sched;
            if let Some(pid) = sched::current_pid() {
                Ok(pid as usize)
            } else {
                Ok(0)
            }
        }
        SyscallNumber::Getppid => {
            // TODO: Get parent PID from task structure
            Ok(0)
        }
        SyscallNumber::Getuid => {
            // TODO: Get UID from task structure
            Ok(0)
        }
        SyscallNumber::Getgid => {
            // TODO: Get GID from task structure
            Ok(0)
        }
        SyscallNumber::Setuid => {
            // TODO: Set UID in task structure
            Err(errno::ENOSYS)
        }
        SyscallNumber::Setgid => {
            // TODO: Set GID in task structure
            Err(errno::ENOSYS)
        }
        SyscallNumber::Mmap => {
            // TODO: Implement memory mapping
            Err(errno::ENOSYS)
        }
        SyscallNumber::Munmap => {
            // TODO: Implement memory unmapping
            Err(errno::ENOSYS)
        }
        SyscallNumber::Mprotect => {
            // TODO: Implement memory protection change
            Err(errno::ENOSYS)
        }
        SyscallNumber::SchedYield => {
            use crate::process::sched;
            sched::yield_now();
            Ok(0)
        }
        SyscallNumber::Time => {
            // Return system time in seconds
            let time = crate::time::uptime_sec();
            Ok(time as usize)
        }
        SyscallNumber::Stat => {
            // TODO: Implement stat
            Err(errno::ENOSYS)
        }
        SyscallNumber::Fstat => {
            // TODO: Implement fstat
            Err(errno::ENOSYS)
        }
        SyscallNumber::Unknown | _ => {
            crate::printk::printk("Unknown syscall: ");
            // TODO: Print syscall number
            crate::printk::printk("\n");
            Err(errno::ENOSYS)
        }
    }
}

/// Initialize system call interface
pub fn init() {
    crate::printk::printk("  System call interface initialized\n");
    // TODO: Setup syscall entry point in IDT
}
