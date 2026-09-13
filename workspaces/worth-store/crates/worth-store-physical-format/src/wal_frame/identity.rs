use core::num::NonZeroU64;

/// Canonical physical-format identity of a WAL segment container.
///
/// LSN bounds and payload length are intentionally absent: they exist only in
/// the checksummed frame and are exposed after framing validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalSegmentIdentity {
    segment: NonZeroU64,
    generation: NonZeroU64,
}

impl WalSegmentIdentity {
    pub const fn new(segment: u64, generation: u64) -> Option<Self> {
        let Some(segment) = NonZeroU64::new(segment) else {
            return None;
        };
        let Some(generation) = NonZeroU64::new(generation) else {
            return None;
        };
        Some(Self {
            segment,
            generation,
        })
    }

    pub const fn segment(self) -> NonZeroU64 {
        self.segment
    }

    pub const fn generation(self) -> NonZeroU64 {
        self.generation
    }
}
