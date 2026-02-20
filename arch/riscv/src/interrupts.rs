//! RISC-V Interrupt Management

use crate::csr;

/// Initialize interrupts
pub fn init() {
    kernel::printk!("[RISCV] Initializing interrupts...\n");
    
    // Enable supervisor external interrupts
    let mut sie = csr::read_sie();
    sie |= (1 << 9); // SEIE - Supervisor external interrupt enable
    sie |= (1 << 5); // STIE - Supervisor timer interrupt enable
    sie |= (1 << 1); // SSIE - Supervisor software interrupt enable
    csr::write_sie(sie);
    
    crate::enable_interrupts();
    
    kernel::printk!("[RISCV] Interrupts initialized\n");
}
