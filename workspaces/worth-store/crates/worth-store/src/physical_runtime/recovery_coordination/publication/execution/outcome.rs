use crate::physical_runtime::recovery_coordination::{
    PerformedRecoveryPhysicalEffect, RecoveryRootProtocolReplacementAction,
};
use crate::physical_runtime::PhysicalWorkSchedulerPosture;

use super::super::{
    CompletedPhysicalRecoveryPublicationCandidate, PhysicalRecoveryPublicationCommandDenial,
    PhysicalRecoveryPublicationCommandDenialKind, PhysicalRecoveryPublicationCommandOutcome,
    PhysicalRecoveryPublicationCommandStage,
};

pub(super) fn pre_effect(
    stage: PhysicalRecoveryPublicationCommandStage,
    denial: crate::physical_runtime::PhysicalWorkPreEffectDenial,
    candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    root_protocol: Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    denied(
        stage,
        PhysicalRecoveryPublicationCommandDenialKind::PreEffect(denial),
        candidates,
        root_protocol,
        None,
    )
}

pub(super) fn attach_effects(
    outcome: PhysicalRecoveryPublicationCommandOutcome,
    candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    root_protocol: PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    match outcome {
        PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(denial) => denied(
            denial.stage(),
            denial.denial(),
            candidates,
            Some(root_protocol),
            denial.scheduler_posture(),
        ),
        other => other,
    }
}

pub(super) fn attach_candidates(
    outcome: PhysicalRecoveryPublicationCommandOutcome,
    candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    match outcome {
        PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(denial) => {
            let scheduler = denial.scheduler_posture();
            denied(
                denial.stage(),
                denial.denial(),
                candidates,
                denial.root_protocol,
                scheduler,
            )
        }
        other => other,
    }
}

pub(super) fn denied(
    stage: PhysicalRecoveryPublicationCommandStage,
    denial: PhysicalRecoveryPublicationCommandDenialKind,
    candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    root_protocol: Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
    scheduler: Option<PhysicalWorkSchedulerPosture>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
        PhysicalRecoveryPublicationCommandDenial::new(
            stage,
            denial,
            candidates,
            None,
            root_protocol,
            scheduler,
        ),
    )
}
