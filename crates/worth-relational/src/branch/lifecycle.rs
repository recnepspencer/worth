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

pub use denials::{RelationalBranchArchiveDenial, RelationalBranchDeleteDenial};
pub use outcomes::{
    ArchivedRelationalBranch, DeletedRelationalBranch, RelationalBranchDeletionOutcome,
    RelationalBranchDeletionPending,
};

/// Which deletion transition to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalBranchDeletionKind {
    /// Decide from the branch's active operations, as a live delete does:
    /// request deletion while any remain, otherwise delete.
    ByActiveOperations,
    /// Mark the branch for deletion and retire its name.
    Requested,
    /// Remove the branch, retire its head and its name.
    Delete,
}

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
    ) -> Result<
        crate::branch::reference::observation::RelationalBranchReferenceObservation,
        crate::branch::reference::RelationalBranchCellDenial,
    > {
        let generation = self
            .observation
            .generation()
            .checked_advance()
            .map_err(|_| {
                crate::branch::reference::RelationalBranchCellDenial::GenerationOverflow
            })?;
        Ok(
            crate::branch::reference::observation::RelationalBranchReferenceObservation::new(
                self.observation.branch_id().clone(),
                self.observation.target().clone(),
                generation,
            ),
        )
    }

    /// Install a prepared archive. Cannot fail.
    pub(super) fn install_archive(
        &mut self,
        observation: crate::branch::reference::observation::RelationalBranchReferenceObservation,
    ) {
        self.observation = observation;
        self.lifecycle = RelationalBranchLifecyclePosture::Archived;
    }

    pub(super) fn mark_deleting(&mut self) {
        self.lifecycle = RelationalBranchLifecyclePosture::Deleting;
    }
}

impl crate::branch::reference::RelationalBranchReferenceCell {
    /// Give a cell restored from a checkpoint the posture the checkpoint
    /// recorded for it.
    pub(crate) fn restore_lifecycle_posture(&self, posture: RelationalBranchLifecyclePosture) {
        self.state().lifecycle = posture;
    }
}

fn require_local_identity(
    runtime_instance_id: u64,
    identity: &crate::branch::identity::RelationalBranchIdentity,
) -> Result<(), ()> {
    (identity.runtime_instance_id() == runtime_instance_id)
        .then_some(())
        .ok_or(())
}
