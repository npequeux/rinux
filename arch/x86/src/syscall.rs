//! System Call Entry and Exit
//!
//! Low-level system call handling for x86_64 using the syscall instruction.

use core::arch::asm;

/// MSR addresses for syscall/sysret
const MSR_STAR: u32 = 0xC0000081; // CS/SS selectors for syscall/sysret
const MSR_LSTAR: u32 = 0xC0000082; // 64-bit mode syscall target
const _MSR_CSTAR: u32 = 0xC0000083; // Compatibility mode syscall target
const MSR_SFMASK: u32 = 0xC0000084; // Flag mask for syscall

/// System call frame (saved registers)
#[repr(C)]
pub struct SyscallFrame {
    // Callee-saved registers
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,

    // Syscall number and arguments
    pub rax: u64, // syscall number / return value
    pub rdi: u64, // arg1
    pub rsi: u64, // arg2
    pub rdx: u64, // arg3
    pub r10: u64, // arg4 (rcx is used by syscall instruction)
    pub r8: u64,  // arg5
    pub r9: u64,  // arg6

    // Saved by syscall instruction
    pub rcx: u64, // return rip
    pub r11: u64, // return rflags
}

/// Initialize system call support
pub fn init() {
    unsafe {
        // Set up syscall entry point
        write_msr(MSR_LSTAR, syscall_entry as *const () as u64);

        // Set up segment selectors
        // STAR[63:48] = kernel CS (0x08), SS (0x10)
        // STAR[47:32] = user CS (0x18 | 3), SS (0x20 | 3)
        let star: u64 = (0x0008u64 << 32) | ((0x0018 | 3) as u64) << 48;
        write_msr(MSR_STAR, star);

        // Set up flag mask (mask out IF, TF, DF)
        write_msr(MSR_SFMASK, 0x300);

        // Enable syscall/sysret in EFER
        let mut efer = read_msr(0xC0000080);
        efer |= 1 << 0; // SCE (System Call Extensions)
        write_msr(0xC0000080, efer);
    }

    crate::kernel::printk::printk("  System call interface initialized\n");
}

/// Write to MSR
unsafe fn write_msr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nostack, preserves_flags)
    );
}

/// Read from MSR
unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}

/// System call entry point (naked function)
#[unsafe(naked)]
pub unsafe extern "C" fn syscall_entry() -> ! {
    core::arch::naked_asm!(
        // TODO: Save user stack pointer and switch to kernel stack
        // This requires per-CPU data structure with kernel stack pointer
        // For now, assume stack is already correct

        // Allocate space for SyscallFrame
        "sub rsp, 0x80",

        // Save registers
        "mov [rsp + 0x00], r15",
        "mov [rsp + 0x08], r14",
        "mov [rsp + 0x10], r13",
        "mov [rsp + 0x18], r12",
        "mov [rsp + 0x20], rbp",
        "mov [rsp + 0x28], rbx",

        // Save syscall number and arguments
        "mov [rsp + 0x30], rax",  // syscall number
        "mov [rsp + 0x38], rdi",  // arg1
        "mov [rsp + 0x40], rsi",  // arg2
        "mov [rsp + 0x48], rdx",  // arg3
        "mov [rsp + 0x50], r10",  // arg4
        "mov [rsp + 0x58], r8",   // arg5
        "mov [rsp + 0x60], r9",   // arg6

        // Save return address and flags
        "mov [rsp + 0x68], rcx",  // return rip
        "mov [rsp + 0x70], r11",  // return rflags

        // Set up arguments for syscall_handler
        "mov rdi, rsp",           // pass SyscallFrame pointer

        // Call the high-level handler
        "call {0}",

        // Restore return value
        "mov rax, [rsp + 0x30]",

        // Restore callee-saved registers
        "mov r15, [rsp + 0x00]",
        "mov r14, [rsp + 0x08]",
        "mov r13, [rsp + 0x10]",
        "mov r12, [rsp + 0x18]",
        "mov rbp, [rsp + 0x20]",
        "mov rbx, [rsp + 0x28]",

        // Restore return address and flags
        "mov rcx, [rsp + 0x68]",
        "mov r11, [rsp + 0x70]",

        // Restore stack pointer
        "add rsp, 0x80",

        // Return to user space
        "sysretq",

        sym syscall_handler,
    )
}

/// High-level system call handler
#[no_mangle]
extern "C" fn syscall_handler(frame: &mut SyscallFrame) {
    use rinux_kernel::syscall::SyscallNumber;

    let syscall_num = SyscallNumber::from(frame.rax);

    // Dispatch to appropriate handler
    let result = match syscall_num {
        SyscallNumber::Read => {
            // sys_read(frame.rdi as i32, frame.rsi as *mut u8, frame.rdx as usize)
            Err(-38) // ENOSYS - not implemented
        }
        SyscallNumber::Write => {
            // sys_write(frame.rdi as i32, frame.rsi as *const u8, frame.rdx as usize)
            Err(-38)
        }
        SyscallNumber::Open => Err(-38),
        SyscallNumber::Close => Err(-38),
        SyscallNumber::Fork => {
            // Call kernel fork implementation
            match rinux_kernel::process::fork::do_fork() {
                Ok(child_pid) => Ok(child_pid as usize),
                Err(_) => Err(-12), // ENOMEM
            }
        }
        SyscallNumber::Execve => Err(-38),
        SyscallNumber::Exit => {
            // sys_exit(frame.rdi as i32)
            // This should not return
            Err(-38)
        }
        SyscallNumber::Getpid => {
            // Get current process ID
            match rinux_kernel::process::sched::current_pid() {
                Some(pid) => Ok(pid as usize),
                None => Ok(0),
            }
        }
        SyscallNumber::SchedYield => {
            rinux_kernel::process::sched::yield_now();
            Ok(0)
        }
        _ => {
            // Unknown syscall
            Err(-38) // ENOSYS
        }
    };

    // Set return value
    frame.rax = match result {
        Ok(val) => val as u64,
        Err(errno) => errno as u64, // Linux uses negative values for errors in syscall return
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_frame_size() {
        assert_eq!(core::mem::size_of::<SyscallFrame>(), 0x78);
    }
}
