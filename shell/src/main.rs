//! Simple Shell
//!
//! A basic command-line shell for Rinux.

/// Shell state
struct Shell {
    running: bool,
    cwd: [u8; 256],
}

impl Shell {
    fn new() -> Self {
        let mut cwd = [0u8; 256];
        cwd[0] = b'/';
        
        Self {
            running: true,
            cwd,
        }
    }

    fn run(&mut self) {
        syscall_write(1, b"Rinux Shell v0.1\n");
        syscall_write(1, b"Type 'help' for available commands\n\n");

        while self.running {
            // Print prompt
            self.print_prompt();

            // Read command
            let mut input = [0u8; 256];
            let len = self.read_line(&mut input);

            if len > 0 {
                self.execute_command(&input[..len]);
            }
        }
    }

    fn print_prompt(&self) {
        syscall_write(1, b"rinux:");
        
        // Print current directory
        let mut i = 0;
        while i < 256 && self.cwd[i] != 0 {
            syscall_write(1, &self.cwd[i..i+1]);
            i += 1;
        }
        
        syscall_write(1, b"$ ");
    }

    fn read_line(&self, buf: &mut [u8]) -> usize {
        // TODO: Implement actual keyboard input reading
        // For now, simulate a command
        let cmd = b"help";
        let len = cmd.len().min(buf.len());
        buf[..len].copy_from_slice(&cmd[..len]);
        syscall_write(1, cmd);
        syscall_write(1, b"\n");
        len
    }

    fn execute_command(&mut self, cmd: &[u8]) {
        // Parse command
        let cmd_str = core::str::from_utf8(cmd).unwrap_or("");
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();

        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => self.cmd_help(),
            "exit" => self.cmd_exit(),
            "pwd" => self.cmd_pwd(),
            "cd" => self.cmd_cd(parts.get(1).unwrap_or(&"/")),
            "ls" => self.cmd_ls(parts.get(1).unwrap_or(&".")),
            "cat" => {
                if parts.len() > 1 {
                    self.cmd_cat(parts[1]);
                } else {
                    syscall_write(1, b"cat: missing file argument\n");
                }
            }
            "echo" => self.cmd_echo(&parts[1..]),
            "clear" => self.cmd_clear(),
            _ => {
                syscall_write(1, b"Unknown command: ");
                syscall_write(1, parts[0].as_bytes());
                syscall_write(1, b"\n");
            }
        }
    }

    fn cmd_help(&self) {
        syscall_write(1, b"Available commands:\n");
        syscall_write(1, b"  help   - Show this help message\n");
        syscall_write(1, b"  exit   - Exit the shell\n");
        syscall_write(1, b"  pwd    - Print working directory\n");
        syscall_write(1, b"  cd     - Change directory\n");
        syscall_write(1, b"  ls     - List directory contents\n");
        syscall_write(1, b"  cat    - Display file contents\n");
        syscall_write(1, b"  echo   - Print text\n");
        syscall_write(1, b"  clear  - Clear screen\n");
    }

    fn cmd_exit(&mut self) {
        syscall_write(1, b"Exiting shell...\n");
        self.running = false;
        syscall_exit(0);
    }

    fn cmd_pwd(&self) {
        let mut i = 0;
        while i < 256 && self.cwd[i] != 0 {
            syscall_write(1, &self.cwd[i..i+1]);
            i += 1;
        }
        syscall_write(1, b"\n");
    }

    fn cmd_cd(&mut self, path: &str) {
        // TODO: Implement actual directory changing
        syscall_write(1, b"cd: not yet implemented\n");
    }

    fn cmd_ls(&self, path: &str) {
        // TODO: Implement directory listing
        syscall_write(1, b"ls: not yet implemented\n");
    }

    fn cmd_cat(&self, path: &str) {
        // TODO: Implement file reading
        syscall_write(1, b"cat: not yet implemented\n");
    }

    fn cmd_echo(&self, args: &[&str]) {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                syscall_write(1, b" ");
            }
            syscall_write(1, arg.as_bytes());
        }
        syscall_write(1, b"\n");
    }

    fn cmd_clear(&self) {
        // ANSI escape code to clear screen
        syscall_write(1, b"\x1b[2J\x1b[H");
    }
}

/// Main entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut shell = Shell::new();
    shell.run();
    
    // Should not reach here
    syscall_exit(0);
    loop {}
}

/// System call wrappers
fn syscall_write(fd: usize, buf: &[u8]) -> isize {
    let result: isize;
    unsafe {
        core::arch::asm!(
            "mov rax, 1",
            "mov rdi, {0}",
            "mov rsi, {1}",
            "mov rdx, {2}",
            "syscall",
            in(reg) fd,
            in(reg) buf.as_ptr(),
            in(reg) buf.len(),
            out("rax") result,
            options(nostack)
        );
    }
    result
}

fn syscall_exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "mov rax, 60",
            "mov rdi, {0}",
            "syscall",
            in(reg) code,
            options(noreturn)
        );
    }
}

/// Panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    syscall_write(1, b"Shell panicked!\n");
    loop {}
}

// Simple Vec implementation (since we can't use std)
struct Vec<T> {
    data: [Option<T>; 32],
    len: usize,
}

impl<T: Copy> Vec<T> {
    fn new() -> Self {
        Self {
            data: [None; 32],
            len: 0,
        }
    }

    fn push(&mut self, item: T) {
        if self.len < 32 {
            self.data[self.len] = Some(item);
            self.len += 1;
        }
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            self.data[index].as_ref()
        } else {
            None
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter(&self) -> VecIter<T> {
        VecIter {
            vec: self,
            index: 0,
        }
    }
}

struct VecIter<'a, T> {
    vec: &'a Vec<T>,
    index: usize,
}

impl<'a, T: Copy> Iterator for VecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let item = self.vec.data[self.index].as_ref();
            self.index += 1;
            item
        } else {
            None
        }
    }
}
