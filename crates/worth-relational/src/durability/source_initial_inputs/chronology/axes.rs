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
