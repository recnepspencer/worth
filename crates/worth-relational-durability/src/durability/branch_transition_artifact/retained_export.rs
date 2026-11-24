use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

use worth_execution::ExecutionAllocationPolicy;

use super::RelationalBranchTransitionArtifact;
use contracts::durability::branch_transition_artifact::encode_denial::RelationalBranchTransitionArtifactEncodeDenial;
use worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition;

/// Lazily framed artifacts for one fixed selection of performed transitions,
/// in strictly increasing sequence order. A denial for one item does not end
/// the iteration; the caller decides whether to continue.
pub struct RelationalBranchTransitionArtifacts<'scope, 'authority> {
    covered: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    performed: std::vec::IntoIter<Arc<PositionedBranchTransition>>,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
}

impl<'scope, 'authority> RelationalBranchTransitionArtifacts<'scope, 'authority> {
    pub(crate) fn new(
        covered: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
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
    ) -> contracts::branch::transition::sequence::RelationalBranchTransitionSequence {
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

#[doc(hidden)]
pub trait RelationalBranchTransitionArtifactsOwnerPort<'scope, 'authority>: Sized {
    fn new(
        covered: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
        transitions: Vec<Arc<PositionedBranchTransition>>,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> Self;
}

impl<'scope, 'authority> RelationalBranchTransitionArtifactsOwnerPort<'scope, 'authority>
    for RelationalBranchTransitionArtifacts<'scope, 'authority>
{
    fn new(
        covered: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
        transitions: Vec<Arc<PositionedBranchTransition>>,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> Self {
        Self::new(covered, transitions, policy)
    }
}
