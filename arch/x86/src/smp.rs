//! Symmetric Multiprocessing (SMP) Support
//!
//! Multi-core CPU initialization and management.

use crate::apic;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Maximum number of CPUs supported
pub const MAX_CPUS: usize = 256;

/// Per-CPU data structure
#[repr(C)]
pub struct CpuInfo {
    pub id: u32,
    pub apic_id: u32,
    pub online: AtomicBool,
    pub started: AtomicBool,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuInfo {
    pub const fn new() -> Self {
        Self {
            id: 0,
            apic_id: 0,
            online: AtomicBool::new(false),
            started: AtomicBool::new(false),
        }
    }
}

/// Global CPU information array
static mut CPUS: [CpuInfo; MAX_CPUS] = [const { CpuInfo::new() }; MAX_CPUS];

/// Number of detected CPUs
static CPU_COUNT: AtomicU32 = AtomicU32::new(0);

/// Bootstrap processor (BSP) ID
static BSP_ID: AtomicU32 = AtomicU32::new(0);

/// Get number of CPUs
pub fn cpu_count() -> u32 {
    CPU_COUNT.load(Ordering::Acquire)
}

/// Get BSP CPU ID
pub fn bsp_id() -> u32 {
    BSP_ID.load(Ordering::Acquire)
}

/// Get current CPU ID
pub fn current_cpu_id() -> u32 {
    apic::get_id()
}

/// Check if current CPU is BSP
pub fn is_bsp() -> bool {
    current_cpu_id() == bsp_id()
}

/// Register a CPU
fn register_cpu(apic_id: u32) -> Option<u32> {
    let count = CPU_COUNT.fetch_add(1, Ordering::AcqRel);
    if count >= MAX_CPUS as u32 {
        rinux_kernel::printk!("[SMP] Too many CPUs (max {})\n", MAX_CPUS);
        return None;
    }

    unsafe {
        CPUS[count as usize].id = count;
        CPUS[count as usize].apic_id = apic_id;
        CPUS[count as usize].online.store(false, Ordering::Release);
        CPUS[count as usize].started.store(false, Ordering::Release);
    }

    Some(count)
}

/// Mark CPU as online
pub fn set_cpu_online(cpu_id: u32, online: bool) {
    if cpu_id < MAX_CPUS as u32 {
        unsafe {
            CPUS[cpu_id as usize]
                .online
                .store(online, Ordering::Release);
        }
    }
}

/// Check if CPU is online
pub fn is_cpu_online(cpu_id: u32) -> bool {
    if cpu_id < MAX_CPUS as u32 {
        unsafe { CPUS[cpu_id as usize].online.load(Ordering::Acquire) }
    } else {
        false
    }
}

/// Detect CPUs via ACPI MADT table
fn detect_cpus_acpi() -> u32 {
    // TODO: Parse ACPI MADT (Multiple APIC Description Table)
    // For now, just return 1 (BSP only)
    rinux_kernel::printk!("[SMP] ACPI MADT parsing not yet implemented\n");
    1
}

/// Detect CPUs via CPUID
fn detect_cpus_cpuid() -> u32 {
    use crate::cpu::cpuid;

    // Check for HTT (Hyper-Threading Technology)
    let (_, ebx, _, edx) = cpuid(1);

    if (edx & (1 << 28)) != 0 {
        // HTT is supported, get logical processor count
        let logical_count = (ebx >> 16) & 0xFF;
        rinux_kernel::printk!("[SMP] CPUID reports {} logical processors\n", logical_count);
        logical_count
    } else {
        1
    }
}

/// Send INIT IPI to a CPU
fn send_init_ipi(apic_id: u32) {
    use crate::apic::{reg, write_register};

    // Set destination
    write_register(reg::ICR_HIGH, apic_id << 24);

    // Send INIT IPI
    write_register(reg::ICR_LOW, 0x00C500);

    // Wait for delivery
    while (read_register(reg::ICR_LOW) & (1 << 12)) != 0 {
        core::hint::spin_loop();
    }
}

/// Send STARTUP IPI to a CPU
fn send_startup_ipi(apic_id: u32, vector: u8) {
    use crate::apic::{reg, write_register};

    // Set destination
    write_register(reg::ICR_HIGH, apic_id << 24);

    // Send STARTUP IPI
    let command = 0x00C600 | (vector as u32);
    write_register(reg::ICR_LOW, command);

    // Wait for delivery
    while (read_register(reg::ICR_LOW) & (1 << 12)) != 0 {
        core::hint::spin_loop();
    }
}

/// Application Processor (AP) entry point
#[allow(dead_code)]
extern "C" fn ap_entry() -> ! {
    // Initialize APIC for this CPU
    apic::init();

    // Get our APIC ID
    let apic_id = apic::get_id();
    rinux_kernel::printk!("[SMP] AP {} started\n", apic_id);

    // Mark ourselves as online
    // (Find our CPU ID from APIC ID)
    for i in 0..cpu_count() {
        unsafe {
            if CPUS[i as usize].apic_id == apic_id {
                CPUS[i as usize].started.store(true, Ordering::Release);
                CPUS[i as usize].online.store(true, Ordering::Release);
                break;
            }
        }
    }

    // Idle loop - halt until next interrupt
    crate::halt()
}

/// Start an Application Processor
#[allow(dead_code)]
fn start_ap(cpu_id: u32) -> bool {
    unsafe {
        let apic_id = CPUS[cpu_id as usize].apic_id;

        rinux_kernel::printk!("[SMP] Starting AP {} (APIC ID: {})\n", cpu_id, apic_id);

        // TODO: Setup trampoline code in low memory
        // For now, we can't actually start APs without proper setup

        // Send INIT IPI
        send_init_ipi(apic_id);

        // Wait 10ms
        crate::timers::delay_ms(10);

        // Send STARTUP IPI (twice as per Intel spec)
        let vector = 0x08; // Trampoline at 0x8000
        send_startup_ipi(apic_id, vector);
        crate::timers::delay_us(200);
        send_startup_ipi(apic_id, vector);

        // Wait for AP to start
        for _ in 0..100 {
            if CPUS[cpu_id as usize].started.load(Ordering::Acquire) {
                rinux_kernel::printk!("[SMP] AP {} started successfully\n", cpu_id);
                return true;
            }
            crate::timers::delay_ms(10);
        }

        rinux_kernel::printk!("[SMP] AP {} failed to start\n", cpu_id);
        false
    }
}

/// Initialize SMP support
pub fn init() {
    rinux_kernel::printk!("[SMP] Initializing multi-core support...\n");

    // Get BSP APIC ID
    let bsp_apic_id = apic::get_id();
    BSP_ID.store(bsp_apic_id, Ordering::Release);

    // Register BSP
    if let Some(cpu_id) = register_cpu(bsp_apic_id) {
        set_cpu_online(cpu_id, true);
        rinux_kernel::printk!(
            "[SMP] BSP registered as CPU {} (APIC ID: {})\n",
            cpu_id,
            bsp_apic_id
        );
    }

    // Detect additional CPUs
    let detected = detect_cpus_cpuid();
    rinux_kernel::printk!("[SMP] Detected {} CPU(s)\n", detected);

    // Try ACPI detection for more accurate info
    let _acpi_count = detect_cpus_acpi();

    if detected > 1 {
        rinux_kernel::printk!("[SMP] Multi-core detected, but AP startup not yet implemented\n");
        rinux_kernel::printk!("[SMP] Trampoline code and memory setup required\n");
        // TODO: Start APs
        // for cpu_id in 1..detected {
        //     start_ap(cpu_id);
        // }
    }

    rinux_kernel::printk!("[SMP] Online CPUs: {}\n", cpu_count());
}

// Helper function to read APIC register (needed for IPI functions)
use crate::apic::read_register;
