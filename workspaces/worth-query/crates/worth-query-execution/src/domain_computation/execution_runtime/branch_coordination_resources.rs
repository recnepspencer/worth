use std::num::NonZeroUsize;

/// Aggregate Query custody for branch commit lanes, including the Weak map
/// entry and an Arc control block that can outlive the last strong lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryBranchCoordinationResourceProfile {
    retained_bytes: NonZeroUsize,
}

impl WorthQueryBranchCoordinationResourceProfile {
    pub const fn bounded(retained_bytes: NonZeroUsize) -> Self {
        Self { retained_bytes }
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes.get()
    }
}

impl Default for WorthQueryBranchCoordinationResourceProfile {
    fn default() -> Self {
        Self::bounded(NonZeroUsize::new(4 * 1_024 * 1_024).unwrap())
    }
}
