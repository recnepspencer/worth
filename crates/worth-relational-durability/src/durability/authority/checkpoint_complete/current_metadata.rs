//! One request-funded freeze of current owner state, independent of source class.
use super::super::checkpoint_components::capture_complete;
use super::super::checkpoint_components::CapturedCompleteComponents;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::FrozenRecordIdentity;

pub struct FrozenCurrentCheckpointMetadata {
    pub branches: CheckpointBranchMetadata,
    pub components: CapturedCompleteComponents,
    pub identity: FrozenRecordIdentity,
    pub main_cell: usize,
}
impl FrozenCurrentCheckpointMetadata {
    /// The caller holds configuration and a settled publication epoch.
    /// Every allocating snapshot enters its concrete owner's funded door.
    pub fn capture(
        history: &HistorySubsystem,
        record_identity: &RecordIdentitySubsystem,
        request: &ExecutionRequest<'_, '_>,
        runtime_name: &str,
        main: &contracts::history::data::coordinates::BranchId,
    ) -> Result<Self, Denial> {
        let branches = history
            .capture_checkpoint_branch_metadata(request, runtime_name)
            .map_err(|cause| match cause {
                worth_relational_branch::internal_port::branch::registry::checkpoint_capture::CheckpointBranchMetadataCaptureDenial::Request(cause) => {
                    Denial::Request(cause)
                }
                worth_relational_branch::internal_port::branch::registry::checkpoint_capture::CheckpointBranchMetadataCaptureDenial::SizeOverflow => {
                    Denial::Request(
                        contracts::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
                    )
                }
                worth_relational_branch::internal_port::branch::registry::checkpoint_capture::CheckpointBranchMetadataCaptureDenial::SourceChanged => {
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
            .frozen_checkpoint_identity_for_request(request)
            .map_err(contracts::durability::data::native_checkpoint_capture_denial::capture_stop)?;
        Ok(Self {
            branches,
            components,
            identity,
            main_cell,
        })
    }
}
