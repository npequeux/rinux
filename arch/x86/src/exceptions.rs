//! Exception Handlers
//!
//! Complete exception handlers for x86_64 CPU exceptions.

use crate::idt::InterruptStackFrame;

/// Division Error (#DE)
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Division Error (#DE)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::printk!("CS:  {:#x}\n", stack_frame.code_segment);
    kernel::printk!("RFLAGS: {:#x}\n", stack_frame.cpu_flags);
    kernel::printk!("RSP: {:#x}\n", stack_frame.stack_pointer.as_u64());
    kernel::printk!("SS:  {:#x}\n", stack_frame.stack_segment);
    kernel::panic!("Division Error");
}

/// Debug Exception (#DB)
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Debug (#DB)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Debug Exception");
}

/// Non-Maskable Interrupt
pub extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Non-Maskable Interrupt\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("NMI");
}

/// Breakpoint (#BP)
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Breakpoint (#BP)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    // Don't panic on breakpoint, just report it
}

/// Overflow (#OF)
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Overflow (#OF)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Overflow Exception");
}

/// Bound Range Exceeded (#BR)
pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Bound Range Exceeded (#BR)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Bound Range Exceeded");
}

/// Invalid Opcode (#UD)
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Invalid Opcode (#UD)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Invalid Opcode");
}

/// Device Not Available (#NM)
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Device Not Available (#NM)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Device Not Available");
}

/// Double Fault (#DF)
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    kernel::printk!("\n[EXCEPTION] Double Fault (#DF)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::printk!("CS:  {:#x}\n", stack_frame.code_segment);
    kernel::printk!("RFLAGS: {:#x}\n", stack_frame.cpu_flags);
    kernel::printk!("RSP: {:#x}\n", stack_frame.stack_pointer.as_u64());
    kernel::printk!("SS:  {:#x}\n", stack_frame.stack_segment);
    kernel::panic!("Double Fault");
}

/// Invalid TSS (#TS)
pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kernel::printk!("\n[EXCEPTION] Invalid TSS (#TS)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Invalid TSS");
}

/// Segment Not Present (#NP)
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kernel::printk!("\n[EXCEPTION] Segment Not Present (#NP)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Segment Not Present");
}

/// Stack-Segment Fault (#SS)
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kernel::printk!("\n[EXCEPTION] Stack-Segment Fault (#SS)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Stack-Segment Fault");
}

/// General Protection Fault (#GP)
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kernel::printk!("\n[EXCEPTION] General Protection Fault (#GP)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::printk!("CS:  {:#x}\n", stack_frame.code_segment);
    kernel::printk!("RFLAGS: {:#x}\n", stack_frame.cpu_flags);
    kernel::printk!("RSP: {:#x}\n", stack_frame.stack_pointer.as_u64());
    kernel::printk!("SS:  {:#x}\n", stack_frame.stack_segment);
    kernel::panic!("General Protection Fault");
}

/// Page Fault (#PF)
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    use crate::long_mode::read_cr2;

    let cr2 = read_cr2();
    kernel::printk!("\n[EXCEPTION] Page Fault (#PF)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("  Present:   {}\n", error_code & 0x1 != 0);
    kernel::printk!("  Write:     {}\n", error_code & 0x2 != 0);
    kernel::printk!("  User:      {}\n", error_code & 0x4 != 0);
    kernel::printk!("  Reserved:  {}\n", error_code & 0x8 != 0);
    kernel::printk!("  Inst Fetch:{}\n", error_code & 0x10 != 0);
    kernel::printk!("Address (CR2): {:#x}\n", cr2);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Page Fault");
}

/// x87 Floating-Point Exception (#MF)
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] x87 Floating-Point (#MF)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("x87 Floating-Point Exception");
}

/// Alignment Check (#AC)
pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kernel::printk!("\n[EXCEPTION] Alignment Check (#AC)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Alignment Check");
}

/// Machine Check (#MC)
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    kernel::printk!("\n[EXCEPTION] Machine Check (#MC)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Machine Check");
}

/// SIMD Floating-Point Exception (#XM)
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] SIMD Floating-Point (#XM)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("SIMD Floating-Point Exception");
}

/// Virtualization Exception (#VE)
pub extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    kernel::printk!("\n[EXCEPTION] Virtualization (#VE)\n");
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Virtualization Exception");
}

/// Security Exception (#SX)
pub extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kernel::printk!("\n[EXCEPTION] Security Exception (#SX)\n");
    kernel::printk!("Error Code: {:#x}\n", error_code);
    kernel::printk!("RIP: {:#x}\n", stack_frame.instruction_pointer.as_u64());
    kernel::panic!("Security Exception");
}
