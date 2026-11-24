use super::RelationalDurableTransitionReadFailure;
use crate::branch::transition::sequence::RelationalBranchTransitionSequence;
use std::sync::Arc;

/// Read-only history access for a speculative runtime that cannot publish to
/// the source's medium. Moving/closing that medium still makes reads fail.
#[derive(Debug, Clone)]
pub(crate) struct RelationalTransitionHistoryReader {
    binding: Arc<super::RelationalDurableMediumBinding>,
}

impl super::RelationalDurableMediumBinding {
    pub(crate) fn transition_history(self: &Arc<Self>) -> RelationalTransitionHistoryReader {
        RelationalTransitionHistoryReader {
            binding: Arc::clone(self),
        }
    }
}

impl RelationalTransitionHistoryReader {
    pub(crate) fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<
        crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact,
        RelationalDurableTransitionReadFailure,
    > {
        self.binding.read_transition(sequence)
    }
}
