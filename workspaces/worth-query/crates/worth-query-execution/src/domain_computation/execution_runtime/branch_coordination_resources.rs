use std::num::NonZeroUsize;

/// Aggregate Query custody for branch commit lanes, including the Weak map
/// entry and an Arc control block that can outlive the last strong lane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryBranchCoordinationResourceProfile {
    retained_bytes: Option<NonZeroUsize>,
}

impl WorthQueryBranchCoordinationResourceProfile {
    pub const fn bounded(retained_bytes: NonZeroUsize) -> Self {
        Self {
            retained_bytes: Some(retained_bytes),
        }
    }

    pub fn retained_bytes(self) -> Option<usize> {
        self.retained_bytes.map(NonZeroUsize::get)
    }
}
