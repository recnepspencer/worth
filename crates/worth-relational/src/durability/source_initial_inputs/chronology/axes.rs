//! Small descriptive branch axes; no root materialization or live authority.
use super::*;
use crate::branch::{RelationalBranchTarget, RelationalBranchVersion};
use crate::history::RelationalCommitIndexEntry;
use worth_foundational::{
    FoundationalBranchReferenceGeneration as Generation, FoundationalBranchTarget,
};

#[derive(Clone)]
pub(super) struct Axes<'a> {
    pub(super) generation: Generation,
    pub(super) truth: RelationalBranchVersion,
    pub(super) lifecycle: Lifecycle,
    pub(super) commit: Option<&'a RelationalCommitIndexEntry>,
    pub(super) schema: [u8; 32],
    pub(super) removed: bool,
}
impl Axes<'_> {
    pub(super) fn empty(original: &super::super::recorded::RecordedEmptySchemaObservation) -> Self {
        Self {
            generation: original.generation,
            truth: original.truth_version,
            lifecycle: original.lifecycle,
            commit: None,
            schema: original.schema_carrier,
            removed: false,
        }
    }
    pub(super) fn matches_empty(
        &self,
        original: &super::super::recorded::RecordedEmptySchemaObservation,
    ) -> bool {
        self.commit.is_none()
            && self.generation == original.generation
            && self.truth == original.truth_version
            && self.lifecycle == original.lifecycle
            && self.schema == original.schema_carrier
    }
    pub(super) fn matches_basis(&self, target: &RelationalBranchTarget) -> bool {
        self.commit.is_some_and(|index| {
            index.commit.commit_id.0 == target.selected_commit_id()
                && index.commit.version_id.0 == target.version_id()
                && &index.root == target.roots()
                && index
                    .commit
                    .parents
                    .iter()
                    .map(|id| id.0)
                    .eq(target.parent_commit_ids().iter().copied())
        })
    }
    pub(super) fn matches_target(
        &self,
        target: &FoundationalBranchTarget<RelationalBranchTarget>,
    ) -> bool {
        match target {
            FoundationalBranchTarget::Empty => self.commit.is_none(),
            FoundationalBranchTarget::Basis(target) => self.matches_basis(target),
        }
    }
}
pub(super) fn posture(lifecycle: Lifecycle) -> u8 {
    match lifecycle {
        Lifecycle::Live => 0,
        Lifecycle::Archived => 1,
        Lifecycle::Deleting => 2,
    }
}

impl Axes<'_> {
    pub(super) fn same_state(&self, other: &Self) -> bool {
        self.generation == other.generation
            && self.truth == other.truth
            && self.lifecycle == other.lifecycle
            && self.commit.map(|row| row.commit.commit_id)
                == other.commit.map(|row| row.commit.commit_id)
            && self.schema == other.schema
            && self.removed == other.removed
    }
    pub(super) fn requires_transition(&self, entry: &TransitionOriginalRecord) -> bool {
        if self.removed || self.schema != entry.schema {
            return false;
        }
        match &entry.transition {
            Transition::Fork {
                source_generation,
                source_truth_version,
                source_head,
                ..
            } => {
                self.lifecycle == Lifecycle::Live
                    && self.generation == *source_generation
                    && self.truth == *source_truth_version
                    && self.matches_basis(source_head)
            }
            Transition::Archive {
                prior_generation,
                prior_truth_version,
                ..
            } => {
                self.lifecycle == Lifecycle::Live
                    && self.generation == *prior_generation
                    && self.truth == *prior_truth_version
            }
            Transition::DeletionRequested {
                prior_generation,
                prior_truth_version,
                ..
            } => self.generation == *prior_generation && self.truth == *prior_truth_version,
            Transition::Delete {
                prior_generation,
                prior_truth_version,
                head_commit,
                ..
            } => {
                self.generation == *prior_generation
                    && self.truth == *prior_truth_version
                    && self.commit.map(|row| row.commit.commit_id) == *head_commit
            }
            Transition::RetainedFork { .. } => false,
        }
    }
}
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct ObservationKey<'a> {
    branch: &'a BranchId,
    generation: u64,
    truth: u64,
    posture: u8,
    commit: Option<crate::history::data::CommitId>,
}
impl<'a> ObservationKey<'a> {
    pub(super) fn of_axes(branch: &'a BranchId, axes: &Axes<'_>) -> Self {
        Self {
            branch,
            generation: axes.generation.get(),
            truth: axes.truth.as_u64(),
            posture: posture(axes.lifecycle),
            commit: axes.commit.map(|row| row.commit.commit_id),
        }
    }
    pub(super) fn of_descriptor(
        descriptor: &'a crate::branch::RelationalBranchBasisDescriptor,
    ) -> Self {
        use crate::branch::RelationalBranchBasisPosture;
        Self {
            branch: descriptor.branch_id(),
            generation: descriptor.reference().generation().get(),
            truth: descriptor.truth_version().as_u64(),
            posture: match descriptor.posture() {
                RelationalBranchBasisPosture::Live => 0,
                RelationalBranchBasisPosture::Archived => 1,
                RelationalBranchBasisPosture::Deleting => 2,
            },
            commit: match descriptor.reference().target() {
                FoundationalBranchTarget::Empty => None,
                FoundationalBranchTarget::Basis(target) => {
                    Some(crate::history::data::CommitId(target.selected_commit_id()))
                }
            },
        }
    }
}
