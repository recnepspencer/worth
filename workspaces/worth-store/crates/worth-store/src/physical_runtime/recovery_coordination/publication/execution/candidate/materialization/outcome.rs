use crate::physical_runtime::recovery_coordination::CompletedPhysicalRecoveryPublicationCandidate;
use crate::physical_runtime::PhysicalWorkSchedulerPosture;

use super::super::super::super::{
    PhysicalRecoveryPublicationCommandDenial, PhysicalRecoveryPublicationCommandDenialKind,
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
};

pub(super) fn denied(
    stage: PhysicalRecoveryPublicationCommandStage,
    completed: Vec<CompletedPhysicalRecoveryPublicationCandidate>,
    scheduler: Option<PhysicalWorkSchedulerPosture>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
        PhysicalRecoveryPublicationCommandDenial::new(
            stage,
            PhysicalRecoveryPublicationCommandDenialKind::Submission,
            completed.into_boxed_slice(),
            None,
            None,
            scheduler,
        ),
    )
}

pub(super) fn attach_completed(
    outcome: PhysicalRecoveryPublicationCommandOutcome,
    completed: Vec<CompletedPhysicalRecoveryPublicationCandidate>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    match outcome {
        PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(denial) => {
            PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
                PhysicalRecoveryPublicationCommandDenial::new(
                    denial.stage(),
                    denial.denial(),
                    completed.into_boxed_slice(),
                    None,
                    None,
                    denial.scheduler_posture(),
                ),
            )
        }
        other => other,
    }
}
