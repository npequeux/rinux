//! Signal Types
//!
//! Signal numbers and signal set.

/// Signal numbers (following POSIX standard)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Signal {
    /// Hangup detected on controlling terminal
    SIGHUP = 1,
    /// Interrupt from keyboard
    SIGINT = 2,
    /// Quit from keyboard
    SIGQUIT = 3,
    /// Illegal instruction
    SIGILL = 4,
    /// Trace/breakpoint trap
    SIGTRAP = 5,
    /// Abort signal
    SIGABRT = 6,
    /// Bus error
    SIGBUS = 7,
    /// Floating point exception
    SIGFPE = 8,
    /// Kill signal
    SIGKILL = 9,
    /// User-defined signal 1
    SIGUSR1 = 10,
    /// Invalid memory reference
    SIGSEGV = 11,
    /// User-defined signal 2
    SIGUSR2 = 12,
    /// Broken pipe
    SIGPIPE = 13,
    /// Alarm clock
    SIGALRM = 14,
    /// Termination signal
    SIGTERM = 15,
    /// Stack fault
    SIGSTKFLT = 16,
    /// Child stopped or terminated
    SIGCHLD = 17,
    /// Continue if stopped
    SIGCONT = 18,
    /// Stop process
    SIGSTOP = 19,
    /// Stop typed at terminal
    SIGTSTP = 20,
    /// Terminal input for background process
    SIGTTIN = 21,
    /// Terminal output for background process
    SIGTTOU = 22,
    /// Urgent condition on socket
    SIGURG = 23,
    /// CPU time limit exceeded
    SIGXCPU = 24,
    /// File size limit exceeded
    SIGXFSZ = 25,
    /// Virtual alarm clock
    SIGVTALRM = 26,
    /// Profiling timer expired
    SIGPROF = 27,
    /// Window resize signal
    SIGWINCH = 28,
    /// I/O now possible
    SIGIO = 29,
    /// Power failure
    SIGPWR = 30,
    /// Bad system call
    SIGSYS = 31,
}

impl Signal {
    /// Convert signal number to Signal enum
    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            1 => Some(Signal::SIGHUP),
            2 => Some(Signal::SIGINT),
            3 => Some(Signal::SIGQUIT),
            4 => Some(Signal::SIGILL),
            5 => Some(Signal::SIGTRAP),
            6 => Some(Signal::SIGABRT),
            7 => Some(Signal::SIGBUS),
            8 => Some(Signal::SIGFPE),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            11 => Some(Signal::SIGSEGV),
            12 => Some(Signal::SIGUSR2),
            13 => Some(Signal::SIGPIPE),
            14 => Some(Signal::SIGALRM),
            15 => Some(Signal::SIGTERM),
            16 => Some(Signal::SIGSTKFLT),
            17 => Some(Signal::SIGCHLD),
            18 => Some(Signal::SIGCONT),
            19 => Some(Signal::SIGSTOP),
            20 => Some(Signal::SIGTSTP),
            21 => Some(Signal::SIGTTIN),
            22 => Some(Signal::SIGTTOU),
            23 => Some(Signal::SIGURG),
            24 => Some(Signal::SIGXCPU),
            25 => Some(Signal::SIGXFSZ),
            26 => Some(Signal::SIGVTALRM),
            27 => Some(Signal::SIGPROF),
            28 => Some(Signal::SIGWINCH),
            29 => Some(Signal::SIGIO),
            30 => Some(Signal::SIGPWR),
            31 => Some(Signal::SIGSYS),
            _ => None,
        }
    }

    /// Check if signal can be caught/ignored
    pub fn is_catchable(&self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }
}

/// Signal set (bitmask of signals)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalSet {
    mask: u64,
}

impl SignalSet {
    /// Create an empty signal set
    pub const fn empty() -> Self {
        SignalSet { mask: 0 }
    }

    /// Create a full signal set
    pub const fn full() -> Self {
        SignalSet { mask: u64::MAX }
    }

    /// Add a signal to the set
    pub fn add(&mut self, signal: Signal) {
        let bit = signal as u8;
        if bit < 64 {
            self.mask |= 1u64 << bit;
        }
    }

    /// Remove a signal from the set
    pub fn remove(&mut self, signal: Signal) {
        let bit = signal as u8;
        if bit < 64 {
            self.mask &= !(1u64 << bit);
        }
    }

    /// Check if signal is in the set
    pub fn contains(&self, signal: Signal) -> bool {
        let bit = signal as u8;
        if bit < 64 {
            (self.mask & (1u64 << bit)) != 0
        } else {
            false
        }
    }

    /// Clear all signals
    pub fn clear(&mut self) {
        self.mask = 0;
    }
}
