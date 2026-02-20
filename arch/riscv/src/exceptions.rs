//! RISC-V Exception Handling

use crate::csr;

/// Initialize exception handling
pub fn init() {
    kernel::printk!("[RISCV] Initializing exception handling...\n");
    
    // TODO: Set up trap vector (stvec)
    // extern "C" {
    //     fn trap_vector();
    // }
    // csr::write_stvec(trap_vector as usize);
    
    kernel::printk!("[RISCV] Exception handling initialized\n");
}

/// Trap handler
#[no_mangle]
pub extern "C" fn trap_handler() {
    let scause = csr::read_scause();
    let stval = csr::read_stval();
    let sepc = csr::read_sepc();
    
    let is_interrupt = (scause & (1 << 63)) != 0;
    let code = scause & 0x7FFFFFFFFFFFFFFF;
    
    if is_interrupt {
        kernel::printk!("[RISCV] Interrupt: code={}, stval={:#x}, sepc={:#x}\n", 
                       code, stval, sepc);
    } else {
        kernel::printk!("[RISCV] Exception: code={}, stval={:#x}, sepc={:#x}\n", 
                       code, stval, sepc);
        match code {
            0 => kernel::printk!("  Instruction address misaligned\n"),
            1 => kernel::printk!("  Instruction access fault\n"),
            2 => kernel::printk!("  Illegal instruction\n"),
            3 => kernel::printk!("  Breakpoint\n"),
            4 => kernel::printk!("  Load address misaligned\n"),
            5 => kernel::printk!("  Load access fault\n"),
            6 => kernel::printk!("  Store/AMO address misaligned\n"),
            7 => kernel::printk!("  Store/AMO access fault\n"),
            8 => kernel::printk!("  Environment call from U-mode\n"),
            9 => kernel::printk!("  Environment call from S-mode\n"),
            12 => kernel::printk!("  Instruction page fault\n"),
            13 => kernel::printk!("  Load page fault\n"),
            15 => kernel::printk!("  Store/AMO page fault\n"),
            _ => kernel::printk!("  Unknown exception\n"),
        }
        kernel::panic!("Unhandled exception");
    }
}
