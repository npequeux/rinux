//! Real-Time Clock (RTC) Driver
//!
//! CMOS RTC driver for reading date and time.

use rinux_arch_x86::io::{inb, outb};
use spin::Mutex;

/// CMOS address port
const CMOS_ADDRESS: u16 = 0x70;
/// CMOS data port
const CMOS_DATA: u16 = 0x71;

/// RTC registers
const RTC_SECONDS: u8 = 0x00;
const RTC_MINUTES: u8 = 0x02;
const RTC_HOURS: u8 = 0x04;
const RTC_DAY: u8 = 0x07;
const RTC_MONTH: u8 = 0x08;
const RTC_YEAR: u8 = 0x09;
const RTC_STATUS_A: u8 = 0x0A;
const RTC_STATUS_B: u8 = 0x0B;

/// Global RTC lock
static RTC: Mutex<()> = Mutex::new(());

/// Date and time structure
#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Default for DateTime {
    fn default() -> Self {
        Self::new()
    }
}

impl DateTime {
    /// Create a new DateTime
    pub const fn new() -> Self {
        Self {
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

/// Read a CMOS register
///
/// # Safety
///
/// Performs I/O port operations. Caller must hold the RTC lock.
unsafe fn read_cmos(reg: u8) -> u8 {
    outb(CMOS_ADDRESS, reg);
    inb(CMOS_DATA)
}

/// Check if RTC update is in progress
///
/// # Safety
///
/// Performs I/O port operations.
unsafe fn is_update_in_progress() -> bool {
    read_cmos(RTC_STATUS_A) & 0x80 != 0
}

/// Wait for RTC update to complete
///
/// # Safety
///
/// Performs I/O port operations.
unsafe fn wait_for_update() {
    while is_update_in_progress() {
        core::hint::spin_loop();
    }
}

/// Convert BCD to binary
fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Read current date and time from RTC
///
/// Returns None if the RTC is not accessible or values are invalid.
pub fn read_datetime() -> Option<DateTime> {
    let _lock = RTC.lock();

    unsafe {
        // Wait for any update to complete
        wait_for_update();

        // Read all values
        let second = read_cmos(RTC_SECONDS);
        let minute = read_cmos(RTC_MINUTES);
        let hour = read_cmos(RTC_HOURS);
        let day = read_cmos(RTC_DAY);
        let month = read_cmos(RTC_MONTH);
        let year = read_cmos(RTC_YEAR);

        // Check format (BCD or binary)
        let status_b = read_cmos(RTC_STATUS_B);
        let is_bcd = (status_b & 0x04) == 0;

        // Convert from BCD if needed
        let second = if is_bcd {
            bcd_to_binary(second)
        } else {
            second
        };
        let minute = if is_bcd {
            bcd_to_binary(minute)
        } else {
            minute
        };
        let hour = if is_bcd { bcd_to_binary(hour) } else { hour };
        let day = if is_bcd { bcd_to_binary(day) } else { day };
        let month = if is_bcd { bcd_to_binary(month) } else { month };
        let year = if is_bcd { bcd_to_binary(year) } else { year };

        // Calculate full year (assume 21st century for now)
        let full_year = 2000 + year as u16;

        // Basic validation
        if month == 0
            || month > 12
            || day == 0
            || day > 31
            || hour > 23
            || minute > 59
            || second > 59
        {
            return None;
        }

        Some(DateTime {
            year: full_year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }
}

/// Initialize RTC
pub fn init() {
    // Just verify we can read from the RTC
    if let Some(dt) = read_datetime() {
        rinux_kernel::printk!(
            "[RTC] Initialized - Current time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}\n",
            dt.year,
            dt.month,
            dt.day,
            dt.hour,
            dt.minute,
            dt.second
        );
    } else {
        rinux_kernel::printk!("[RTC] Warning: Could not read RTC\n");
    }
}
