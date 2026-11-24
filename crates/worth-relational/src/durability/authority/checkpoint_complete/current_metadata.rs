//! One request-funded freeze of current owner state, independent of source class.
use super::{
    super::checkpoint_components::{capture_complete, CapturedCompleteComponents},
    Denial,
};
use crate::branch::registry::CheckpointBranchMetadata;
use crate::durability::data::DurableRecordIdentityState;
use crate::runtime::state::subsystems::{
    history::HistorySubsystem, record_identity::RecordIdentitySubsystem,
};
use worth_execution::ExecutionRequest;

pub(super) struct FrozenCurrentCheckpointMetadata {
    pub(super) branches: CheckpointBranchMetadata,
    pub(super) components: CapturedCompleteComponents,
    pub(super) identity: DurableRecordIdentityState,
    pub(super) main_cell: usize,
}
impl FrozenCurrentCheckpointMetadata {
    /// The caller holds configuration and a settled publication epoch.
    /// Every allocating snapshot enters its concrete owner's funded door.
    pub(super) fn capture(
        history: &HistorySubsystem,
        record_identity: &RecordIdentitySubsystem,
        request: &ExecutionRequest<'_, '_>,
        runtime_name: &str,
        main: &crate::history::data::BranchId,
    ) -> Result<Self, Denial> {
        let branches = history
            .capture_checkpoint_branch_metadata(request, runtime_name)
            .map_err(|cause| match cause {
                crate::branch::registry::CheckpointBranchMetadataCaptureDenial::Request(cause) => {
                    Denial::Request(cause)
                }
                crate::branch::registry::CheckpointBranchMetadataCaptureDenial::SizeOverflow => {
                    Denial::Request(
                        crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
                    )
                }
                crate::branch::registry::CheckpointBranchMetadataCaptureDenial::SourceChanged => {
                    Denial::OriginalAuthorityUnavailable
                }
            })?;
        let main_cell = branches
            .branch_cells
            .iter()
            .position(|cell| &cell.branch_id == main)
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let components = capture_complete(history, request, &branches)?;
        let identity = record_identity
            .checkpoint_identity_for_request(request)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        Ok(Self {
            branches,
            components,
            identity,
            main_cell,
        })
    }
}
