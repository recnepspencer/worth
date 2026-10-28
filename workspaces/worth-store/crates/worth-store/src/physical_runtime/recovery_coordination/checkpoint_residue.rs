//! Purpose-specific removal of the one unselected checkpoint candidate that
//! would otherwise collide with the next exact checkpoint sequence: candidate
//! `selected + 1`, or candidate 1 when recovery observed no checkpoint.

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, ArtifactTreeFailure, BackendRecoveryArtifactExpectation,
    BackendRecoveryCleanupRemovalRequest,
};
use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::VerifiedCheckpointFacts;

use super::selected_checkpoint::RecoveryCheckpointSelection;
use super::PhysicalRecoveryCoordination;

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
        candidate: PhysicalCheckpointIdentity,
        candidate_digest: [u8; 32],
    },
    DeniedBeforeEffect(RecoveryCheckpointResidueDenial),
    Indeterminate,
}

struct AdmittedCheckpointResidue {
    candidate: PhysicalCheckpointIdentity,
    candidate_digest: [u8; 32],
    request: BackendRecoveryCleanupRemovalRequest,
}

/// What `checkpoint.current` holds for this recovery, checked again on media
/// before the removal is admitted and once more by C.4 at unlink.
enum ResidueCheckpointBasis {
    Selected(VerifiedCheckpointFacts),
    Absent,
}

impl PhysicalRecoveryCoordination {
    /// Only the exact next-sequence staging candidate can be discarded. A
    /// selected stream is C9-admitted, but its bytes are checked again against
    /// the actual current slot; an absent selection must still be absent.
    pub fn remove_unselected_checkpoint_candidate(
        &self,
        media: &AdmittedRecoveryFilesystemMedia,
        maximum_candidate_bytes: u64,
    ) -> RecoveryCheckpointResidueOutcome {
        let basis = match &self.checkpoint_selection {
            RecoveryCheckpointSelection::Selected { shared, .. }
                if shared.matches_owner(&self.residency)
                    && self.require_selected_checkpoint(&shared.facts()).is_ok() =>
            {
                ResidueCheckpointBasis::Selected(shared.facts())
            }
            RecoveryCheckpointSelection::Absent => ResidueCheckpointBasis::Absent,
            _ => {
                return RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(
                    RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch,
                )
            }
        };
        let admitted = match admit(self, media, &basis, maximum_candidate_bytes) {
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
    basis: &ResidueCheckpointBasis,
    maximum_candidate_bytes: u64,
) -> Result<Option<AdmittedCheckpointResidue>, RecoveryCheckpointResidueDenial> {
    if media.store_identity() != coordination.store {
        return Err(RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch);
    }
    let selected_sequence = match basis {
        ResidueCheckpointBasis::Selected(selected) => {
            let identity = selected.source().identity();
            if identity.store_identity() != coordination.store {
                return Err(RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch);
            }
            identity.sequence().get()
        }
        ResidueCheckpointBasis::Absent => 0,
    };
    let next = selected_sequence
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .ok_or(RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch)?;
    let (checkpoint_digest, current) = observe_current(media, basis)?;
    let candidate = media
        .observe_next_checkpoint_candidate(selected_sequence, maximum_candidate_bytes)
        .map_err(RecoveryCheckpointResidueDenial::Observation)?;
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.checkpoint-residue.v2");
    digest.update(coordination.store.bytes());
    digest.update(selected_sequence.to_le_bytes());
    digest.update(checkpoint_digest);
    digest.update(candidate.byte_count().to_le_bytes());
    digest.update(candidate.digest());
    let plan: [u8; 32] = digest.finalize().into();
    let session = coordination.session_identity();
    let expectation = BackendRecoveryArtifactExpectation::from_checkpoint_observation(&candidate);
    let request = match current {
        CurrentCheckpointObservation::Selected(current) => {
            BackendRecoveryCleanupRemovalRequest::new(
                media.store_identity(),
                session,
                plan,
                BackendRecoveryArtifactExpectation::from_checkpoint_observation(&current),
                expectation,
                plan,
            )
        }
        CurrentCheckpointObservation::Absent(current) => {
            BackendRecoveryCleanupRemovalRequest::without_selected_checkpoint(
                media.store_identity(),
                session,
                plan,
                current,
                expectation,
                plan,
            )
        }
    }
    .ok_or(RecoveryCheckpointResidueDenial::Admission)?;
    Ok(Some(AdmittedCheckpointResidue {
        candidate: PhysicalCheckpointIdentity::new(coordination.store, next),
        candidate_digest: candidate.digest(),
        request,
    }))
}

enum CurrentCheckpointObservation {
    Selected(worth_store_physical_backend::ObservedRecoveryCheckpointArtifact),
    Absent(worth_store_physical_backend::ArtifactTreeFile),
}

/// The current slot as observed now, and the digest that binds the removal
/// plan to it: the selected stream's digest, or zeros for an absent slot.
fn observe_current(
    media: &AdmittedRecoveryFilesystemMedia,
    basis: &ResidueCheckpointBasis,
) -> Result<([u8; 32], CurrentCheckpointObservation), RecoveryCheckpointResidueDenial> {
    match basis {
        ResidueCheckpointBasis::Selected(selected) => {
            let current = media
                .observe_current_checkpoint_for_residue(selected.encoded_bytes().saturating_add(1))
                .map_err(RecoveryCheckpointResidueDenial::Observation)?;
            if current.byte_count() != selected.encoded_bytes()
                || current.digest() != selected.encoded_digest()
            {
                return Err(RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch);
            }
            Ok((
                selected.encoded_digest(),
                CurrentCheckpointObservation::Selected(current),
            ))
        }
        ResidueCheckpointBasis::Absent => {
            let current = media
                .observe_absent_checkpoint_for_residue()
                .map_err(RecoveryCheckpointResidueDenial::Observation)?;
            Ok(([0; 32], CurrentCheckpointObservation::Absent(current)))
        }
    }
}
