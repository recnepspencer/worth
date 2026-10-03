use super::PhysicalCheckpointActionFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalCheckpointCaptureFailureKind {
    SequenceExhausted,
    CheckpointCustodyUnavailable,
    CheckpointCustodyBacking(crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial),
    RuntimeUnavailable,
    NoDurableWalSource,
    SourceAuthorityMismatch,
    ResidencyUnavailable,
    ResidencyAdmission(worth_store_buffer_pool::PhysicalResidencyDenial),
    RetainedWalTailUnavailable,
    RetainedWalTailLimitExceeded,
    BindingCompactionUnavailable,
    BindingCompactionCommitFailed,
    WorkSubmissionUnavailable,
    DependencyBlocked,
    SchedulerCapacityUnavailable,
    BackgroundYielded,
    BackgroundDeferred,
    BackgroundDenied,
    BackgroundThrottled,
    BackgroundViolation,
    SchedulerDemandRejected,
    QueueAdmissionRejected,
    CommandRejected,
    MediaDeniedBeforeEffect,
    CheckpointActionIndeterminate,
    CandidateContinuationFailed,
    WorkerPanicked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PhysicalCheckpointCaptureFailurePosture {
    NoCandidateCreated,
    CandidateInspectionRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PhysicalCheckpointCaptureFailure {
    kind: PhysicalCheckpointCaptureFailureKind,
    posture: PhysicalCheckpointCaptureFailurePosture,
}

impl PhysicalCheckpointCaptureFailure {
    pub(super) const fn before_candidate(kind: PhysicalCheckpointCaptureFailureKind) -> Self {
        Self {
            kind,
            posture: PhysicalCheckpointCaptureFailurePosture::NoCandidateCreated,
        }
    }

    pub(super) const fn candidate_requires_inspection(
        kind: PhysicalCheckpointCaptureFailureKind,
    ) -> Self {
        Self {
            kind,
            posture: PhysicalCheckpointCaptureFailurePosture::CandidateInspectionRequired,
        }
    }

    pub(super) fn from_initial_action(failure: PhysicalCheckpointActionFailure) -> Self {
        use PhysicalCheckpointActionFailure as Action;
        let kind = match failure {
            Action::RuntimeReleased => PhysicalCheckpointCaptureFailureKind::RuntimeUnavailable,
            Action::SubmissionDenied
            | Action::SubmissionDeferred
            | Action::SubmissionStale
            | Action::SubmissionFailed
            | Action::PreEffect => PhysicalCheckpointCaptureFailureKind::WorkSubmissionUnavailable,
            Action::DependencyBlocked => PhysicalCheckpointCaptureFailureKind::DependencyBlocked,
            Action::SchedulerCapacityUnavailable => {
                PhysicalCheckpointCaptureFailureKind::SchedulerCapacityUnavailable
            }
            Action::BackgroundYielded => PhysicalCheckpointCaptureFailureKind::BackgroundYielded,
            Action::BackgroundDeferred => PhysicalCheckpointCaptureFailureKind::BackgroundDeferred,
            Action::BackgroundDenied => PhysicalCheckpointCaptureFailureKind::BackgroundDenied,
            Action::BackgroundThrottled => {
                PhysicalCheckpointCaptureFailureKind::BackgroundThrottled
            }
            Action::BackgroundViolation => {
                PhysicalCheckpointCaptureFailureKind::BackgroundViolation
            }
            Action::SchedulerDemandRejected => {
                PhysicalCheckpointCaptureFailureKind::SchedulerDemandRejected
            }
            Action::QueueAdmissionRejected => {
                PhysicalCheckpointCaptureFailureKind::QueueAdmissionRejected
            }
            Action::Command => PhysicalCheckpointCaptureFailureKind::CommandRejected,
            Action::MediaDeniedBeforeEffect(_media) => {
                PhysicalCheckpointCaptureFailureKind::MediaDeniedBeforeEffect
            }
            Action::EffectRequiresInspection | Action::StaleOrForeignSettlement => {
                return Self::candidate_requires_inspection(
                    PhysicalCheckpointCaptureFailureKind::CheckpointActionIndeterminate,
                );
            }
        };
        Self::before_candidate(kind)
    }

    pub(super) fn kind(&self) -> PhysicalCheckpointCaptureFailureKind {
        self.kind.clone()
    }

    pub(super) const fn requires_inspection(&self) -> bool {
        matches!(
            self.posture,
            PhysicalCheckpointCaptureFailurePosture::CandidateInspectionRequired
        )
    }
}
