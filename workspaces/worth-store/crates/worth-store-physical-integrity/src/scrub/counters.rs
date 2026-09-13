#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalIntegrityScrubCounters {
    windows: u64,
    bytes: u64,
}

impl PhysicalIntegrityScrubCounters {
    pub const fn new(windows: u64, bytes: u64) -> Self {
        Self { windows, bytes }
    }

    pub const fn windows(self) -> u64 {
        self.windows
    }

    pub const fn bytes(self) -> u64 {
        self.bytes
    }
}
