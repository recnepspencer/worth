//! Durability of the checkpoint publication this recovery selected, when the
//! killed runtime replaced `checkpoint.current` but did not prove the
//! replacement durable.

use worth_store_physical_backend::{AdmittedRecoveryFilesystemMedia, ArtifactTreeFailure};

use super::{PhysicalRecoveryCoordination, SharedRecoveryCheckpoint};

/// Why the selected checkpoint's publication was not made durable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedCheckpointPublicationDenial {
    /// The published checkpoint is not the one this recovery selected, or
    /// could not be observed. Nothing was synchronized.
    CheckpointMismatch,
    Observation(ArtifactTreeFailure),
    /// The namespace synchronization started and did not complete.
    Synchronization(ArtifactTreeFailure),
}

impl PhysicalRecoveryCoordination {
    /// Requires `checkpoint.current` to be byte-identical to the checkpoint
    /// this recovery selected, then synchronizes its namespace. Repeating it
    /// is harmless, so an interrupted recovery repeats it.
    pub fn synchronize_selected_checkpoint_publication(
        &self,
        media: &AdmittedRecoveryFilesystemMedia,
        selected: &SharedRecoveryCheckpoint,
    ) -> Result<(), SelectedCheckpointPublicationDenial> {
        let facts = selected.facts();
        if !selected.matches_owner(&self.residency)
            || self.require_selected_checkpoint(&facts).is_err()
            || facts.source().identity().store_identity() != media.store_identity()
        {
            return Err(SelectedCheckpointPublicationDenial::CheckpointMismatch);
        }
        let current = media
            .observe_current_checkpoint_for_residue(facts.encoded_bytes().saturating_add(1))
            .map_err(SelectedCheckpointPublicationDenial::Observation)?;
        if current.byte_count() != facts.encoded_bytes()
            || current.digest() != facts.encoded_digest()
        {
            return Err(SelectedCheckpointPublicationDenial::CheckpointMismatch);
        }
        media
            .synchronize_recovery_checkpoint_publication()
            .map_err(SelectedCheckpointPublicationDenial::Synchronization)
    }
}
