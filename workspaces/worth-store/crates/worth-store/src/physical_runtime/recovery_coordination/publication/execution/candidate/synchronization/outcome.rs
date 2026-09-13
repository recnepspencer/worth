use crate::physical_runtime::recovery_coordination::{
    CompletedPhysicalRecoveryPublicationCandidate,
    PhysicalRecoveryPublicationCandidateMaterialization,
};
use crate::physical_runtime::PhysicalWorkSchedulerPosture;

use super::super::super::super::{
    PhysicalRecoveryPublicationCommandDenial, PhysicalRecoveryPublicationCommandDenialKind,
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
};

pub(super) fn denied(
    stage: PhysicalRecoveryPublicationCommandStage,
    completed: Vec<CompletedPhysicalRecoveryPublicationCandidate>,
    materialization: PhysicalRecoveryPublicationCandidateMaterialization,
    scheduler: Option<PhysicalWorkSchedulerPosture>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
        PhysicalRecoveryPublicationCommandDenial::new(
            stage,
            PhysicalRecoveryPublicationCommandDenialKind::Submission,
            completed.into_boxed_slice(),
            Some(materialization),
            None,
            scheduler,
        ),
    )
}

pub(super) fn attach_materialization(
    outcome: PhysicalRecoveryPublicationCommandOutcome,
    completed: Vec<CompletedPhysicalRecoveryPublicationCandidate>,
    materialization: PhysicalRecoveryPublicationCandidateMaterialization,
) -> PhysicalRecoveryPublicationCommandOutcome {
    match outcome {
        PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(denial) => {
            PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
                PhysicalRecoveryPublicationCommandDenial::new(
                    denial.stage(),
                    denial.denial(),
                    completed.into_boxed_slice(),
                    Some(materialization),
                    None,
                    denial.scheduler_posture(),
                ),
            )
        }
        other => other,
    }
}
