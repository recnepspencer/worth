use std::sync::Arc;

use worth_execution::ExecutionAllocationPolicy;

use super::{RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactEncodeDenial};
use crate::branch::transition::PositionedBranchTransition;

/// Lazily framed artifacts for one fixed selection of performed transitions,
/// in strictly increasing sequence order. A denial for one item does not end
/// the iteration; the caller decides whether to continue.
pub struct RelationalBranchTransitionArtifacts<'scope, 'authority> {
    covered: crate::branch::transition::sequence::RelationalBranchTransitionSequence,
    performed: std::vec::IntoIter<Arc<PositionedBranchTransition>>,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
}

impl<'scope, 'authority> RelationalBranchTransitionArtifacts<'scope, 'authority> {
    pub(crate) fn new(
        covered: crate::branch::transition::sequence::RelationalBranchTransitionSequence,
        transitions: Vec<Arc<PositionedBranchTransition>>,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> Self {
        Self {
            covered,
            performed: transitions.into_iter(),
            policy,
        }
    }

    /// Compacted coverage immediately preceding this retained tail. Historical
    /// artifacts at or below this sequence belong to the durable journal.
    pub fn covered_sequence(
        &self,
    ) -> crate::branch::transition::sequence::RelationalBranchTransitionSequence {
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
