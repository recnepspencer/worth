//! Removal of root-protocol candidates a killed runtime left in record
//! staging. Recovery runs it after its own publication is namespace durable;
//! each removal is an admitted, scheduled publication effect, and an already
//! absent candidate completes, so an interrupted pass can be repeated.

use crate::physical_runtime::{PhysicalPublicationEffect, PhysicalWorkScope};

use super::super::{
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
};
use super::{admission, effect};
use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStagingResidueDenial {
    /// Record staging could not be listed, or held an entry that is not a
    /// root-protocol candidate.
    Observation(worth_store_physical_backend::ArtifactTreeFailure),
    /// A removal was refused before its effect.
    RemovalRefused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStagingResidueOutcome {
    Absent,
    Removed { candidates: u64 },
    DeniedBeforeEffect(RecoveryStagingResidueDenial),
    Indeterminate,
}

impl PhysicalRecoveryCoordination {
    /// Removes every stale record-staging candidate, at most `maximum_entries`.
    pub fn remove_record_staging_residue(
        &self,
        media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
        maximum_entries: usize,
    ) -> RecoveryStagingResidueOutcome {
        let candidates = match media.list_recovery_record_staging_candidates(maximum_entries) {
            Ok(candidates) => candidates,
            Err(failure) => {
                return RecoveryStagingResidueOutcome::DeniedBeforeEffect(
                    RecoveryStagingResidueDenial::Observation(failure),
                )
            }
        };
        if candidates.is_empty() {
            return RecoveryStagingResidueOutcome::Absent;
        }
        for (removed, candidate) in candidates.iter().enumerate() {
            if let Err(outcome) = remove(self, media, *candidate) {
                return match outcome {
                    PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(_)
                        if removed == 0 =>
                    {
                        RecoveryStagingResidueOutcome::DeniedBeforeEffect(
                            RecoveryStagingResidueDenial::RemovalRefused,
                        )
                    }
                    _ => RecoveryStagingResidueOutcome::Indeterminate,
                };
            }
        }
        RecoveryStagingResidueOutcome::Removed {
            candidates: candidates.len() as u64,
        }
    }
}

fn remove(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    candidate: worth_store_physical_format::RecordArtifactFile,
) -> Result<(), PhysicalRecoveryPublicationCommandOutcome> {
    let stage = PhysicalRecoveryPublicationCommandStage::StagingResidueRemoval;
    let work = admission::admit(coordination, stage, PhysicalWorkScope::artifact(candidate))?;
    let (dispatched, plan) = work
        .into_execution_parts(None)
        .map_err(|denial| super::outcome::pre_effect(stage, denial, Box::new([]), None))?;
    let physical = media.remove_recovery_staging_candidate_scheduled(
        candidate,
        plan.backend_completion_binding()
            .backend_execution_binding(),
    );
    effect::complete(
        coordination,
        dispatched,
        plan,
        physical,
        stage,
        candidate,
        PhysicalPublicationEffect::RemoveArtifact,
        Box::new([]),
        None,
    )
    .map(|_| ())
}
