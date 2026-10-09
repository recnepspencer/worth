use worth_store_physical_format::RecordArtifactFile;

mod candidate_write;
mod projection;

pub use candidate_write::{
    PhysicalRootCandidateWriteFailureCause, PhysicalRootCandidateWriteFailurePosture,
};
use projection::{failure_cause, failure_group, failure_members, failure_planning};

use crate::physical_runtime::record_serving::{
    RejectedSettledRootProjections, RootCandidateWriteFailureKind,
};
use crate::physical_runtime::{
    CandidateFrameContractViolation, DataSettledPhysicalMutationMembers,
    PhysicalDurabilityGroupBasis, PhysicalRootPublicationMemberIdentity, RecordAppendError,
    RootPublicationCandidatePlan, RootPublicationPhysicalMutationMember,
    RootPublicationPlanningMembers, RootPublicationPreparedPhysicalMutationMembers,
    SettledPhysicalWork, SettledRootProjectionMergeDenial, WrittenRootPublicationCandidate,
};

use super::{
    PhysicalRootPublicationTransitionDenial, PhysicalRootPublicationWorkFailure,
    PhysicalRootPublicationWorkFailureCause,
};

pub enum PhysicalRootPublicationPreparationOutcome {
    Prepared(RootPublicationPreparedPhysicalMutationMembers),
    NotStarted(PhysicalRootPublicationPreparationNotStarted),
    InspectionRequired(IndeterminatePhysicalRootPublicationPreparation),
}

pub struct PhysicalRootPublicationPreparationNotStarted {
    failure: PhysicalRootPublicationPreparationFailure,
}

pub struct IndeterminatePhysicalRootPublicationPreparation {
    failure: PhysicalRootPublicationPreparationFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRootPublicationPreparationFailureCause {
    RuntimeReleased,
    CurrentRootMismatch,
    ProjectionRejected(SettledRootProjectionMergeDenial),
    TransitionDenied(PhysicalRootPublicationTransitionDenial),
    Planning(RecordAppendError),
    CandidateAdmission(RecordAppendError),
    CandidateWrite {
        candidate_generation: u64,
        failed_artifact: RecordArtifactFile,
        completed_artifact_count: usize,
        cause: PhysicalRootCandidateWriteFailureCause,
    },
    CandidateFrameSetIncomplete(CandidateFrameContractViolation),
    CandidateSynchronization {
        artifact: RecordArtifactFile,
        completed_synchronization_count: usize,
        cause: PhysicalRootCandidateSynchronizationFailureCause,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootCandidateSynchronizationFailureCause {
    Work(PhysicalRootPublicationWorkFailureCause),
    Settlement {
        fate: crate::physical_runtime::PhysicalWorkEffectFate,
        recovery: crate::physical_runtime::PhysicalWorkRecoveryDisposition,
    },
}

#[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
pub(in crate::physical_runtime) enum PhysicalRootPublicationPreparationFailure {
    NotStarted {
        settled: DataSettledPhysicalMutationMembers,
        cause: PhysicalRootPublicationPreparationNotStartedCause,
    },
    ProjectionRejected {
        planning: RootPublicationPlanningMembers,
        rejected: RejectedSettledRootProjections,
    },
    PlanningAuthorityReleased {
        planning: RootPublicationPlanningMembers,
    },
    TransitionDenied {
        planning: RootPublicationPlanningMembers,
        cause: PhysicalRootPublicationTransitionDenial,
    },
    Planning {
        planning: RootPublicationPlanningMembers,
        cause: RecordAppendError,
    },
    CandidateAuthorityReleased {
        candidate: RootPublicationCandidatePlan,
    },
    CandidateAdmission {
        candidate: RootPublicationCandidatePlan,
        cause: RecordAppendError,
    },
    CandidateWriteNotStarted {
        candidate: RootPublicationCandidatePlan,
        failed_artifact: RecordArtifactFile,
        cause: RootCandidateWriteFailureKind,
    },
    CandidateWrite {
        candidate: RootPublicationCandidatePlan,
        completed_artifacts: Box<[RecordArtifactFile]>,
        failed_artifact: RecordArtifactFile,
        cause: RootCandidateWriteFailureKind,
    },
    CandidateFrameSetIncomplete {
        candidate: WrittenRootPublicationCandidate,
        violation: CandidateFrameContractViolation,
    },
    CandidateSynchronization {
        candidate: WrittenRootPublicationCandidate,
        completed: Box<[SettledPhysicalWork]>,
        artifact: RecordArtifactFile,
        cause: RootCandidateSynchronizationFailure,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalRootPublicationPreparationNotStartedCause {
    RuntimeReleased,
    CurrentRootMismatch,
}

pub(in crate::physical_runtime) enum RootCandidateSynchronizationFailure {
    Work(PhysicalRootPublicationWorkFailure),
    Settlement {
        fate: crate::physical_runtime::PhysicalWorkEffectFate,
        recovery: crate::physical_runtime::PhysicalWorkRecoveryDisposition,
    },
}

impl PhysicalRootPublicationPreparationOutcome {
    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(in crate::physical_runtime) fn runtime_released(
        settled: DataSettledPhysicalMutationMembers,
    ) -> Self {
        Self::NotStarted(PhysicalRootPublicationPreparationNotStarted {
            failure: PhysicalRootPublicationPreparationFailure::NotStarted {
                settled,
                cause: PhysicalRootPublicationPreparationNotStartedCause::RuntimeReleased,
            },
        })
    }

    pub(in crate::physical_runtime) fn from_result(
        result: Result<
            RootPublicationPreparedPhysicalMutationMembers,
            PhysicalRootPublicationPreparationFailure,
        >,
    ) -> Self {
        match result {
            Ok(prepared) => Self::Prepared(prepared),
            Err(failure @ PhysicalRootPublicationPreparationFailure::CandidateWrite { .. })
            | Err(
                failure @ PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete {
                    ..
                },
            )
            | Err(
                failure @ PhysicalRootPublicationPreparationFailure::CandidateSynchronization {
                    ..
                },
            ) => Self::InspectionRequired(IndeterminatePhysicalRootPublicationPreparation {
                failure,
            }),
            Err(failure) => {
                Self::NotStarted(PhysicalRootPublicationPreparationNotStarted { failure })
            }
        }
    }
}

impl PhysicalRootPublicationPreparationNotStarted {
    pub fn cause(&self) -> PhysicalRootPublicationPreparationFailureCause {
        failure_cause(&self.failure)
    }

    pub fn settled_members(&self) -> Option<&DataSettledPhysicalMutationMembers> {
        match &self.failure {
            PhysicalRootPublicationPreparationFailure::NotStarted { settled, .. } => Some(settled),
            PhysicalRootPublicationPreparationFailure::ProjectionRejected { .. }
            | PhysicalRootPublicationPreparationFailure::PlanningAuthorityReleased { .. }
            | PhysicalRootPublicationPreparationFailure::TransitionDenied { .. }
            | PhysicalRootPublicationPreparationFailure::Planning { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateAdmission { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWrite { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateSynchronization { .. } => None,
        }
    }

    pub fn planning_members(&self) -> Option<&RootPublicationPlanningMembers> {
        failure_planning(&self.failure)
    }

    pub fn into_planning_members(self) -> Option<RootPublicationPlanningMembers> {
        match self.failure {
            PhysicalRootPublicationPreparationFailure::ProjectionRejected { planning, .. }
            | PhysicalRootPublicationPreparationFailure::PlanningAuthorityReleased { planning }
            | PhysicalRootPublicationPreparationFailure::TransitionDenied { planning, .. }
            | PhysicalRootPublicationPreparationFailure::Planning { planning, .. } => {
                Some(planning)
            }
            PhysicalRootPublicationPreparationFailure::NotStarted { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateAdmission { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWrite { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateSynchronization { .. } => None,
        }
    }

    pub fn into_settled_members(self) -> Option<DataSettledPhysicalMutationMembers> {
        match self.failure {
            PhysicalRootPublicationPreparationFailure::NotStarted { settled, .. } => Some(settled),
            PhysicalRootPublicationPreparationFailure::ProjectionRejected { .. }
            | PhysicalRootPublicationPreparationFailure::PlanningAuthorityReleased { .. }
            | PhysicalRootPublicationPreparationFailure::TransitionDenied { .. }
            | PhysicalRootPublicationPreparationFailure::Planning { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateAdmission { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWrite { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete { .. }
            | PhysicalRootPublicationPreparationFailure::CandidateSynchronization { .. } => None,
        }
    }

    pub fn group_basis(&self) -> Option<PhysicalDurabilityGroupBasis> {
        failure_group(&self.failure)
    }

    pub fn candidate_plan(&self) -> Option<&RootPublicationCandidatePlan> {
        match &self.failure {
            PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { candidate }
            | PhysicalRootPublicationPreparationFailure::CandidateAdmission { candidate, .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted {
                candidate,
                ..
            } => Some(candidate),
            _ => None,
        }
    }

    pub fn into_candidate_plan(self) -> Option<RootPublicationCandidatePlan> {
        match self.failure {
            PhysicalRootPublicationPreparationFailure::CandidateAuthorityReleased { candidate }
            | PhysicalRootPublicationPreparationFailure::CandidateAdmission { candidate, .. }
            | PhysicalRootPublicationPreparationFailure::CandidateWriteNotStarted {
                candidate,
                ..
            } => Some(candidate),
            _ => None,
        }
    }

    pub fn member_identities(&self) -> &[PhysicalRootPublicationMemberIdentity] {
        failure_members(&self.failure)
    }

    pub fn planning_error(&self) -> Option<&RecordAppendError> {
        match &self.failure {
            PhysicalRootPublicationPreparationFailure::Planning { cause, .. }
            | PhysicalRootPublicationPreparationFailure::CandidateAdmission { cause, .. } => {
                Some(cause)
            }
            _ => None,
        }
    }
}

impl IndeterminatePhysicalRootPublicationPreparation {
    pub fn cause(&self) -> PhysicalRootPublicationPreparationFailureCause {
        failure_cause(&self.failure)
    }

    pub fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        failure_group(&self.failure)
            .expect("inspection-required root preparation has consumed one exact group")
    }

    pub fn member_identities(&self) -> &[PhysicalRootPublicationMemberIdentity] {
        failure_members(&self.failure)
    }

    pub fn settled_members(&self) -> &[RootPublicationPhysicalMutationMember] {
        match &self.failure {
            PhysicalRootPublicationPreparationFailure::CandidateWrite { candidate, .. } => {
                candidate.settled_members()
            }
            PhysicalRootPublicationPreparationFailure::CandidateFrameSetIncomplete {
                candidate,
                ..
            }
            | PhysicalRootPublicationPreparationFailure::CandidateSynchronization {
                candidate,
                ..
            } => candidate.settled_members(),
            _ => unreachable!("inspection-required preparation retains exact candidate state"),
        }
    }

    pub fn candidate_artifacts(&self) -> Option<&[RecordArtifactFile]> {
        match &self.failure {
            PhysicalRootPublicationPreparationFailure::CandidateSynchronization {
                candidate,
                ..
            } => Some(candidate.candidate().artifacts()),
            _ => None,
        }
    }
}
