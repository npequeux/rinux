//! VGA Driver
//!
//! VGA text mode driver with scrolling, cursor, and color support.

use core::fmt;
use spin::Mutex;

/// VGA buffer dimensions
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

/// VGA buffer address
const VGA_BUFFER_ADDR: usize = 0xB8000;

/// Tab width in characters
const TAB_WIDTH: usize = 4;

/// VGA color codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// VGA color attribute (foreground + background)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// VGA character (character + color)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// VGA text buffer
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT],
}

/// VGA writer
pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Create a new writer
    fn new() -> Writer {
        Writer {
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) },
        }
    }

    /// Write a byte to the screen
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.column_position = 0,
            b'\t' => {
                // Tab: align to next multiple of TAB_WIDTH
                let spaces = TAB_WIDTH - (self.column_position % TAB_WIDTH);
                for _ in 0..spaces {
                    self.write_byte(b' ');
                }
            }
            byte => {
                if self.column_position >= VGA_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;
            }
        }
    }

    /// Write a string to the screen
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII or newline
                0x20..=0x7e | b'\n' | b'\r' | b'\t' => self.write_byte(byte),
                // Non-printable: write a box character
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Move to a new line
    fn new_line(&mut self) {
        self.column_position = 0;
        
        if self.row_position < VGA_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scroll up
            self.scroll_up();
        }
    }

    /// Scroll the screen up by one line
    fn scroll_up(&mut self) {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                self.buffer.chars[row - 1][col] = self.buffer.chars[row][col];
            }
        }
        
        // Clear the last line
        self.clear_row(VGA_HEIGHT - 1);
        self.row_position = VGA_HEIGHT - 1;
    }

    /// Clear a row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..VGA_WIDTH {
            self.buffer.chars[row][col] = blank;
        }
    }

    /// Clear the screen
    pub fn clear_screen(&mut self) {
        for row in 0..VGA_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 0;
    }

    /// Set foreground and background colors
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    /// Update hardware cursor position
    pub fn update_cursor(&self) {
        let pos = self.row_position * VGA_WIDTH + self.column_position;
        
        unsafe {
            // Write cursor location low byte
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x3D4u16,
                in("al") 0x0Fu8,
                options(nomem, nostack)
            );
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x3D5u16,
                in("al") (pos & 0xFF) as u8,
                options(nomem, nostack)
            );
            
            // Write cursor location high byte
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x3D4u16,
                in("al") 0x0Eu8,
                options(nomem, nostack)
            );
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x3D5u16,
                in("al") ((pos >> 8) & 0xFF) as u8,
                options(nomem, nostack)
            );
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Global VGA writer
static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

/// Initialize VGA
pub fn init() {
    let writer = Writer::new();
    let mut lock = WRITER.lock();
    *lock = Some(writer);
    
    if let Some(ref mut w) = *lock {
        w.clear_screen();
        w.update_cursor();
    }
}

/// Write to VGA
pub fn write_str(s: &str) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.write_string(s);
        writer.update_cursor();
    }
}

/// Write formatted string to VGA
pub fn write_fmt(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.write_fmt(args).unwrap();
        writer.update_cursor();
    }
}

/// Clear the screen
pub fn clear_screen() {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.clear_screen();
        writer.update_cursor();
    }
}

/// Set VGA colors
pub fn set_color(foreground: Color, background: Color) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.set_color(foreground, background);
    }
}
