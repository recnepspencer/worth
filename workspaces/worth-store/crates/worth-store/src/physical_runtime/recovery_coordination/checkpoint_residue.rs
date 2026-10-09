//! Purpose-specific removal of the one unselected checkpoint candidate that
//! would otherwise collide with the next exact checkpoint sequence.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, ArtifactTreeFailure, BackendRecoveryArtifactExpectation,
    BackendRecoveryCleanupRemovalRequest,
};
use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::VerifiedCheckpointFacts;

use super::{PhysicalRecoveryCoordination, SharedRecoveryCheckpoint};

mod execution;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCheckpointResidueDenial {
    SelectedCheckpointMismatch,
    Observation(ArtifactTreeFailure),
    Admission,
    Scheduler,
    Backend(ArtifactTreeFailure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCheckpointResidueOutcome {
    Absent,
    Removed {
        checkpoint: PhysicalCheckpointIdentity,
        candidate_digest: [u8; 32],
    },
    DeniedBeforeEffect(RecoveryCheckpointResidueDenial),
    Indeterminate,
}

struct AdmittedCheckpointResidue {
    checkpoint: PhysicalCheckpointIdentity,
    candidate_digest: [u8; 32],
    request: BackendRecoveryCleanupRemovalRequest,
}

impl PhysicalRecoveryCoordination {
    /// Only the exact next-sequence staging candidate can be discarded. The
    /// selected stream is C9-admitted, but its bytes are checked again against
    /// the actual current slot before the effect and once more by C.4 at unlink.
    pub fn remove_unselected_checkpoint_candidate(
        &self,
        media: &AdmittedRecoveryFilesystemMedia,
        selected: &SharedRecoveryCheckpoint,
        maximum_candidate_bytes: u64,
    ) -> RecoveryCheckpointResidueOutcome {
        if !selected.matches_owner(&self.residency)
            || self.require_selected_checkpoint(&selected.facts()).is_err()
        {
            return RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(
                RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch,
            );
        }
        let admitted = match admit(self, media, &selected.facts(), maximum_candidate_bytes) {
            Ok(Some(admitted)) => admitted,
            Ok(None) => return RecoveryCheckpointResidueOutcome::Absent,
            Err(denial) => return RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(denial),
        };
        execution::remove(self, media, admitted)
    }
}

fn admit(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    selected: &VerifiedCheckpointFacts,
    maximum_candidate_bytes: u64,
) -> Result<Option<AdmittedCheckpointResidue>, RecoveryCheckpointResidueDenial> {
    let checkpoint = selected.source().identity();
    if checkpoint.store_identity() != media.store_identity()
        || checkpoint.store_identity() != coordination.store
        || checkpoint.sequence().get().checked_add(1).is_none()
    {
        return Err(RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch);
    }
    let current = media
        .observe_current_checkpoint_for_residue(selected.encoded_bytes().saturating_add(1))
        .map_err(RecoveryCheckpointResidueDenial::Observation)?;
    if current.byte_count() != selected.encoded_bytes()
        || current.digest() != selected.encoded_digest()
    {
        return Err(RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch);
    }
    let candidate = media
        .observe_next_checkpoint_candidate(checkpoint.sequence().get(), maximum_candidate_bytes)
        .map_err(RecoveryCheckpointResidueDenial::Observation)?;
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.checkpoint-residue.v1");
    digest.update(checkpoint.store_identity().bytes());
    digest.update(checkpoint.sequence().get().to_le_bytes());
    digest.update(selected.encoded_bytes().to_le_bytes());
    digest.update(selected.encoded_digest());
    digest.update(candidate.byte_count().to_le_bytes());
    digest.update(candidate.digest());
    let plan: [u8; 32] = digest.finalize().into();
    let request = BackendRecoveryCleanupRemovalRequest::new(
        media.store_identity(),
        coordination.session_identity(),
        plan,
        BackendRecoveryArtifactExpectation::from_checkpoint_observation(&current),
        BackendRecoveryArtifactExpectation::from_checkpoint_observation(&candidate),
        plan,
    )
    .ok_or(RecoveryCheckpointResidueDenial::Admission)?;
    Ok(Some(AdmittedCheckpointResidue {
        checkpoint,
        candidate_digest: candidate.digest(),
        request,
    }))
}
