use std::sync::Arc;
use worth_relational_contracts::internal_port::branch::transition::sequence::RelationalBranchTransitionSequence;
use worth_relational_storage::internal_port::durability::medium::transition_read::RelationalDurableTransitionReadFailure;

/// Read-only history access for a speculative runtime that cannot publish to
/// the source's medium. Moving/closing that medium still makes reads fail.
#[derive(Debug, Clone)]
pub struct RelationalTransitionHistoryReader {
    binding: Arc<super::RelationalDurableMediumBinding>,
}

impl super::RelationalDurableMediumBinding {
    pub fn transition_history(self: &Arc<Self>) -> RelationalTransitionHistoryReader {
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
