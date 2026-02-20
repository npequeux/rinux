//! Interrupt Descriptor Table
//!
//! IDT setup and management.

use core::arch::asm;
use spin::Mutex;

/// IDT entry
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_middle: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    const fn null() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_middle: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    fn new(handler: u64, selector: u16, type_attr: u8) -> Self {
        IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector,
            ist: 0,
            type_attr,
            offset_middle: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFFFFFF) as u32,
            zero: 0,
        }
    }
}

/// IDT pointer
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

/// IDT
struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    const fn new() -> Self {
        Idt {
            entries: [IdtEntry::null(); 256],
        }
    }

    fn set_handler(&mut self, index: u8, handler: u64) {
        self.entries[index as usize] = IdtEntry::new(handler, 0x08, 0x8E);
    }

    fn pointer(&self) -> IdtPointer {
        IdtPointer {
            limit: (core::mem::size_of::<Self>() - 1) as u16,
            base: self.entries.as_ptr() as u64,
        }
    }
}

static IDT: Mutex<Idt> = Mutex::new(Idt::new());

/// Initialize IDT
pub fn init() {
    let mut idt = IDT.lock();

    // Set up exception handlers
    idt.set_handler(0, divide_by_zero_handler as u64);
    idt.set_handler(1, debug_handler as u64);
    idt.set_handler(2, nmi_handler as u64);
    idt.set_handler(3, breakpoint_handler as u64);
    idt.set_handler(4, overflow_handler as u64);
    idt.set_handler(5, bound_range_exceeded_handler as u64);
    idt.set_handler(6, invalid_opcode_handler as u64);
    idt.set_handler(7, device_not_available_handler as u64);
    idt.set_handler(8, double_fault_handler as u64);
    idt.set_handler(10, invalid_tss_handler as u64);
    idt.set_handler(11, segment_not_present_handler as u64);
    idt.set_handler(12, stack_segment_fault_handler as u64);
    idt.set_handler(13, general_protection_fault_handler as u64);
    idt.set_handler(14, page_fault_handler as u64);
    idt.set_handler(16, fpu_fault_handler as u64);
    idt.set_handler(17, alignment_check_handler as u64);
    idt.set_handler(18, machine_check_handler as u64);
    idt.set_handler(19, simd_exception_handler as u64);
    idt.set_handler(20, virtualization_exception_handler as u64);

    let pointer = idt.pointer();

    unsafe {
        asm!(
            "lidt [{}]",
            in(reg) &pointer,
            options(readonly, nostack)
        );
    }
}

// Exception handler type for handlers without error code
type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);

// Exception handler type for handlers with error code
type HandlerFuncWithErrCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);

/// Interrupt stack frame
#[repr(C)]
struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

// Exception handlers
extern "x86-interrupt" fn divide_by_zero_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: Divide by zero");
}

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: Debug");
}

extern "x86-interrupt" fn nmi_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NMI");
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // Breakpoint - can be non-fatal
}

extern "x86-interrupt" fn overflow_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: Overflow");
}

extern "x86-interrupt" fn bound_range_exceeded_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: Bound range exceeded");
}

extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: Invalid opcode");
}

extern "x86-interrupt" fn device_not_available_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: Device not available");
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: Double fault");
}

extern "x86-interrupt" fn invalid_tss_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("EXCEPTION: Invalid TSS");
}

extern "x86-interrupt" fn segment_not_present_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: Segment not present");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: Stack segment fault");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: General protection fault");
}

extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("EXCEPTION: Page fault");
}

extern "x86-interrupt" fn fpu_fault_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: FPU fault");
}

extern "x86-interrupt" fn alignment_check_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: Alignment check");
}

extern "x86-interrupt" fn machine_check_handler(_stack_frame: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: Machine check");
}

extern "x86-interrupt" fn simd_exception_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD exception");
}

extern "x86-interrupt" fn virtualization_exception_handler(_stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: Virtualization exception");
}
