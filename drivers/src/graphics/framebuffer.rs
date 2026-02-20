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
