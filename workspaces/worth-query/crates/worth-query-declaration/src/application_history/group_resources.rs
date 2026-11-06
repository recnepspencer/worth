//! Finite installation policy for private gesture members and retained results.
use std::num::NonZeroUsize;

/// Runtime resource policy carried by the installed history recipe. This is
/// not historical model state and is never restored by snapshot navigation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHistoryGroupResources {
    pub maximum_members: NonZeroUsize,
    pub maximum_retained_results: NonZeroUsize,
}

impl ApplicationHistoryGroupResources {
    /// One document admits one unfinished group. Its member and result storage
    /// is bounded independently of the number of retained history steps.
    pub const DEFAULT: Self = Self {
        maximum_members: NonZeroUsize::new(64).unwrap(),
        maximum_retained_results: NonZeroUsize::new(64).unwrap(),
    };
}
