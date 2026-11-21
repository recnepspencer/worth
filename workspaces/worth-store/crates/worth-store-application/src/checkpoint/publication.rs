use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store::physical_runtime::{
    AnchoredRecordPublicationFailure, AnchoredRecordReadFailure, AnchoredRecordsDenial,
};

#[derive(Debug)]
pub enum ApplicationHomeCheckpointDenial {
    Maintenance(super::ApplicationHomeCheckpointMaintenanceDenial),
    /// Pre-effect unavailable: no Native section, capture, or checkpoint task
    /// ever existed for this failed construction.
    NativeSectionNeverAttached,
    /// Native readmission or pre-service Query initialization is incomplete;
    /// no capture or checkpoint task is attempted by preparation cleanup.
    HomePreparationIncomplete,
    ReadSourceBusy,
    Payload(worth_execution::ExecutionRequestArrayDenial),
    WorldSectionRequired,
    SignalSectionRequired,
    Signal(Box<worth_signal::facade::durability::SignalDurableAdvanceDenial>),
    CoverageAheadOfJournal,
    JournalDescriptionMismatch,
    CheckpointReadProtection(worth_store::physical_runtime::PhysicalCheckpointReadProtectionDenial),
    World(worth_runtime_world::facade::WorldDurableMediumDenial),
    WorldReadmission(worth_runtime_world::facade::WorldReadmissionDenial),
    JournalUnavailable(worth_store::physical_runtime::OrderedJournalDenial),
    JournalRead(worth_store::physical_runtime::OrderedJournalReadFailure),
    Unavailable(AnchoredRecordsDenial),
    Read(AnchoredRecordReadFailure),
    Manifest(super::HomeManifestDenial),
    Native(worth_store_relational::RelationalSectionCheckpointDenial),
    Execution(WorkCeilingDenial),
    InterruptionView(worth_execution::ExecutionInterruptionViewDenial),
    Deadline(crate::ApplicationHomeDeadlineDenial),
    SizeLimit,
    NoPendingCheckpoint,
    RecoveryRequired(super::ApplicationHomeCheckpointRecovery),
    Publication(AnchoredRecordPublicationFailure),
    PhysicalPublication(worth_store::physical_runtime::AnchoredRecordOperationFailure),
    StoppedBeforeSelection {
        prefix: Vec<worth_store::physical_runtime::AnchoredRecordMemberLocator>,
    },
    PhysicalCheckpoint {
        detail: String,
    },
    PhysicalCheckpointOutcome(worth_store::physical_runtime::PhysicalCheckpointOutcome),
}

impl ApplicationHomeCheckpointDenial {
    /// The original request's typed control stop behind this refusal, whichever
    /// layer reported it: the capture request, the World medium or readmission,
    /// the Native section or its payload.
    pub fn control_stop(&self) -> Option<worth_execution::MapKernelStop> {
        use worth_execution::{ExecutionRequestArrayDenial as Array, MapKernelStop as Stop};
        use worth_relational::facade::durability::RelationalNativeCheckpointCaptureDenial as Capture;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
        use worth_runtime_world::facade::{
            WorldDurableMediumDenial as World, WorldReadmissionDenial as Readmission,
        };
        use worth_store_relational::RelationalSectionCheckpointDenial as Section;
        let stopped = |work: &WorkCeilingDenial| match work {
            WorkCeilingDenial::Stopped(stop) => Some(*stop),
            _ => None,
        };
        match self {
            Self::Execution(work)
            | Self::Native(Section::Execution(work))
            | Self::Payload(Array::Request(work))
            | Self::World(World::Request(work)) => stopped(work),
            Self::WorldReadmission(readmission) => {
                let mut readmission = readmission;
                loop {
                    match readmission {
                        Readmission::Medium(World::Request(work)) => break stopped(work),
                        Readmission::NativeCustody { cause, .. } => readmission = cause.as_ref(),
                        _ => break None,
                    }
                }
            }
            Self::Native(Section::Capture(Capture::Request(cause))) => match cause {
                Cause::Cancelled => Some(Stop::Cancelled),
                Cause::DeadlineElapsed => Some(Stop::DeadlineElapsed),
                Cause::WorkCounterOverflow => Some(Stop::WorkCounterOverflow),
                Cause::AccountingUnavailable => Some(Stop::AccountingUnavailable),
                Cause::WorkExhausted => Some(Stop::WorkCeiling),
                Cause::NestedStopped => Some(Stop::NestedStopped),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ApplicationHomeCheckpoint {
    pub covered: PatchStreamPosition,
    pub covered_transitions: RelationalBranchTransitionSequence,
    pub generation: u64,
    pub bytes: usize,
}

impl ApplicationHomeCheckpoint {
    pub fn covered(&self) -> PatchStreamPosition {
        self.covered
    }
    pub fn covered_transitions(&self) -> RelationalBranchTransitionSequence {
        self.covered_transitions
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

impl crate::ReadmittedApplicationHome {
    pub fn checkpoint(
        &self,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
        Err(ApplicationHomeCheckpointDenial::WorldSectionRequired)
    }

    /// Finish only the original accepted physical step and Native acknowledgement.
    /// A fresh request never restarts members or anchor publication.
    pub fn recover_checkpoint(
        &self,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
        Err(ApplicationHomeCheckpointDenial::WorldSectionRequired)
    }

    pub fn checkpoint_observation(&self) -> Option<super::ApplicationHomeCheckpointRecovery> {
        self.context
            .checkpoint
            .pending()
            .map(|attempt| attempt.observation())
    }
}

/// Preserve the original pre-effect worldless refusal order without admitting
/// an ordinary Native capture or fabricating a missing World capability.
pub(crate) fn worldless_checkpoint(
    owner: &super::owner::HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    if let Some(attempt) = owner.pending() {
        return Err(ApplicationHomeCheckpointDenial::RecoveryRequired(
            attempt.observation(),
        ));
    }
    execution
        .consult()
        .map_err(ApplicationHomeCheckpointDenial::Execution)?;
    Err(ApplicationHomeCheckpointDenial::WorldSectionRequired)
}
