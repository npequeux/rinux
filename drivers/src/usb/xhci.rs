//! xHCI (eXtensible Host Controller Interface) Driver
//!
//! USB 3.0+ host controller driver.

use super::{UsbHostController, UsbSpeed};
use crate::pci::PciDevice;
use core::ptr;

/// xHCI capability registers (offset from base)
#[repr(C)]
#[derive(Debug)]
struct XhciCapRegs {
    caplength: u8, // 0x00: Capability register length
    _reserved: u8,
    hciversion: u16, // 0x02: Interface version number
    hcsparams1: u32, // 0x04: Structural parameters 1
    hcsparams2: u32, // 0x08: Structural parameters 2
    hcsparams3: u32, // 0x0C: Structural parameters 3
    hccparams1: u32, // 0x10: Capability parameters 1
    dboff: u32,      // 0x14: Doorbell offset
    rtsoff: u32,     // 0x18: Runtime register space offset
    hccparams2: u32, // 0x1C: Capability parameters 2
}

/// xHCI operational registers
#[repr(C)]
#[derive(Debug)]
struct XhciOpRegs {
    usbcmd: u32,   // 0x00: USB command
    usbsts: u32,   // 0x04: USB status
    pagesize: u32, // 0x08: Page size
    _reserved1: [u32; 2],
    dnctrl: u32,  // 0x14: Device notification control
    crcr_lo: u32, // 0x18: Command ring control (low)
    crcr_hi: u32, // 0x1C: Command ring control (high)
    _reserved2: [u32; 4],
    dcbaap_lo: u32, // 0x30: Device context base address array pointer (low)
    dcbaap_hi: u32, // 0x34: Device context base address array pointer (high)
    config: u32,    // 0x38: Configure
}

/// xHCI port register set
#[repr(C)]
#[derive(Debug)]
struct XhciPortRegs {
    portsc: u32,    // Port status and control
    portpmsc: u32,  // Port power management status and control
    portli: u32,    // Port link info
    porthlpmc: u32, // Port hardware LPM control
}

/// USB Command register bits
const USBCMD_RUN: u32 = 1 << 0;
const USBCMD_RESET: u32 = 1 << 1;
#[allow(dead_code)]
const USBCMD_INTERRUPTER_ENABLE: u32 = 1 << 2;
#[allow(dead_code)]
const USBCMD_HOST_SYSTEM_ERROR_ENABLE: u32 = 1 << 3;

/// USB Status register bits
const USBSTS_HCH: u32 = 1 << 0; // HC Halted
#[allow(dead_code)]
const USBSTS_HSE: u32 = 1 << 2; // Host System Error
#[allow(dead_code)]
const USBSTS_EINT: u32 = 1 << 3; // Event Interrupt
#[allow(dead_code)]
const USBSTS_PCD: u32 = 1 << 4; // Port Change Detect
const USBSTS_CNR: u32 = 1 << 11; // Controller Not Ready

/// Port status and control register bits
const PORTSC_CCS: u32 = 1 << 0; // Current Connect Status
#[allow(dead_code)]
const PORTSC_PED: u32 = 1 << 1; // Port Enabled/Disabled
const PORTSC_PR: u32 = 1 << 4; // Port Reset
#[allow(dead_code)]
const PORTSC_PLS_MASK: u32 = 0xF << 5; // Port Link State
#[allow(dead_code)]
const PORTSC_PP: u32 = 1 << 9; // Port Power
const PORTSC_SPEED_MASK: u32 = 0xF << 10; // Port Speed
#[allow(dead_code)]
const PORTSC_CSC: u32 = 1 << 17; // Connect Status Change
#[allow(dead_code)]
const PORTSC_PRC: u32 = 1 << 21; // Port Reset Change
#[allow(dead_code)]
const PORTSC_WPR: u32 = 1 << 31; // Warm Port Reset

/// xHCI controller
pub struct XhciController {
    #[allow(dead_code)]
    cap_regs: *mut XhciCapRegs,
    op_regs: *mut XhciOpRegs,
    port_regs: *mut XhciPortRegs,
    num_ports: u8,
    #[allow(dead_code)]
    base_addr: u64,
}

impl XhciController {
    /// Create a new xHCI controller from PCI device
    pub fn new(pci_dev: &PciDevice) -> Result<Self, &'static str> {
        // Get BAR0 (memory mapped registers)
        let bar0 = pci_dev.bars[0];

        if bar0 == 0 || (bar0 & 1) == 1 {
            return Err("Invalid BAR0");
        }

        // Clear bits to get base address
        let base_addr = (bar0 & !0xF) as u64;

        // For 64-bit BAR, combine with BAR1
        let base_addr = if (bar0 & 0x4) != 0 {
            base_addr | ((pci_dev.bars[1] as u64) << 32)
        } else {
            base_addr
        };

        if base_addr == 0 {
            return Err("Invalid base address");
        }

        let cap_regs = base_addr as *mut XhciCapRegs;

        unsafe {
            let caplength = ptr::read_volatile(&(*cap_regs).caplength);
            let op_regs = (base_addr + caplength as u64) as *mut XhciOpRegs;

            let hcsparams1 = ptr::read_volatile(&(*cap_regs).hcsparams1);
            let num_ports = (hcsparams1 >> 24) as u8;

            let port_regs = (op_regs as u64 + 0x400) as *mut XhciPortRegs;

            Ok(Self {
                cap_regs,
                op_regs,
                port_regs,
                num_ports,
                base_addr,
            })
        }
    }

    /// Read operational register
    unsafe fn read_op_reg(&self, offset: usize) -> u32 {
        ptr::read_volatile((self.op_regs as *const u8).add(offset) as *const u32)
    }

    /// Write operational register
    unsafe fn write_op_reg(&mut self, offset: usize, value: u32) {
        ptr::write_volatile((self.op_regs as *mut u8).add(offset) as *mut u32, value);
    }

    /// Read port register
    unsafe fn read_port_reg(&self, port: u8, offset: usize) -> u32 {
        if port >= self.num_ports {
            return 0;
        }
        let port_base = (self.port_regs as usize) + (port as usize * 0x10);
        ptr::read_volatile((port_base + offset) as *const u32)
    }

    /// Write port register
    unsafe fn write_port_reg(&mut self, port: u8, offset: usize, value: u32) {
        if port >= self.num_ports {
            return;
        }
        let port_base = (self.port_regs as usize) + (port as usize * 0x10);
        ptr::write_volatile((port_base + offset) as *mut u32, value);
    }

    /// Wait for controller to be ready
    fn wait_ready(&self) -> Result<(), &'static str> {
        for _ in 0..1000 {
            unsafe {
                let status = self.read_op_reg(0x04); // USBSTS
                if (status & USBSTS_CNR) == 0 {
                    return Ok(());
                }
            }
            // Small delay
            for _ in 0..10000 {
                core::hint::spin_loop();
            }
        }
        Err("Controller not ready timeout")
    }

    /// Get port speed
    fn get_port_speed(&self, port: u8) -> UsbSpeed {
        unsafe {
            let portsc = self.read_port_reg(port, 0);
            let speed = ((portsc & PORTSC_SPEED_MASK) >> 10) as u8;

            match speed {
                1 => UsbSpeed::Full,
                2 => UsbSpeed::Low,
                3 => UsbSpeed::High,
                4 => UsbSpeed::Super,
                5 => UsbSpeed::SuperPlus,
                _ => UsbSpeed::Full,
            }
        }
    }
}

impl UsbHostController for XhciController {
    fn init(&mut self) -> Result<(), &'static str> {
        rinux_kernel::printk::printk("    Initializing xHCI controller...\n");

        // Wait for controller to be ready
        self.wait_ready()?;

        Ok(())
    }

    fn reset(&mut self) -> Result<(), &'static str> {
        unsafe {
            // Stop the controller
            let mut cmd = self.read_op_reg(0x00); // USBCMD
            cmd &= !USBCMD_RUN;
            self.write_op_reg(0x00, cmd);

            // Wait for halt
            for _ in 0..1000 {
                let status = self.read_op_reg(0x04); // USBSTS
                if (status & USBSTS_HCH) != 0 {
                    break;
                }
                for _ in 0..1000 {
                    core::hint::spin_loop();
                }
            }

            // Reset the controller
            cmd = self.read_op_reg(0x00);
            cmd |= USBCMD_RESET;
            self.write_op_reg(0x00, cmd);

            // Wait for reset to complete
            for _ in 0..1000 {
                let cmd = self.read_op_reg(0x00);
                if (cmd & USBCMD_RESET) == 0 {
                    return Ok(());
                }
                for _ in 0..1000 {
                    core::hint::spin_loop();
                }
            }
        }

        Err("Reset timeout")
    }

    fn port_count(&self) -> u8 {
        self.num_ports
    }

    fn port_connected(&self, port: u8) -> bool {
        unsafe {
            let portsc = self.read_port_reg(port, 0);
            (portsc & PORTSC_CCS) != 0
        }
    }

    fn reset_port(&mut self, port: u8) -> Result<(), &'static str> {
        unsafe {
            let mut portsc = self.read_port_reg(port, 0);
            portsc |= PORTSC_PR;
            self.write_port_reg(port, 0, portsc);

            // Wait for reset to complete
            for _ in 0..1000 {
                let portsc = self.read_port_reg(port, 0);
                if (portsc & PORTSC_PR) == 0 {
                    return Ok(());
                }
                for _ in 0..1000 {
                    core::hint::spin_loop();
                }
            }
        }

        Err("Port reset timeout")
    }

    fn enumerate_devices(&mut self) -> usize {
        let mut count = 0;

        rinux_kernel::printk::printk("    Enumerating USB devices...\n");

        for port in 0..self.num_ports {
            if self.port_connected(port) {
                rinux_kernel::printk::printk("      Port ");
                // TODO: Print port number
                rinux_kernel::printk::printk(": Device connected (");

                let speed = self.get_port_speed(port);
                match speed {
                    UsbSpeed::Low => rinux_kernel::printk::printk("Low Speed"),
                    UsbSpeed::Full => rinux_kernel::printk::printk("Full Speed"),
                    UsbSpeed::High => rinux_kernel::printk::printk("High Speed"),
                    UsbSpeed::Super => rinux_kernel::printk::printk("Super Speed"),
                    UsbSpeed::SuperPlus => rinux_kernel::printk::printk("Super Speed+"),
                }

                rinux_kernel::printk::printk(")\n");

                // Register device with device manager
                unsafe {
                    if let Some(_address) = super::device::device_manager_mut()
                        .register_device(port, speed)
                    {
                        rinux_kernel::printk::printk("        Assigned address: ");
                        // TODO: Print address
                        rinux_kernel::printk::printk("\n");
                    }
                }

                count += 1;
            }
        }

        count
    }
}

/// Initialize an xHCI controller
pub fn init_controller(pci_dev: &PciDevice) -> Result<(), &'static str> {
    // Enable bus mastering and memory space
    pci_dev.enable_bus_mastering();
    pci_dev.enable_memory_space();

    let mut controller = XhciController::new(pci_dev)?;

    rinux_kernel::printk::printk("    xHCI version: ");
    // TODO: Print version
    rinux_kernel::printk::printk("\n");

    rinux_kernel::printk::printk("    Ports: ");
    // TODO: Print port count
    rinux_kernel::printk::printk("\n");

    // Reset controller
    controller.reset()?;

    // Initialize controller
    controller.init()?;

    // Enumerate devices
    let device_count = controller.enumerate_devices();

    if device_count > 0 {
        rinux_kernel::printk::printk("    Found ");
        // TODO: Print device count
        rinux_kernel::printk::printk(" USB devices\n");
    }

    Ok(())
}
