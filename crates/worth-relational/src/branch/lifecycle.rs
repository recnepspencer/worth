//! Branch lifecycle: the posture a branch reference holds and the durable
//! transitions that change it.
//!
//! Archive and delete are durable branch transitions (Milestone 2 D8): every
//! fallible step runs before the transition is published, and the posture,
//! the branch registry, retention and the head index change only after the
//! medium answers `Committed`.

#[path = "lifecycle_archive.rs"]
mod archive;
#[path = "lifecycle_delete.rs"]
mod delete;
#[path = "lifecycle_denials.rs"]
mod denials;
#[path = "lifecycle_outcomes.rs"]
mod outcomes;

pub(crate) use delete::RelationalBranchDeletionKind;
pub use denials::{RelationalBranchArchiveDenial, RelationalBranchDeleteDenial};
pub use outcomes::{
    ArchivedRelationalBranch, DeletedRelationalBranch, RelationalBranchDeletionOutcome,
    RelationalBranchDeletionPending,
};

/// Runtime-local posture of one mutable branch reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RelationalBranchLifecyclePosture {
    Live,
    Archived,
    Deleting,
}

impl super::reference::RelationalBranchReferenceMutableState {
    /// The observation an archive installs: the same target at the next
    /// generation. Computing it changes nothing.
    pub(super) fn archived_observation(
        &self,
    ) -> Result<super::RelationalBranchReferenceObservation, super::RelationalBranchCellDenial>
    {
        let generation = self
            .observation
            .generation()
            .checked_advance()
            .map_err(|_| super::RelationalBranchCellDenial::GenerationOverflow)?;
        Ok(super::RelationalBranchReferenceObservation::new(
            self.observation.branch_id().clone(),
            self.observation.target().clone(),
            generation,
        ))
    }

    /// Install a prepared archive. Cannot fail.
    pub(super) fn install_archive(
        &mut self,
        observation: super::RelationalBranchReferenceObservation,
    ) {
        self.observation = observation;
        self.lifecycle = RelationalBranchLifecyclePosture::Archived;
    }

    pub(super) fn mark_deleting(&mut self) {
        self.lifecycle = RelationalBranchLifecyclePosture::Deleting;
    }
}

impl super::RelationalBranchReferenceCell {
    /// Give a cell restored from a checkpoint the posture the checkpoint
    /// recorded for it.
    pub(crate) fn restore_lifecycle_posture(&self, posture: RelationalBranchLifecyclePosture) {
        self.state().lifecycle = posture;
    }
}

fn require_local_identity(
    runtime: &crate::runtime::RelationalRuntime,
    identity: &super::RelationalBranchIdentity,
) -> Result<(), ()> {
    (identity.runtime_instance_id() == runtime.runtime_instance_id())
        .then_some(())
        .ok_or(())
}
