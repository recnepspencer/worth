//! Read-only retained Native writer attribution on a performed receipt.
use super::CommitPublication;

impl CommitPublication {
    /// Original applied source effects, before synchronous producers appended
    /// their actual writes. This is descriptive receipt custody, never a grant.
    pub fn authored_source_effects(
        &self,
    ) -> Option<&crate::mvcc::RelationalCoupledAuthoredSourceEffects> {
        self.source_effects.as_ref()
    }
}
