//! Framebuffer Driver
//!
//! Generic framebuffer support for display output.

use core::ptr;
use spin::Mutex;

/// Pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    RGB888,   // 24-bit RGB
    RGBX8888, // 32-bit RGB with padding
    BGR888,   // 24-bit BGR
    BGRX8888, // 32-bit BGR with padding
}

/// Framebuffer information
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
    pub format: PixelFormat,
}

/// Framebuffer device
pub struct Framebuffer {
    info: FramebufferInfo,
    buffer: *mut u8,
}

impl Framebuffer {
    /// Create a new framebuffer
    pub fn new(info: FramebufferInfo) -> Self {
        Self {
            info,
            buffer: info.addr as *mut u8,
        }
    }

    /// Get framebuffer info
    pub fn info(&self) -> &FramebufferInfo {
        &self.info
    }

    /// Clear the framebuffer with a color
    pub fn clear(&mut self, color: u32) {
        let pixels = (self.info.height * self.info.pitch / (self.info.bpp / 8)) as usize;

        unsafe {
            match self.info.bpp {
                32 => {
                    let buf = self.buffer as *mut u32;
                    for i in 0..pixels {
                        ptr::write_volatile(buf.add(i), color);
                    }
                }
                24 => {
                    let r = ((color >> 16) & 0xFF) as u8;
                    let g = ((color >> 8) & 0xFF) as u8;
                    let b = (color & 0xFF) as u8;

                    for i in 0..pixels {
                        let offset = i * 3;
                        ptr::write_volatile(self.buffer.add(offset), b);
                        ptr::write_volatile(self.buffer.add(offset + 1), g);
                        ptr::write_volatile(self.buffer.add(offset + 2), r);
                    }
                }
                _ => {}
            }
        }
    }

    /// Draw a pixel
    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let offset = (y * self.info.pitch + x * (self.info.bpp / 8)) as usize;

        unsafe {
            match self.info.bpp {
                32 => {
                    let ptr = self.buffer.add(offset) as *mut u32;
                    ptr::write_volatile(ptr, color);
                }
                24 => {
                    let r = ((color >> 16) & 0xFF) as u8;
                    let g = ((color >> 8) & 0xFF) as u8;
                    let b = (color & 0xFF) as u8;

                    ptr::write_volatile(self.buffer.add(offset), b);
                    ptr::write_volatile(self.buffer.add(offset + 1), g);
                    ptr::write_volatile(self.buffer.add(offset + 2), r);
                }
                _ => {}
            }
        }
    }

    /// Draw a rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        for dy in 0..height {
            for dx in 0..width {
                self.put_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Fill a horizontal line
    pub fn fill_line(&mut self, y: u32, color: u32) {
        if y >= self.info.height {
            return;
        }

        let offset = (y * self.info.pitch) as usize;

        unsafe {
            match self.info.bpp {
                32 => {
                    let ptr = self.buffer.add(offset) as *mut u32;
                    for x in 0..self.info.width {
                        ptr::write_volatile(ptr.add(x as usize), color);
                    }
                }
                24 => {
                    let r = ((color >> 16) & 0xFF) as u8;
                    let g = ((color >> 8) & 0xFF) as u8;
                    let b = (color & 0xFF) as u8;

                    for x in 0..self.info.width {
                        let pixel_offset = offset + (x as usize * 3);
                        ptr::write_volatile(self.buffer.add(pixel_offset), b);
                        ptr::write_volatile(self.buffer.add(pixel_offset + 1), g);
                        ptr::write_volatile(self.buffer.add(pixel_offset + 2), r);
                    }
                }
                _ => {}
            }
        }
    }

    /// Draw a line using Bresenham's algorithm
    pub fn draw_line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, color: u32) {
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && y0 >= 0 {
                self.put_pixel(x0 as u32, y0 as u32, color);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draw a circle using midpoint circle algorithm
    pub fn draw_circle(&mut self, cx: u32, cy: u32, radius: u32, color: u32) {
        let mut x = 0i32;
        let mut y = radius as i32;
        let mut d = 3 - 2 * radius as i32;
        let cx = cx as i32;
        let cy = cy as i32;

        while y >= x {
            // Draw 8 octants
            self.put_pixel_safe(cx + x, cy + y, color);
            self.put_pixel_safe(cx - x, cy + y, color);
            self.put_pixel_safe(cx + x, cy - y, color);
            self.put_pixel_safe(cx - x, cy - y, color);
            self.put_pixel_safe(cx + y, cy + x, color);
            self.put_pixel_safe(cx - y, cy + x, color);
            self.put_pixel_safe(cx + y, cy - x, color);
            self.put_pixel_safe(cx - y, cy - x, color);

            x += 1;
            if d > 0 {
                y -= 1;
                d = d + 4 * (x - y) + 10;
            } else {
                d = d + 4 * x + 6;
            }
        }
    }

    /// Draw a filled circle
    pub fn fill_circle(&mut self, cx: u32, cy: u32, radius: u32, color: u32) {
        let cx = cx as i32;
        let cy = cy as i32;
        let radius = radius as i32;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius * radius {
                    self.put_pixel_safe(cx + x, cy + y, color);
                }
            }
        }
    }

    /// Draw a filled rectangle with border
    pub fn draw_rect_with_border(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        fill_color: u32,
        border_color: u32,
    ) {
        // Fill interior
        self.draw_rect(x, y, width, height, fill_color);

        // Draw border
        if width > 0 && height > 0 {
            // Top and bottom
            for dx in 0..width {
                self.put_pixel(x + dx, y, border_color);
                if height > 1 {
                    self.put_pixel(x + dx, y + height - 1, border_color);
                }
            }

            // Left and right
            for dy in 0..height {
                self.put_pixel(x, y + dy, border_color);
                if width > 1 {
                    self.put_pixel(x + width - 1, y + dy, border_color);
                }
            }
        }
    }

    /// Put pixel with bounds checking using signed coordinates
    fn put_pixel_safe(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as u32) < self.info.width && (y as u32) < self.info.height {
            self.put_pixel(x as u32, y as u32, color);
        }
    }

    /// Draw a simple 8x8 character (basic font)
    pub fn draw_char(&mut self, x: u32, y: u32, ch: u8, color: u32) {
        // Simple 8x8 bitmap font for basic ASCII
        let glyph = get_char_bitmap(ch);

        for dy in 0..8 {
            let row = glyph[dy as usize];
            for dx in 0..8 {
                if (row & (1 << (7 - dx))) != 0 {
                    self.put_pixel(x + dx, y + dy, color);
                }
            }
        }
    }

    /// Draw a string
    pub fn draw_string(&mut self, x: u32, y: u32, s: &str, color: u32) {
        let mut curr_x = x;
        for ch in s.bytes() {
            if ch == b'\n' {
                // Newline not supported in simple implementation
                continue;
            }
            self.draw_char(curr_x, y, ch, color);
            curr_x += 8; // Character width
            if curr_x >= self.info.width {
                break;
            }
        }
    }
}

/// Get 8x8 bitmap for a character (simplified font)
fn get_char_bitmap(ch: u8) -> &'static [u8; 8] {
    // Simplified font - only a few characters for demonstration
    match ch {
        b'A' => &[0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00],
        b'B' => &[0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
        b'C' => &[0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
        b'D' => &[0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00],
        b'E' => &[0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x7E, 0x00],
        b'F' => &[0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x00],
        b'0' => &[0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        b'1' => &[0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
        b'2' => &[0x3C, 0x66, 0x06, 0x0C, 0x18, 0x30, 0x7E, 0x00],
        b'3' => &[0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00],
        b' ' => &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        b'.' => &[0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        b'!' => &[0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00],
        _ => &[0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x00], // Box for unknown chars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_format_variants() {
        assert_ne!(PixelFormat::RGB888, PixelFormat::RGBX8888);
        assert_ne!(PixelFormat::BGR888, PixelFormat::BGRX8888);
    }

    #[test]
    fn test_framebuffer_info_creation() {
        let info = FramebufferInfo {
            addr: 0xB8000,
            width: 1920,
            height: 1080,
            pitch: 1920 * 4,
            bpp: 32,
            format: PixelFormat::RGBX8888,
        };
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
        assert_eq!(info.bpp, 32);
    }

    #[test]
    fn test_char_bitmap_exists() {
        // Test that some characters have valid bitmaps
        let bitmap_a = get_char_bitmap(b'A');
        assert_eq!(bitmap_a.len(), 8);

        let bitmap_space = get_char_bitmap(b' ');
        assert_eq!(
            bitmap_space,
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );

        // Unknown char should return box
        let bitmap_unknown = get_char_bitmap(b'~');
        assert_eq!(
            bitmap_unknown,
            &[0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x00]
        );
    }
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

/// Global framebuffer instance
static FRAMEBUFFER: Mutex<Option<Framebuffer>> = Mutex::new(None);

/// Initialize framebuffer (will be set up by bootloader or UEFI)
pub fn init() {
    rinux_kernel::printk::printk("  Framebuffer: Waiting for initialization from bootloader\n");

    // In a real implementation, this would be set up by the bootloader
    // (GRUB, UEFI) and passed to the kernel via boot parameters
    // For now, we'll just initialize an empty framebuffer
}

/// Set up framebuffer with given info
pub fn setup(info: FramebufferInfo) {
    let fb = Framebuffer::new(info);
    *FRAMEBUFFER.lock() = Some(fb);

    rinux_kernel::printk::printk("  Framebuffer initialized: ");
    // TODO: Print resolution
    rinux_kernel::printk::printk("\n");
}

/// Get framebuffer instance
pub fn get() -> Option<&'static Mutex<Option<Framebuffer>>> {
    Some(&FRAMEBUFFER)
}

/// Test framebuffer with a simple pattern
pub fn test() {
    let mut fb_lock = FRAMEBUFFER.lock();
    if let Some(ref mut fb) = *fb_lock {
        // Clear to black
        fb.clear(0x000000);

        // Draw colored bars
        let bar_height = fb.info.height / 4;
        fb.draw_rect(0, 0, fb.info.width, bar_height, 0xFF0000); // Red
        fb.draw_rect(0, bar_height, fb.info.width, bar_height, 0x00FF00); // Green
        fb.draw_rect(0, bar_height * 2, fb.info.width, bar_height, 0x0000FF); // Blue
        fb.draw_rect(0, bar_height * 3, fb.info.width, bar_height, 0xFFFFFF); // White
    }
}

/// Demo drawing primitives (example/demonstration function)
pub fn demo_primitives() {
    let mut fb_lock = FRAMEBUFFER.lock();
    if let Some(ref mut fb) = *fb_lock {
        // Clear to dark blue
        fb.clear(0x001020);

        let width = fb.info.width;
        let height = fb.info.height;

        // Draw some circles
        fb.draw_circle(width / 4, height / 4, 50, 0xFF0000); // Red circle
        fb.fill_circle(width / 2, height / 4, 40, 0x00FF00); // Green filled circle
        fb.draw_circle(3 * width / 4, height / 4, 60, 0x0000FF); // Blue circle

        // Draw some lines
        fb.draw_line(0, height / 2, width, height / 2, 0xFFFFFF); // Horizontal white line
        fb.draw_line(width / 2, 0, width / 2, height, 0xFFFF00); // Vertical yellow line

        // Draw some rectangles
        fb.draw_rect_with_border(
            width / 4 - 50,
            3 * height / 4 - 30,
            100,
            60,
            0xFF00FF,
            0xFFFFFF,
        ); // Magenta box with white border
        fb.draw_rect_with_border(
            width / 2 - 60,
            3 * height / 4 - 40,
            120,
            80,
            0x00FFFF,
            0x000000,
        ); // Cyan box with black border

        // Draw some text
        if width >= 200 && height >= 100 {
            fb.draw_string(10, 10, "Rinux Graphics Demo", 0xFFFFFF);
            fb.draw_string(10, 20, "Primitives Example", 0xFFFF00);
        }
    }
}
