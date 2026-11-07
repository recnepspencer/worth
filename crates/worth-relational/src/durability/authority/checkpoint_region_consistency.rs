//! Capture must preserve every selected leaf fact and name a disagreeing region.
use crate::branch::RelationalRootRegionCheckpoint;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalCheckpointRegion};
use crate::errors::data::{ErrorOperation, SuggestedFix};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};

pub(super) fn validate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &RelationalRootRegionCheckpoint,
    captured: &RelationalCheckpointRegion,
) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
    let equal = crate::storage::residency::verify_checkpoint_directory(
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
mod tests;
