//! Capture must preserve every selected leaf fact and name a disagreeing region.
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::errors::data::context::{ErrorOperation, SuggestedFix};
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use worth_relational_branch::internal_port::branch::root_checkpoint::RelationalRootRegionCheckpoint;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::data::checkpoint_region::RelationalCheckpointRegion;

pub(super) fn validate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &RelationalRootRegionCheckpoint,
    captured: &RelationalCheckpointRegion,
) -> Result<(), contracts::durability::data::recovery_stop::RelationalRecoveryStop> {
    let equal = worth_relational_storage::internal_port::storage::residency::checkpoint_capture::verify_checkpoint_directory(
        execution,
        &selected.directory,
        &captured.directory,
    )?;
    validate_value(selected, captured, equal).map_err(Into::into)
}
fn validate_value(
    selected: &RelationalRootRegionCheckpoint,
    captured: &RelationalCheckpointRegion,
    directory_equal: bool,
) -> Result<(), DurabilityError> {
    if captured.partition_id == selected.partition_id
        && captured.content_digest == selected.content_digest
        && directory_equal
        && captured.live_counts == selected.live_counts
        && captured.entity_slots == selected.entity_slots
        && captured.relation_slots == selected.relation_slots
        && captured.unavailable_records == selected.unavailable_records
        && captured.kind_authority == selected.kind_commitment
    {
        return Ok(());
    }
    let mut error = DurabilityError::new(
        RecoveryFailureClass::CorruptCheckpoint,
        "captured region differs from its selected root leaf",
    );
    error.storage_unavailable = Some(RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(selected.partition_id),
        cause: RelationalStorageUnavailableCause::IntegrityMismatch,
    });
    // This is a capture consistency failure, before any candidate bytes publish.
    // No particular record or existing durable store has been proved corrupt.
    error.context.operation = ErrorOperation::WriteDurableStore;
    error.context.suggested_fix = Some(SuggestedFix::InspectDiagnostics);
    Err(error)
}

#[cfg(test)]
pub mod tests;
