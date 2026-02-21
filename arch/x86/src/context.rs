//! Context Switching
//!
//! Low-level context switching implementation for x86_64.

/// CPU context for context switching
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Context {
    // Callee-saved registers
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
    // Return address (rip)
    pub rip: u64,
}

impl Context {
    /// Create a new context
    pub const fn new() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: 0,
        }
    }

    /// Initialize context for a new task
    pub fn init(&mut self, stack_top: u64, entry_point: u64) {
        self.rbp = stack_top;
        self.rip = entry_point;
    }
}

/// Switch from one context to another
///
/// # Safety
/// This function manipulates CPU registers directly and must be called
/// with valid context pointers.
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(old: *mut Context, new: *const Context) {
    core::arch::naked_asm!(
        // Save old context (callee-saved registers)
        "mov [rdi + 0x00], r15",
        "mov [rdi + 0x08], r14",
        "mov [rdi + 0x10], r13",
        "mov [rdi + 0x18], r12",
        "mov [rdi + 0x20], rbx",
        "mov [rdi + 0x28], rbp",
        // Save return address
        "mov rax, [rsp]",
        "mov [rdi + 0x30], rax",
        // Load new context
        "mov r15, [rsi + 0x00]",
        "mov r14, [rsi + 0x08]",
        "mov r13, [rsi + 0x10]",
        "mov r12, [rsi + 0x18]",
        "mov rbx, [rsi + 0x20]",
        "mov rbp, [rsi + 0x28]",
        "mov rax, [rsi + 0x30]",
        "mov [rsp], rax",
        "ret",
    )
}

/// Interrupt frame for syscalls and interrupts
#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptFrame {
    // Pushed by CPU (in reverse order)
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl InterruptFrame {
    /// Create a new interrupt frame
    pub const fn new() -> Self {
        Self {
            rip: 0,
            cs: 0,
            rflags: 0,
            rsp: 0,
            ss: 0,
        }
    }
}

/// Full register state for task switching (includes all registers)
#[derive(Clone, Copy)]
#[repr(C)]
pub struct FullContext {
    // General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    // Instruction pointer and flags
    pub rip: u64,
    pub rflags: u64,
    // Segment registers
    pub cs: u64,
    pub ss: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}

impl FullContext {
    /// Create a new full context
    pub const fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0,
            cs: 0,
            ss: 0,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
        }
    }

    /// Initialize for user mode
    pub fn init_user(&mut self, entry: u64, stack: u64) {
        self.rip = entry;
        self.rsp = stack;
        self.rflags = 0x202; // IF (interrupt enable) flag
        self.cs = 0x18 | 3; // User code segment with RPL=3
        self.ss = 0x20 | 3; // User data segment with RPL=3
        self.ds = 0x20 | 3;
        self.es = 0x20 | 3;
    }

    /// Initialize for kernel mode
    pub fn init_kernel(&mut self, entry: u64, stack: u64) {
        self.rip = entry;
        self.rsp = stack;
        self.rflags = 0x202; // IF flag
        self.cs = 0x08; // Kernel code segment
        self.ss = 0x10; // Kernel data segment
        self.ds = 0x10;
        self.es = 0x10;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = Context::new();
        assert_eq!(ctx.rip, 0);
        assert_eq!(ctx.rbp, 0);
    }

    #[test]
    fn test_context_init() {
        let mut ctx = Context::new();
        ctx.init(0x1000, 0x400000);
        assert_eq!(ctx.rbp, 0x1000);
        assert_eq!(ctx.rip, 0x400000);
    }

    #[test]
    fn test_interrupt_frame_new() {
        let frame = InterruptFrame::new();
        assert_eq!(frame.rip, 0);
        assert_eq!(frame.rsp, 0);
    }

    #[test]
    fn test_full_context_new() {
        let ctx = FullContext::new();
        assert_eq!(ctx.rax, 0);
        assert_eq!(ctx.rip, 0);
    }

    #[test]
    fn test_full_context_init_user() {
        let mut ctx = FullContext::new();
        ctx.init_user(0x400000, 0x7FFFFFFFE000);
        assert_eq!(ctx.rip, 0x400000);
        assert_eq!(ctx.rsp, 0x7FFFFFFFE000);
        assert_eq!(ctx.cs & 3, 3); // Check RPL=3 (user mode)
    }

    #[test]
    fn test_full_context_init_kernel() {
        let mut ctx = FullContext::new();
        ctx.init_kernel(0xFFFFFFFF80000000, 0xFFFFFFFF80100000);
        assert_eq!(ctx.rip, 0xFFFFFFFF80000000);
        assert_eq!(ctx.rsp, 0xFFFFFFFF80100000);
        assert_eq!(ctx.cs, 0x08); // Kernel code segment
    }
}
