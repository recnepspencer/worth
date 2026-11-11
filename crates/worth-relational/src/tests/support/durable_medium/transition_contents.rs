//! The test medium's exact branch-transition bytes.

use super::TestDurableMedium;
use crate::branch::RelationalBranchTransitionSequence;

impl TestDurableMedium {
    /// The exact transition bytes held at `sequence`.
    pub(crate) fn transition_bytes(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Option<Box<[u8]>> {
        self.transition_journal().bytes_at(sequence).map(Box::from)
    }
}
