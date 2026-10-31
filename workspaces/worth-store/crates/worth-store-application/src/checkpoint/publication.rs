use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store::physical_runtime::{
    AnchoredRecordPublicationFailure, AnchoredRecordReadFailure, AnchoredRecordsDenial,
};

#[derive(Debug)]
pub enum ApplicationHomeCheckpointDenial {
    /// Pre-effect unavailable: no Native section, capture, or checkpoint task
    /// ever existed for this failed construction.
    NativeSectionNeverAttached,
    WorldSectionRequired,
    WorldJournalUnavailable(worth_store::physical_runtime::OrderedJournalDenial),
    WorldJournalRead(worth_store::physical_runtime::OrderedJournalReadFailure),
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
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
        checkpoint(
            &self.context.serving,
            &self.context.checkpoint,
            execution,
            || self.relational.prepare_checkpoint(execution),
        )
    }

    /// Finish only the original accepted physical step and Native acknowledgement.
    /// A fresh request never restarts members or anchor publication.
    pub fn recover_checkpoint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
        super::journey::recover(&self.context.serving, &self.context.checkpoint, execution)
    }

    pub fn checkpoint_observation(&self) -> Option<super::ApplicationHomeCheckpointRecovery> {
        self.context
            .checkpoint
            .pending()
            .map(|attempt| attempt.observation())
    }
}

pub(crate) fn checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    owner: &super::owner::HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
    capture: impl FnOnce() -> Result<
        worth_store_relational::PreparedRelationalSection,
        worth_store_relational::RelationalSectionCheckpointDenial,
    >,
) -> Result<ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    super::journey::checkpoint(serving, owner, execution, capture, None, None)
}
