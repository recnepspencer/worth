//! A finite navigation window and explicit bounds on its protected and pending custody.
use std::num::NonZeroUsize;

/// Maximum eligible unpinned steps across all futures. Current selection and
/// explicit bookmarks are protected separately. Increasing the window does
/// not reconstruct sources already excluded by an earlier frontier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHistoryRecentSteps(NonZeroUsize);

impl ApplicationHistoryRecentSteps {
    pub const DEFAULT: Self = Self(NonZeroUsize::new(200).unwrap());

    pub const fn new(steps: NonZeroUsize) -> Self {
        Self(steps)
    }

    pub const fn get(self) -> usize {
        self.0.get()
    }
}

/// Installed session-resource policy, independent of historical model payload.
/// Pending releases retain real head/cleanup owners; reaching capacity refuses
/// additional work before it can create unregistered retirement obligations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHistoryRetentionResources {
    pub maximum_bookmarks: NonZeroUsize,
    pub maximum_pending_releases: NonZeroUsize,
}

impl ApplicationHistoryRetentionResources {
    pub const DEFAULT: Self = Self {
        maximum_bookmarks: NonZeroUsize::new(32).unwrap(),
        maximum_pending_releases: NonZeroUsize::new(32).unwrap(),
    };
}
