//! Capabilities System
//!
//! Linux-style capability bits for fine-grained privilege control.

use core::sync::atomic::{AtomicU64, Ordering};

/// Capability bits (Linux-compatible subset)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Capability {
    /// Override file read/write/execute permissions
    CapDacOverride = 1 << 0,
    /// Override file ownership checks
    CapDacReadSearch = 1 << 1,
    /// Override file owner/group changes
    CapFowner = 1 << 2,
    /// Ignore file set-user-ID/set-group-ID bits
    CapFsetid = 1 << 3,
    /// Bypass permission checks for sending signals
    CapKill = 1 << 4,
    /// Change file owner/group
    CapChown = 1 << 5,
    /// Bind to privileged ports (<1024)
    CapNetBindService = 1 << 6,
    /// Perform network admin operations
    CapNetAdmin = 1 << 7,
    /// Perform raw socket operations
    CapNetRaw = 1 << 8,
    /// Lock memory (mlock)
    CapIpcLock = 1 << 9,
    /// Override IPC ownership checks
    CapIpcOwner = 1 << 10,
    /// Load/unload kernel modules
    CapSysModule = 1 << 11,
    /// Perform raw I/O operations
    CapSysRawio = 1 << 12,
    /// Use chroot()
    CapSysChroot = 1 << 13,
    /// Trace any process
    CapSysPtrace = 1 << 14,
    /// Perform privileged process operations
    CapSysAdmin = 1 << 15,
    /// Use reboot()
    CapSysBoot = 1 << 16,
    /// Raise process nice value
    CapSysNice = 1 << 17,
    /// Override resource limits
    CapSysResource = 1 << 18,
    /// Set system time
    CapSysTime = 1 << 19,
    /// Create device files
    CapMknod = 1 << 20,
    /// Perform lease operations on files
    CapLease = 1 << 21,
    /// Perform auditing operations
    CapAuditWrite = 1 << 22,
    /// Control audit system
    CapAuditControl = 1 << 23,
    /// Set file capabilities
    CapSetfcap = 1 << 24,
    /// Override MAC (Mandatory Access Control)
    CapMacOverride = 1 << 25,
    /// Configure MAC policy
    CapMacAdmin = 1 << 26,
    /// Use setuid/setgid
    CapSetuid = 1 << 27,
    /// Use setgid
    CapSetgid = 1 << 28,
    /// Set capability sets
    CapSetpcap = 1 << 29,
}

impl Capability {
    /// Get the bit mask for this capability
    pub const fn mask(&self) -> u64 {
        *self as u64
    }

    /// Get capability from bit position
    pub fn from_bit(bit: u32) -> Option<Self> {
        if bit >= 30 {
            return None;
        }
        let mask = 1u64 << bit;
        Self::from_mask(mask)
    }

    /// Get capability from mask (single bit)
    pub fn from_mask(mask: u64) -> Option<Self> {
        match mask {
            m if m == Capability::CapDacOverride.mask() => Some(Capability::CapDacOverride),
            m if m == Capability::CapDacReadSearch.mask() => Some(Capability::CapDacReadSearch),
            m if m == Capability::CapFowner.mask() => Some(Capability::CapFowner),
            m if m == Capability::CapFsetid.mask() => Some(Capability::CapFsetid),
            m if m == Capability::CapKill.mask() => Some(Capability::CapKill),
            m if m == Capability::CapChown.mask() => Some(Capability::CapChown),
            m if m == Capability::CapNetBindService.mask() => Some(Capability::CapNetBindService),
            m if m == Capability::CapNetAdmin.mask() => Some(Capability::CapNetAdmin),
            m if m == Capability::CapNetRaw.mask() => Some(Capability::CapNetRaw),
            m if m == Capability::CapIpcLock.mask() => Some(Capability::CapIpcLock),
            m if m == Capability::CapIpcOwner.mask() => Some(Capability::CapIpcOwner),
            m if m == Capability::CapSysModule.mask() => Some(Capability::CapSysModule),
            m if m == Capability::CapSysRawio.mask() => Some(Capability::CapSysRawio),
            m if m == Capability::CapSysChroot.mask() => Some(Capability::CapSysChroot),
            m if m == Capability::CapSysPtrace.mask() => Some(Capability::CapSysPtrace),
            m if m == Capability::CapSysAdmin.mask() => Some(Capability::CapSysAdmin),
            m if m == Capability::CapSysBoot.mask() => Some(Capability::CapSysBoot),
            m if m == Capability::CapSysNice.mask() => Some(Capability::CapSysNice),
            m if m == Capability::CapSysResource.mask() => Some(Capability::CapSysResource),
            m if m == Capability::CapSysTime.mask() => Some(Capability::CapSysTime),
            m if m == Capability::CapMknod.mask() => Some(Capability::CapMknod),
            m if m == Capability::CapLease.mask() => Some(Capability::CapLease),
            m if m == Capability::CapAuditWrite.mask() => Some(Capability::CapAuditWrite),
            m if m == Capability::CapAuditControl.mask() => Some(Capability::CapAuditControl),
            m if m == Capability::CapSetfcap.mask() => Some(Capability::CapSetfcap),
            m if m == Capability::CapMacOverride.mask() => Some(Capability::CapMacOverride),
            m if m == Capability::CapMacAdmin.mask() => Some(Capability::CapMacAdmin),
            m if m == Capability::CapSetuid.mask() => Some(Capability::CapSetuid),
            m if m == Capability::CapSetgid.mask() => Some(Capability::CapSetgid),
            m if m == Capability::CapSetpcap.mask() => Some(Capability::CapSetpcap),
            _ => None,
        }
    }
}

/// Capability set
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilitySet {
    bits: u64,
}

impl CapabilitySet {
    /// Empty capability set
    pub const EMPTY: Self = CapabilitySet { bits: 0 };

    /// Full capability set (all capabilities)
    pub const FULL: Self = CapabilitySet {
        bits: (1u64 << 30) - 1,
    };

    /// Create a new empty capability set
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Create from raw bits
    pub const fn from_bits(bits: u64) -> Self {
        CapabilitySet { bits }
    }

    /// Get raw bits
    pub const fn bits(&self) -> u64 {
        self.bits
    }

    /// Check if capability is set
    pub const fn has(&self, cap: Capability) -> bool {
        (self.bits & cap.mask()) != 0
    }

    /// Add a capability
    pub fn add(&mut self, cap: Capability) {
        self.bits |= cap.mask();
    }

    /// Remove a capability
    pub fn remove(&mut self, cap: Capability) {
        self.bits &= !cap.mask();
    }

    /// Clear all capabilities
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    /// Set all capabilities
    pub fn set_all(&mut self) {
        self.bits = Self::FULL.bits;
    }

    /// Check if empty
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Check if full
    pub const fn is_full(&self) -> bool {
        self.bits == Self::FULL.bits
    }

    /// Intersection with another set
    pub const fn intersect(&self, other: &CapabilitySet) -> CapabilitySet {
        CapabilitySet {
            bits: self.bits & other.bits,
        }
    }

    /// Union with another set
    pub const fn union(&self, other: &CapabilitySet) -> CapabilitySet {
        CapabilitySet {
            bits: self.bits | other.bits,
        }
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-process capability sets
#[derive(Debug)]
pub struct ProcessCapabilities {
    /// Permitted capabilities (superset of effective)
    permitted: AtomicU64,
    /// Effective capabilities (currently active)
    effective: AtomicU64,
    /// Inheritable capabilities (passed to child processes)
    inheritable: AtomicU64,
}

impl Clone for ProcessCapabilities {
    fn clone(&self) -> Self {
        ProcessCapabilities {
            permitted: AtomicU64::new(self.permitted.load(Ordering::Acquire)),
            effective: AtomicU64::new(self.effective.load(Ordering::Acquire)),
            inheritable: AtomicU64::new(self.inheritable.load(Ordering::Acquire)),
        }
    }
}

impl ProcessCapabilities {
    /// Create new process capabilities (empty by default)
    pub const fn new() -> Self {
        ProcessCapabilities {
            permitted: AtomicU64::new(0),
            effective: AtomicU64::new(0),
            inheritable: AtomicU64::new(0),
        }
    }

    /// Create with root capabilities (all capabilities)
    pub const fn root() -> Self {
        let all = CapabilitySet::FULL.bits;
        ProcessCapabilities {
            permitted: AtomicU64::new(all),
            effective: AtomicU64::new(all),
            inheritable: AtomicU64::new(all),
        }
    }

    /// Get permitted capability set
    pub fn permitted(&self) -> CapabilitySet {
        CapabilitySet::from_bits(self.permitted.load(Ordering::Acquire))
    }

    /// Get effective capability set
    pub fn effective(&self) -> CapabilitySet {
        CapabilitySet::from_bits(self.effective.load(Ordering::Acquire))
    }

    /// Get inheritable capability set
    pub fn inheritable(&self) -> CapabilitySet {
        CapabilitySet::from_bits(self.inheritable.load(Ordering::Acquire))
    }

    /// Set permitted capabilities
    pub fn set_permitted(&self, caps: CapabilitySet) {
        self.permitted.store(caps.bits(), Ordering::Release);
    }

    /// Set effective capabilities
    pub fn set_effective(&self, caps: CapabilitySet) {
        self.effective.store(caps.bits(), Ordering::Release);
    }

    /// Set inheritable capabilities
    pub fn set_inheritable(&self, caps: CapabilitySet) {
        self.inheritable.store(caps.bits(), Ordering::Release);
    }

    /// Check if process has a capability (effective)
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.effective().has(cap)
    }

    /// Add capability to effective and permitted sets
    pub fn add_capability(&self, cap: Capability) {
        let mask = cap.mask();
        self.permitted.fetch_or(mask, Ordering::AcqRel);
        self.effective.fetch_or(mask, Ordering::AcqRel);
    }

    /// Remove capability from all sets
    pub fn remove_capability(&self, cap: Capability) {
        let mask = !cap.mask();
        self.permitted.fetch_and(mask, Ordering::AcqRel);
        self.effective.fetch_and(mask, Ordering::AcqRel);
        self.inheritable.fetch_and(mask, Ordering::AcqRel);
    }

    /// Clear all capabilities
    pub fn clear_all(&self) {
        self.permitted.store(0, Ordering::Release);
        self.effective.store(0, Ordering::Release);
        self.inheritable.store(0, Ordering::Release);
    }
}

impl Default for ProcessCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if current process has a capability
///
/// In a real implementation, this would check the current task's capabilities.
/// For now, this is a placeholder that always returns true for root operations.
pub fn cap_capable(_cap: Capability) -> bool {
    // Placeholder: In real implementation, check current task's capabilities
    true
}

/// Raise a capability in the effective set
pub fn cap_raise(_cap: Capability) -> Result<(), &'static str> {
    // Placeholder: In real implementation, modify current task's capabilities
    Ok(())
}

/// Drop a capability from the effective set
pub fn cap_drop(_cap: Capability) -> Result<(), &'static str> {
    // Placeholder: In real implementation, modify current task's capabilities
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_mask() {
        assert_eq!(Capability::CapChown.mask(), 1 << 5);
        assert_eq!(Capability::CapNetAdmin.mask(), 1 << 7);
    }

    #[test]
    fn test_capability_set_empty() {
        let caps = CapabilitySet::EMPTY;
        assert!(caps.is_empty());
        assert!(!caps.has(Capability::CapChown));
    }

    #[test]
    fn test_capability_set_add() {
        let mut caps = CapabilitySet::new();
        assert!(!caps.has(Capability::CapChown));

        caps.add(Capability::CapChown);
        assert!(caps.has(Capability::CapChown));
    }

    #[test]
    fn test_capability_set_remove() {
        let mut caps = CapabilitySet::FULL;
        assert!(caps.has(Capability::CapChown));

        caps.remove(Capability::CapChown);
        assert!(!caps.has(Capability::CapChown));
    }

    #[test]
    fn test_capability_set_operations() {
        let mut caps1 = CapabilitySet::new();
        caps1.add(Capability::CapChown);

        let mut caps2 = CapabilitySet::new();
        caps2.add(Capability::CapNetAdmin);

        let union = caps1.union(&caps2);
        assert!(union.has(Capability::CapChown));
        assert!(union.has(Capability::CapNetAdmin));

        let intersect = caps1.intersect(&caps2);
        assert!(!intersect.has(Capability::CapChown));
        assert!(!intersect.has(Capability::CapNetAdmin));
    }

    #[test]
    fn test_process_capabilities() {
        let caps = ProcessCapabilities::new();
        assert!(!caps.has_capability(Capability::CapChown));

        caps.add_capability(Capability::CapChown);
        assert!(caps.has_capability(Capability::CapChown));

        caps.remove_capability(Capability::CapChown);
        assert!(!caps.has_capability(Capability::CapChown));
    }

    #[test]
    fn test_root_capabilities() {
        let caps = ProcessCapabilities::root();
        assert!(caps.has_capability(Capability::CapChown));
        assert!(caps.has_capability(Capability::CapNetAdmin));
        assert!(caps.has_capability(Capability::CapSysAdmin));
    }
}
