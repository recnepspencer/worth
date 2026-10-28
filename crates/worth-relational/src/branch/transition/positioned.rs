use super::{RelationalBranchTransition, RelationalBranchTransitionSequence};
use crate::publication::patch::data::PatchStreamPosition;

/// One branch transition at its place in durable history: its own sequence
/// and the performed commit position it follows.
///
/// Only Relational issues one: at the durable step of a live transition, or
/// by readmitting a transition artifact. Holding one confers no authority to
/// change a branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedBranchTransition {
    sequence: RelationalBranchTransitionSequence,
    follows: PatchStreamPosition,
    transition: RelationalBranchTransition,
}

impl PositionedBranchTransition {
    pub(crate) fn admit(
        sequence: RelationalBranchTransitionSequence,
        follows: PatchStreamPosition,
        transition: RelationalBranchTransition,
    ) -> Self {
        Self {
            sequence,
            follows,
            transition,
        }
    }

    pub fn sequence(&self) -> RelationalBranchTransitionSequence {
        self.sequence
    }

    /// The last performed commit position when the transition was performed.
    /// Zero when no commit preceded it.
    pub fn follows(&self) -> PatchStreamPosition {
        self.follows
    }

    pub fn transition(&self) -> &RelationalBranchTransition {
        &self.transition
    }
}
