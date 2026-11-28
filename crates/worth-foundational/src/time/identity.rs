/// Identity assigned to a raw wall-clock source by its composition owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockSourceIdentity(u64);

impl ClockSourceIdentity {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// The source assignment recorded alongside wall time. Replacing a source
/// requires a fresh identity when its recorded evidence must be distinguished.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeSourceEpoch(ClockSourceIdentity);

impl TimeSourceEpoch {
    pub const fn new(identity: ClockSourceIdentity) -> Self {
        Self(identity)
    }

    pub const fn identity(self) -> ClockSourceIdentity {
        self.0
    }
}

/// Identity of one uninterrupted monotonic timeline. Adapter owners allocate
/// distinct epochs for independent timelines and after each reboot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BootEpoch(u64);

impl BootEpoch {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}
