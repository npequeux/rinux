//! Signal Handler
//!
//! Signal handler management and delivery.

use super::sig_types::{Signal, SignalSet};
use crate::types::Pid;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Signal handler function type
pub type SignalHandlerFn = fn(Signal);

/// Signal handler action
#[derive(Clone, Copy)]
pub enum SignalHandler {
    /// Default action
    Default,
    /// Ignore signal
    Ignore,
    /// Custom handler function
    Handler(SignalHandlerFn),
}

/// Signal handlers for a process
pub struct SignalHandlers {
    handlers: BTreeMap<u8, SignalHandler>,
    blocked: SignalSet,
    pending: SignalSet,
}

impl Default for SignalHandlers {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalHandlers {
    /// Create new signal handlers with defaults
    pub fn new() -> Self {
        SignalHandlers {
            handlers: BTreeMap::new(),
            blocked: SignalSet::empty(),
            pending: SignalSet::empty(),
        }
    }

    /// Set handler for a signal
    pub fn set_handler(&mut self, signal: Signal, handler: SignalHandler) {
        if signal.is_catchable() {
            self.handlers.insert(signal as u8, handler);
        }
    }

    /// Get handler for a signal
    pub fn get_handler(&self, signal: Signal) -> SignalHandler {
        self.handlers
            .get(&(signal as u8))
            .copied()
            .unwrap_or(SignalHandler::Default)
    }

    /// Block a signal
    pub fn block(&mut self, signal: Signal) {
        self.blocked.add(signal);
    }

    /// Unblock a signal
    pub fn unblock(&mut self, signal: Signal) {
        self.blocked.remove(signal);
    }

    /// Check if signal is blocked
    pub fn is_blocked(&self, signal: Signal) -> bool {
        self.blocked.contains(signal)
    }

    /// Add pending signal
    pub fn add_pending(&mut self, signal: Signal) {
        self.pending.add(signal);
    }

    /// Get and clear next pending signal
    pub fn next_pending(&mut self) -> Option<Signal> {
        for sig_num in 1..32 {
            if let Some(signal) = Signal::from_num(sig_num) {
                if self.pending.contains(signal) && !self.blocked.contains(signal) {
                    self.pending.remove(signal);
                    return Some(signal);
                }
            }
        }
        None
    }
}

/// Global signal handler registry
static SIGNAL_HANDLERS: Mutex<BTreeMap<Pid, SignalHandlers>> = Mutex::new(BTreeMap::new());

/// Initialize signal handlers
pub fn init() {
    // Initialize global signal handler registry
    let mut handlers = SIGNAL_HANDLERS.lock();
    *handlers = BTreeMap::new();
}

/// Register signal handlers for a process
pub fn register_process(pid: Pid) {
    let mut handlers = SIGNAL_HANDLERS.lock();
    handlers.insert(pid, SignalHandlers::new());
}

/// Unregister signal handlers for a process
pub fn unregister_process(pid: Pid) {
    let mut handlers = SIGNAL_HANDLERS.lock();
    handlers.remove(&pid);
}

/// Send a signal to a process
pub fn send_signal(pid: Pid, signal: Signal) -> Result<(), ()> {
    let mut handlers = SIGNAL_HANDLERS.lock();
    if let Some(proc_handlers) = handlers.get_mut(&pid) {
        proc_handlers.add_pending(signal);
        Ok(())
    } else {
        Err(())
    }
}

/// Deliver pending signals for a process
pub fn deliver_signals(pid: Pid) {
    loop {
        // Get next pending signal
        let signal_opt = {
            let mut handlers = SIGNAL_HANDLERS.lock();
            if let Some(proc_handlers) = handlers.get_mut(&pid) {
                proc_handlers.next_pending()
            } else {
                None
            }
        };

        // Break if no more pending signals
        let signal = match signal_opt {
            Some(s) => s,
            None => break,
        };

        // Get handler for this signal
        let handler = {
            let handlers = SIGNAL_HANDLERS.lock();
            if let Some(proc_handlers) = handlers.get(&pid) {
                proc_handlers.get_handler(signal)
            } else {
                SignalHandler::Default
            }
        };

        // Execute handler action
        match handler {
            SignalHandler::Default => {
                // Default action (terminate, stop, etc.)
                default_signal_action(pid, signal);
            }
            SignalHandler::Ignore => {
                // Do nothing
            }
            SignalHandler::Handler(func) => {
                // Call custom handler
                func(signal);
            }
        }
    }
}

/// Default signal action
fn default_signal_action(_pid: Pid, signal: Signal) {
    match signal {
        Signal::SIGKILL | Signal::SIGTERM | Signal::SIGINT | Signal::SIGQUIT => {
            // Terminate process
            // TODO: Implement process termination
        }
        Signal::SIGSTOP | Signal::SIGTSTP | Signal::SIGTTIN | Signal::SIGTTOU => {
            // Stop process
            // TODO: Implement process stopping
        }
        Signal::SIGCONT => {
            // Continue process
            // TODO: Implement process continuation
        }
        _ => {
            // Ignore by default
        }
    }
}
