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
    /// Get time of day
    Gettimeofday = 96,
    /// Seek in file
    Lseek = 8,
    /// Duplicate file descriptor
    Dup = 32,
    /// Duplicate file descriptor to specific fd
    Dup2 = 33,
    /// Get current working directory
    Getcwd = 79,
    /// Change directory
    Chdir = 80,
    /// Create directory
    Mkdir = 83,
    /// Remove directory
    Rmdir = 84,
    /// Unlink/delete file
    Unlink = 87,
    /// Rename file
    Rename = 82,
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
            96 => SyscallNumber::Gettimeofday,
            8 => SyscallNumber::Lseek,
            32 => SyscallNumber::Dup,
            33 => SyscallNumber::Dup2,
            79 => SyscallNumber::Getcwd,
            80 => SyscallNumber::Chdir,
            83 => SyscallNumber::Mkdir,
            84 => SyscallNumber::Rmdir,
            87 => SyscallNumber::Unlink,
            82 => SyscallNumber::Rename,
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
    /// Interrupted system call
    pub const EINTR: isize = -4;
    /// I/O error
    pub const EIO: isize = -5;
    /// Bad file descriptor
    pub const EBADF: isize = -9;
    /// Out of memory
    pub const ENOMEM: isize = -12;
    /// Permission denied
    pub const EACCES: isize = -13;
    /// Bad address
    pub const EFAULT: isize = -14;
    /// File exists
    pub const EEXIST: isize = -17;
    /// Not a directory
    pub const ENOTDIR: isize = -20;
    /// Is a directory
    pub const EISDIR: isize = -21;
    /// Invalid argument
    pub const EINVAL: isize = -22;
    /// Too many open files
    pub const EMFILE: isize = -24;
    /// Out of range
    pub const ERANGE: isize = -34;
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
            let fd = arg1 as i32;
            let buf = arg2 as *mut u8;
            let count = arg3;

            // Validate buffer pointer
            if buf.is_null() || count == 0 {
                return Err(errno::EINVAL);
            }

            // Read from file descriptor
            match crate::fs::fd::read_fd(fd, buf, count) {
                Ok(bytes_read) => Ok(bytes_read),
                Err(_) => Err(errno::EBADF),
            }
        }
        SyscallNumber::Write => {
            // arg1: fd, arg2: buf ptr, arg3: count
            let fd = arg1 as i32;
            let buf = arg2 as *const u8;
            let count = arg3;

            // Validate buffer pointer
            if buf.is_null() || count == 0 {
                return Err(errno::EINVAL);
            }

            // Write to file descriptor
            match crate::fs::fd::write_fd(fd, buf, count) {
                Ok(bytes_written) => Ok(bytes_written),
                Err(_) => Err(errno::EBADF),
            }
        }
        SyscallNumber::Open => {
            // arg1: pathname ptr, arg2: flags, arg3: mode
            let pathname_ptr = arg1 as *const u8;
            let flags = arg2 as i32;
            let mode = arg3 as u32;

            // Validate pathname pointer
            if pathname_ptr.is_null() {
                return Err(errno::EFAULT);
            }

            // Read pathname from user space
            let pathname = unsafe {
                let mut len = 0;
                while len < 4096 && *pathname_ptr.add(len) != 0 {
                    len += 1;
                }
                if len == 0 {
                    return Err(errno::EINVAL);
                }
                let slice = core::slice::from_raw_parts(pathname_ptr, len);
                core::str::from_utf8(slice).map_err(|_| errno::EINVAL)?
            };

            // Open file via VFS
            match crate::fs::open_file(pathname, flags, mode) {
                Ok(fd) => Ok(fd as usize),
                Err(e) => Err(e),
            }
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
            let _exit_code = arg1 as i32;
            // Mark current process as exited and remove from scheduler
            if let Some(pid) = crate::process::sched::current_pid() {
                crate::process::sched::remove_task(pid);
            }
            // Trigger scheduler to switch to another task
            crate::process::sched::schedule();
            // Should never return here, but if we do, return error
            Err(errno::ESRCH)
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
            // TODO: Get parent PID from current task
            Ok(0)
        }
        SyscallNumber::Getuid => {
            // TODO: Get UID from current task
            Ok(0)
        }
        SyscallNumber::Getgid => {
            // TODO: Get GID from current task
            Ok(0)
        }
        SyscallNumber::Setuid => {
            // TODO: Set UID with capability check
            Err(errno::EPERM)
        }
        SyscallNumber::Setgid => {
            // TODO: Set GID with capability check
            Err(errno::EPERM)
        }
        SyscallNumber::Lseek => {
            let fd = arg1 as i32;
            let offset = arg2 as i64;
            let whence = arg3 as i32;

            match crate::fs::fd::seek_fd(fd, offset, whence) {
                Ok(new_pos) => Ok(new_pos as usize),
                Err(_) => Err(errno::EBADF),
            }
        }
        SyscallNumber::Dup => {
            let fd = arg1 as i32;
            match crate::fs::fd::dup_fd(fd) {
                Ok(new_fd) => Ok(new_fd as usize),
                Err(_) => Err(errno::EBADF),
            }
        }
        SyscallNumber::Dup2 => {
            let oldfd = arg1 as i32;
            let newfd = arg2 as i32;
            match crate::fs::fd::dup2_fd(oldfd, newfd) {
                Ok(fd) => Ok(fd as usize),
                Err(_) => Err(errno::EBADF),
            }
        }
        SyscallNumber::Getcwd => {
            let buf = arg1 as *mut u8;
            let size = arg2;

            if buf.is_null() || size == 0 {
                return Err(errno::EINVAL);
            }

            // Get current working directory (default to "/" for now)
            let cwd = "/";
            let cwd_bytes = cwd.as_bytes();

            if cwd_bytes.len() + 1 > size {
                return Err(errno::ERANGE);
            }

            unsafe {
                core::ptr::copy_nonoverlapping(cwd_bytes.as_ptr(), buf, cwd_bytes.len());
                *buf.add(cwd_bytes.len()) = 0; // Null terminator
            }

            Ok(arg1) // Return buffer pointer
        }
        SyscallNumber::Chdir => {
            let path_ptr = arg1 as *const u8;

            if path_ptr.is_null() {
                return Err(errno::EFAULT);
            }

            // Read path from user space
            let path = unsafe {
                let mut len = 0;
                while len < 4096 && *path_ptr.add(len) != 0 {
                    len += 1;
                }
                if len == 0 {
                    return Err(errno::EINVAL);
                }
                let slice = core::slice::from_raw_parts(path_ptr, len);
                core::str::from_utf8(slice).map_err(|_| errno::EINVAL)?
            };

            // TODO: Actually change directory and verify it exists
            let _ = path;
            Ok(0)
        }
        SyscallNumber::Mkdir => {
            let path_ptr = arg1 as *const u8;
            let mode = arg2 as u32;

            if path_ptr.is_null() {
                return Err(errno::EFAULT);
            }

            // Read path from user space
            let path = unsafe {
                let mut len = 0;
                while len < 4096 && *path_ptr.add(len) != 0 {
                    len += 1;
                }
                let slice = core::slice::from_raw_parts(path_ptr, len);
                core::str::from_utf8(slice).map_err(|_| errno::EINVAL)?
            };

            // TODO: Create directory via VFS
            let _ = (path, mode);
            Err(errno::ENOSYS)
        }
        SyscallNumber::Rmdir => {
            let path_ptr = arg1 as *const u8;

            if path_ptr.is_null() {
                return Err(errno::EFAULT);
            }

            // Read path
            let path = unsafe {
                let mut len = 0;
                while len < 4096 && *path_ptr.add(len) != 0 {
                    len += 1;
                }
                let slice = core::slice::from_raw_parts(path_ptr, len);
                core::str::from_utf8(slice).map_err(|_| errno::EINVAL)?
            };

            // TODO: Remove directory via VFS
            let _ = path;
            Err(errno::ENOSYS)
        }
        SyscallNumber::Unlink => {
            let path_ptr = arg1 as *const u8;

            if path_ptr.is_null() {
                return Err(errno::EFAULT);
            }

            let path = unsafe {
                let mut len = 0;
                while len < 4096 && *path_ptr.add(len) != 0 {
                    len += 1;
                }
                let slice = core::slice::from_raw_parts(path_ptr, len);
                core::str::from_utf8(slice).map_err(|_| errno::EINVAL)?
            };

            // TODO: Unlink file via VFS
            let _ = path;
            Err(errno::ENOSYS)
        }
        SyscallNumber::Rename => {
            let oldpath_ptr = arg1 as *const u8;
            let newpath_ptr = arg2 as *const u8;

            if oldpath_ptr.is_null() || newpath_ptr.is_null() {
                return Err(errno::EFAULT);
            }

            // TODO: Rename file via VFS
            let _ = (oldpath_ptr, newpath_ptr);
            Err(errno::ENOSYS)
        }
        SyscallNumber::Gettimeofday => {
            let tv_ptr = arg1 as *mut u64;

            if !tv_ptr.is_null() {
                // Return current uptime in microseconds
                // TODO: Implement real wall-clock time
                unsafe {
                    // For now, return uptime (simplified)
                    *tv_ptr = 0; // seconds
                    *tv_ptr.add(1) = 0; // microseconds
                }
            }
            Ok(0)
        }
        SyscallNumber::Mmap => {
            // arg1: addr, arg2: length, arg3: prot, arg4: flags, arg5: fd, arg6: offset
            let addr = if arg1 == 0 { None } else { Some(arg1) };
            let length = arg2;
            let prot = arg3 as i32;
            let flags = _arg4 as i32;
            let fd = _arg5 as i32;
            let offset = _arg6;

            // Use rinux_mm crate's mmap
            match rinux_mm::mmap::mmap(addr, length, prot, flags, fd, offset) {
                Ok(mapped_addr) => Ok(mapped_addr),
                Err(_) => Err(errno::ENOMEM),
            }
        }
        SyscallNumber::Munmap => {
            // arg1: addr, arg2: length
            let addr = arg1;
            let length = arg2;

            // Use rinux_mm crate's munmap
            match rinux_mm::mmap::munmap(addr, length) {
                Ok(()) => Ok(0),
                Err(_) => Err(errno::EINVAL),
            }
        }
        SyscallNumber::Mprotect => {
            // arg1: addr, arg2: length, arg3: prot
            let addr = arg1;
            let length = arg2;
            let prot = arg3 as i32;

            // Use rinux_mm crate's mprotect
            match rinux_mm::mmap::mprotect(addr, length, prot) {
                Ok(()) => Ok(0),
                Err(_) => Err(errno::EINVAL),
            }
        }
        SyscallNumber::SchedYield => {
            use crate::process::sched;
            sched::yield_now();
            Ok(0)
        }
        SyscallNumber::Time => {
            // POSIX time(2): should return seconds since Unix epoch.
            // We currently only have uptime, not a real wall-clock, so this is unimplemented.
            Err(errno::ENOSYS)
        }
        SyscallNumber::Stat => {
            // TODO: Implement stat
            Err(errno::ENOSYS)
        }
        SyscallNumber::Fstat => {
            // TODO: Implement fstat
            Err(errno::ENOSYS)
        }
        SyscallNumber::Unknown => {
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
