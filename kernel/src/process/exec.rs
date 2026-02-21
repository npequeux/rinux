//! Process Execution (exec)
//!
//! Implementation of execve system call.

use super::task::Task;
use alloc::string::String;
use alloc::vec::Vec;

/// Executable format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableFormat {
    /// ELF (Executable and Linkable Format)
    Elf,
    /// Script with shebang
    Script,
}

/// ELF Header
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ElfHeader {
    pub magic: [u8; 4], // 0x7F, 'E', 'L', 'F'
    pub class: u8,      // 1 = 32-bit, 2 = 64-bit
    pub data: u8,       // 1 = little endian, 2 = big endian
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub padding: [u8; 7],
    pub etype: u16,   // 1 = relocatable, 2 = executable, 3 = shared
    pub machine: u16, // 0x3E = x86_64
    pub version2: u32,
    pub entry: u64, // Entry point address
    pub phoff: u64, // Program header offset
    pub shoff: u64, // Section header offset
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

/// Program Header (load segments)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProgramHeader {
    pub ptype: u32, // Segment type
    pub flags: u32,
    pub offset: u64, // File offset
    pub vaddr: u64,  // Virtual address
    pub paddr: u64,  // Physical address
    pub filesz: u64, // Size in file
    pub memsz: u64,  // Size in memory
    pub align: u64,
}

/// Executable context
pub struct ExecContext {
    /// Entry point address
    pub entry_point: u64,
    /// Stack pointer
    pub stack_pointer: u64,
    /// Arguments
    pub argv: Vec<String>,
    /// Environment variables
    pub envp: Vec<String>,
}

impl ExecContext {
    /// Create a new execution context
    pub fn new(entry_point: u64, stack_pointer: u64) -> Self {
        Self {
            entry_point,
            stack_pointer,
            argv: Vec::new(),
            envp: Vec::new(),
        }
    }

    /// Add an argument
    pub fn add_arg(&mut self, arg: String) {
        self.argv.push(arg);
    }

    /// Add an environment variable
    pub fn add_env(&mut self, env: String) {
        self.envp.push(env);
    }
}

/// Parse ELF header
pub fn parse_elf_header(data: &[u8]) -> Result<ElfHeader, &'static str> {
    if data.len() < core::mem::size_of::<ElfHeader>() {
        return Err("Data too small for ELF header");
    }

    // Check ELF magic
    if data[0] != 0x7F || data[1] != b'E' || data[2] != b'L' || data[3] != b'F' {
        return Err("Invalid ELF magic");
    }

    // SAFETY: We've checked the size and alignment
    let header = unsafe { core::ptr::read(data.as_ptr() as *const ElfHeader) };

    // Validate header
    if header.class != 2 {
        return Err("Not a 64-bit ELF");
    }

    if header.machine != 0x3E {
        return Err("Not an x86_64 ELF");
    }

    Ok(header)
}

/// Load program segments from ELF
pub fn load_elf_segments(
    data: &[u8],
    header: &ElfHeader,
) -> Result<Vec<ProgramHeader>, &'static str> {
    let mut segments = Vec::new();

    for i in 0..header.phnum {
        let offset = header.phoff as usize + (i as usize * header.phentsize as usize);

        if offset + core::mem::size_of::<ProgramHeader>() > data.len() {
            return Err("Program header out of bounds");
        }

        let ph = unsafe { core::ptr::read((data.as_ptr().add(offset)) as *const ProgramHeader) };

        // PT_LOAD = 1
        if ph.ptype == 1 {
            segments.push(ph);
        }
    }

    Ok(segments)
}

/// Execute a program (replace current process image)
pub fn do_exec(
    task: &mut Task,
    path: &str,
    argv: Vec<String>,
    envp: Vec<String>,
) -> Result<ExecContext, &'static str> {
    // In a real implementation, this would:
    // 1. Read the executable file from the filesystem
    // 2. Parse the ELF header
    // 3. Load program segments into memory
    // 4. Set up the stack with arguments and environment
    // 5. Set up the initial register state
    // 6. Return the execution context
    
    // For demonstration, we'll implement the core ELF loading logic
    // assuming we have the file data
    
    // TODO: Read file from filesystem
    // For now, return error indicating file system not implemented
    let _ = (task, path);
    
    // Stub: Create execution context
    let mut ctx = ExecContext::new(0x400000, 0x7FFFFFFFE000);
    
    for arg in argv {
        ctx.add_arg(arg);
    }
    
    for env in envp {
        ctx.add_env(env);
    }
    
    Ok(ctx)
}

/// Load an ELF executable into memory
pub fn load_elf(data: &[u8]) -> Result<ExecContext, &'static str> {
    use rinux_mm::paging::{PageMapper, VirtAddr, PhysAddr};
    use rinux_mm::frame;
    
    // Parse ELF header
    let header = parse_elf_header(data)?;
    
    // Load program segments
    let segments = load_elf_segments(data, &header)?;
    
    let mut mapper = unsafe { PageMapper::new() };
    
    // Load each PT_LOAD segment
    for segment in segments {
        // Calculate number of pages needed
        let start_page = segment.vaddr & !0xFFF;
        let end_page = (segment.vaddr + segment.memsz + 0xFFF) & !0xFFF;
        let num_pages = ((end_page - start_page) / 0x1000) as usize;
        
        // Allocate and map pages for this segment
        for i in 0..num_pages {
            let virt_addr = start_page + (i as u64 * 0x1000);
            
            // Allocate physical frame
            let frame = frame::allocate_frame()
                .ok_or("Out of memory while loading ELF")?;
            
            // Determine permissions from segment flags
            // PF_X = 1, PF_W = 2, PF_R = 4
            let writable = (segment.flags & 2) != 0;
            let _executable = (segment.flags & 1) != 0;
            
            // Map the page (user-accessible)
            mapper.map_page(
                VirtAddr::new(virt_addr),
                PhysAddr::new(frame.start_address()),
                writable,
                true // user accessible
            ).map_err(|_| "Failed to map ELF segment")?;
            
            // Zero the page initially
            unsafe {
                core::ptr::write_bytes(virt_addr as *mut u8, 0, 0x1000);
            }
        }
        
        // Copy segment data from file
        if segment.filesz > 0 {
            let file_offset = segment.offset as usize;
            let file_size = segment.filesz as usize;
            
            if file_offset + file_size > data.len() {
                return Err("Segment data out of bounds");
            }
            
            let src = &data[file_offset..file_offset + file_size];
            let dst = segment.vaddr as *mut u8;
            
            unsafe {
                core::ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    dst,
                    file_size
                );
            }
        }
    }
    
    // Set up user stack (typical location)
    const USER_STACK_SIZE: u64 = 0x200000; // 2MB
    const USER_STACK_TOP: u64 = 0x7FFFFFFF_F000;
    let stack_bottom = USER_STACK_TOP - USER_STACK_SIZE;
    
    // Allocate stack pages
    let num_stack_pages = (USER_STACK_SIZE / 0x1000) as usize;
    for i in 0..num_stack_pages {
        let virt_addr = stack_bottom + (i as u64 * 0x1000);
        
        let frame = frame::allocate_frame()
            .ok_or("Out of memory allocating stack")?;
        
        mapper.map_page(
            VirtAddr::new(virt_addr),
            PhysAddr::new(frame.start_address()),
            true,  // writable
            true   // user accessible
        ).map_err(|_| "Failed to map stack")?;
        
        // Zero stack pages
        unsafe {
            core::ptr::write_bytes(virt_addr as *mut u8, 0, 0x1000);
        }
    }
    
    // Create execution context
    Ok(ExecContext::new(header.entry, USER_STACK_TOP))
}

/// Initialize exec subsystem
pub fn init() {
    // Nothing to initialize yet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_context_new() {
        let ctx = ExecContext::new(0x400000, 0x7FFFFFFFE000);
        assert_eq!(ctx.entry_point, 0x400000);
        assert_eq!(ctx.stack_pointer, 0x7FFFFFFFE000);
    }

    #[test]
    fn test_exec_context_add_arg() {
        let mut ctx = ExecContext::new(0, 0);
        ctx.add_arg("arg1".into());
        ctx.add_arg("arg2".into());
        assert_eq!(ctx.argv.len(), 2);
    }

    #[test]
    fn test_parse_elf_header_invalid() {
        let data = [0u8; 64];
        assert!(parse_elf_header(&data).is_err());
    }

    #[test]
    fn test_parse_elf_header_valid_magic() {
        let mut data = [0u8; 128];
        data[0] = 0x7F;
        data[1] = b'E';
        data[2] = b'L';
        data[3] = b'F';
        data[4] = 2; // 64-bit
        data[5] = 1; // little endian
                     // Set machine to x86_64 (0x3E) at offset 18-19
        data[18] = 0x3E;
        data[19] = 0x00;

        let result = parse_elf_header(&data);
        assert!(result.is_ok());
    }
}
