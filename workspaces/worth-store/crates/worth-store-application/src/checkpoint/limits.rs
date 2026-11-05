//! Finite checked scheduling policy; it never grants payload or coverage authority.
use std::num::NonZeroU64;
use std::time::Duration;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHomeCheckpointLimits {
    entries: NonZeroU64,
    payload_bytes: NonZeroU64,
    maintenance_work: NonZeroU64,
    maintenance_memory_bytes: NonZeroU64,
    maintenance_timeout: Duration,
}
impl ApplicationHomeCheckpointLimits {
    pub const fn new(entries: NonZeroU64, payload_bytes: NonZeroU64) -> Self {
        Self {
            entries,
            payload_bytes,
            maintenance_work: NonZeroU64::new(1 << 30).expect("finite maintenance work default"),
            maintenance_memory_bytes: NonZeroU64::new(256 << 20)
                .expect("finite maintenance memory default"),
            maintenance_timeout: Duration::from_secs(60),
        }
    }
    /// Automatic upkeep owns this allowance independently of application calls.
    /// Explicit checkpoint and recovery calls retain their invoking request.
    pub const fn with_maintenance_allowance(
        mut self,
        work: NonZeroU64,
        memory_bytes: NonZeroU64,
        timeout: Duration,
    ) -> Self {
        self.maintenance_work = work;
        self.maintenance_memory_bytes = memory_bytes;
        self.maintenance_timeout = timeout;
        self
    }
    pub const fn maintenance_work(self) -> NonZeroU64 {
        self.maintenance_work
    }
    pub const fn maintenance_memory_bytes(self) -> NonZeroU64 {
        self.maintenance_memory_bytes
    }
    pub const fn maintenance_timeout(self) -> Duration {
        self.maintenance_timeout
    }
    pub const fn entries(self) -> NonZeroU64 {
        self.entries
    }
    pub const fn payload_bytes(self) -> NonZeroU64 {
        self.payload_bytes
    }
}
impl Default for ApplicationHomeCheckpointLimits {
    fn default() -> Self {
        Self::new(
            NonZeroU64::new(4096).expect("finite nonzero default"),
            NonZeroU64::new(64 << 20).expect("finite nonzero default"),
        )
    }
}
