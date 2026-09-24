use serde::{Deserialize, Serialize};

use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

/// Raw native-file vocabulary. Decoding this type never grants current
/// canonical authority; callers must pass it through owner readmission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PersistedCanonicalCommit {
    position: PatchStreamPosition,
    canonical: CanonicalCommitEnvelope,
}

impl PersistedCanonicalCommit {
    pub(crate) fn from_positioned(commit: &PositionedCanonicalCommit) -> Self {
        Self {
            position: commit.position(),
            canonical: commit.envelope().clone(),
        }
    }

    /// Checkpoint envelopes carry canonical history, not rebuildable index
    /// caches. The versioned checkpoint artifact carries retained generations.
    pub(crate) fn from_checkpoint_positioned(commit: &PositionedCanonicalCommit) -> Self {
        let mut persisted = Self::from_positioned(commit);
        persisted.canonical.derived_index_artifacts = Default::default();
        persisted
    }

    pub(crate) fn into_receipt(self) -> crate::history::data::RelationalCommitReceipt {
        self.canonical.commit
    }

    #[cfg(test)]
    pub(crate) fn envelope_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelope {
        &mut self.canonical
    }

    pub(crate) fn readmit(
        self,
    ) -> Result<crate::durability::migration::ReadmittedCanonicalCommit, String> {
        crate::durability::migration::ReadmittedCanonicalCommit::readmit_current(
            self.position,
            self.canonical,
        )
    }
}
