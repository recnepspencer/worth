//! Moved whole from `domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome`: the items below and every impl they own.

/// Evidence that sealed decision facts changed at the admitted commit basis.
/// Nothing was committed.
///
/// Re-read from a fresh basis and retry with a fresh source. The runtime never
/// reruns the handler for you.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationStaleAttempt {
    stale_fact_count: usize,
}

impl WorthQueryApplicationStaleAttempt {
    pub const fn stale_fact_count(self) -> usize {
        self.stale_fact_count
    }

    pub(in crate::domain_computation::primary_graph) const fn new(stale_fact_count: usize) -> Self {
        Self { stale_fact_count }
    }
}
