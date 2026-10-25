use std::sync::Arc;

use worth_execution::ExecutionAllocationPolicy;

use super::{RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactEncodeDenial};
use crate::branch::PositionedBranchTransition;
use crate::durability::access::DurabilityAccess;

impl DurabilityAccess<'_> {
    /// The runtime's retained branch transitions after its latest checkpoint, in increasing
    /// sequence, as artifacts. This reads the runtime's own transition stream,
    /// the same one checkpoints record their coverage of; it is not a second
    /// history store.
    ///
    /// The selection is fixed when this is called. Encoding is lazy: each
    /// item is framed only when pulled, under `policy`. Cost: O(performed
    /// tail transitions) shared handles up front, then one artifact encode per
    /// pulled item.
    pub fn branch_transition_artifacts<'scope, 'authority>(
        &self,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> RelationalBranchTransitionArtifacts<'scope, 'authority> {
        let (covered, transitions) = self.runtime.history.retained_branch_transitions();
        RelationalBranchTransitionArtifacts {
            covered,
            performed: transitions.into_iter(),
            policy,
        }
    }
}

/// Lazily framed artifacts for one fixed selection of performed transitions,
/// in strictly increasing sequence order. A denial for one item does not end
/// the iteration; the caller decides whether to continue.
pub struct RelationalBranchTransitionArtifacts<'scope, 'authority> {
    covered: crate::branch::RelationalBranchTransitionSequence,
    performed: std::vec::IntoIter<Arc<PositionedBranchTransition>>,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
}

impl RelationalBranchTransitionArtifacts<'_, '_> {
    /// Checkpoint coverage immediately preceding this retained tail. Historical
    /// artifacts at or below this sequence belong to the durable journal.
    pub fn covered_sequence(&self) -> crate::branch::RelationalBranchTransitionSequence {
        self.covered
    }
}

impl Iterator for RelationalBranchTransitionArtifacts<'_, '_> {
    type Item =
        Result<RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactEncodeDenial>;

    fn next(&mut self) -> Option<Self::Item> {
        self.performed.next().map(|transition| {
            RelationalBranchTransitionArtifact::from_positioned(&transition, self.policy)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.performed.size_hint()
    }
}

impl ExactSizeIterator for RelationalBranchTransitionArtifacts<'_, '_> {}

impl std::fmt::Debug for RelationalBranchTransitionArtifacts<'_, '_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalBranchTransitionArtifacts")
            .field("remaining", &self.performed.len())
            .finish_non_exhaustive()
    }
}
