use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalRootPublicationMemberIdentity,
    RootPublicationPlanningMembers,
};

use super::candidate_write::project_candidate_write_failure;
use super::{
    PhysicalRootCandidateSynchronizationFailureCause, PhysicalRootPublicationPreparationFailure,
    PhysicalRootPublicationPreparationFailureCause,
    PhysicalRootPublicationPreparationNotStartedCause, RootCandidateSynchronizationFailure,
};

pub(super) fn failure_cause(
    failure: &PhysicalRootPublicationPreparationFailure,
) -> PhysicalRootPublicationPreparationFailureCause {
    match failure {
        PhysicalRootPublicationPreparationFailure::NotStarted { cause, .. } => match cause {
            PhysicalRootPublicationPreparationNotStartedCause::RuntimeReleased => {
                PhysicalRootPublicationPreparationFailureCause::RuntimeReleased
            }
            PhysicalRootPublicationPreparationNotStartedCause::CurrentRootMismatch => {
                PhysicalRootPublicationPreparationFailureCause::CurrentRootMismatch
            }
        },
        PhysicalRootPublicationPreparationFailure::ProjectionRejected { rejected, .. } => {
            PhysicalRootPublicationPreparationFailureCause::ProjectionRejected(rejected.cause())
        }
        PhysicalRootPublicationPreparationFailure::PlanningAuthorityReleased { .. }
        | PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { .. } => {
            PhysicalRootPublicationPreparationFailureCause::RuntimeReleased
        }
        PhysicalRootPublicationPreparationFailure::TransitionDenied { cause, .. } => {
            PhysicalRootPublicationPreparationFailureCause::TransitionDenied(*cause)
        }
        PhysicalRootPublicationPreparationFailure::Planning { cause, .. } => {
            PhysicalRootPublicationPreparationFailureCause::Planning(cause.clone())
        }
        PhysicalRootPublicationPreparationFailure::CandidateAdmission { cause, .. } => {
            PhysicalRootPublicationPreparationFailureCause::CandidateAdmission(cause.clone())
        }
        PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted {
            candidate,
            failed_artifact,
            cause,
        } => PhysicalRootPublicationPreparationFailureCause::CandidateWrite {
            candidate_generation: candidate.candidate_generation(),
            failed_artifact: *failed_artifact,
            completed_artifact_count: 0,
            cause: project_candidate_write_failure(cause.clone()),
        },
        PhysicalRootPublicationPreparationFailure::CandidateWrite {
            candidate,
            completed_artifacts,
            failed_artifact,
            cause,
        } => PhysicalRootPublicationPreparationFailureCause::CandidateWrite {
            candidate_generation: candidate.candidate_generation(),
            failed_artifact: *failed_artifact,
            completed_artifact_count: completed_artifacts.len(),
            cause: project_candidate_write_failure(cause.clone()),
        },
        PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete {
            violation,
            ..
        } => {
            PhysicalRootPublicationPreparationFailureCause::CandidateFrameSetIncomplete(*violation)
        }
        PhysicalRootPublicationPreparationFailure::CandidateSynchronization {
            artifact,
            completed,
            cause,
            ..
        } => PhysicalRootPublicationPreparationFailureCause::CandidateSynchronization {
            artifact: *artifact,
            completed_synchronization_count: completed.len(),
            cause: match cause {
                RootCandidateSynchronizationFailure::Work(failure) => {
                    PhysicalRootCandidateSynchronizationFailureCause::Work(failure.cause())
                }
                RootCandidateSynchronizationFailure::Settlement { fate, recovery } => {
                    PhysicalRootCandidateSynchronizationFailureCause::Settlement {
                        fate: *fate,
                        recovery: *recovery,
                    }
                }
            },
        },
    }
}

pub(super) fn failure_group(
    failure: &PhysicalRootPublicationPreparationFailure,
) -> Option<PhysicalDurabilityGroupBasis> {
    match failure {
        PhysicalRootPublicationPreparationFailure::NotStarted { settled, .. } => {
            Some(settled.basis())
        }
        PhysicalRootPublicationPreparationFailure::ProjectionRejected { planning, .. }
        | PhysicalRootPublicationPreparationFailure::PlanningAuthorityReleased { planning }
        | PhysicalRootPublicationPreparationFailure::TransitionDenied { planning, .. }
        | PhysicalRootPublicationPreparationFailure::Planning { planning, .. } => {
            Some(planning.group_basis())
        }
        PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { candidate }
        | PhysicalRootPublicationPreparationFailure::CandidateAdmission { candidate, .. }
        | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted {
            candidate, ..
        }
        | PhysicalRootPublicationPreparationFailure::CandidateWrite { candidate, .. } => {
            Some(candidate.group_basis())
        }
        PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete {
            candidate,
            ..
        }
        | PhysicalRootPublicationPreparationFailure::CandidateSynchronization {
            candidate, ..
        } => Some(candidate.group_basis()),
    }
}

pub(super) fn failure_members(
    failure: &PhysicalRootPublicationPreparationFailure,
) -> &[PhysicalRootPublicationMemberIdentity] {
    match failure {
        PhysicalRootPublicationPreparationFailure::NotStarted { .. } => &[],
        PhysicalRootPublicationPreparationFailure::ProjectionRejected { planning, .. }
        | PhysicalRootPublicationPreparationFailure::PlanningAuthorityReleased { planning }
        | PhysicalRootPublicationPreparationFailure::TransitionDenied { planning, .. }
        | PhysicalRootPublicationPreparationFailure::Planning { planning, .. } => {
            planning.member_identities()
        }
        PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { candidate }
        | PhysicalRootPublicationPreparationFailure::CandidateAdmission { candidate, .. }
        | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted {
            candidate, ..
        }
        | PhysicalRootPublicationPreparationFailure::CandidateWrite { candidate, .. } => {
            candidate.member_identities()
        }
        PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete {
            candidate,
            ..
        }
        | PhysicalRootPublicationPreparationFailure::CandidateSynchronization {
            candidate, ..
        } => candidate.member_identities(),
    }
}

pub(super) fn failure_planning(
    failure: &PhysicalRootPublicationPreparationFailure,
) -> Option<&RootPublicationPlanningMembers> {
    match failure {
        PhysicalRootPublicationPreparationFailure::NotStarted { .. }
        | PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { .. }
        | PhysicalRootPublicationPreparationFailure::CandidateAdmission { .. }
        | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted { .. }
        | PhysicalRootPublicationPreparationFailure::CandidateWrite { .. }
        | PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete { .. }
        | PhysicalRootPublicationPreparationFailure::CandidateSynchronization { .. } => None,
        PhysicalRootPublicationPreparationFailure::ProjectionRejected { planning, .. }
        | PhysicalRootPublicationPreparationFailure::PlanningAuthorityReleased { planning }
        | PhysicalRootPublicationPreparationFailure::TransitionDenied { planning, .. }
        | PhysicalRootPublicationPreparationFailure::Planning { planning, .. } => Some(planning),
    }
}
