//! Finite checked scheduling policy; it never grants payload or coverage authority.
use std::num::NonZeroU64;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHomeCheckpointLimits {
    entries: NonZeroU64,
    payload_bytes: NonZeroU64,
}
impl ApplicationHomeCheckpointLimits {
    pub const fn new(entries: NonZeroU64, payload_bytes: NonZeroU64) -> Self {
        Self {
            entries,
            payload_bytes,
        }
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
