//! Capture must preserve every selected leaf fact and name a disagreeing region.
use crate::branch::RelationalRootRegionCheckpoint;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalCheckpointRegion};
use crate::errors::data::{ErrorOperation, SuggestedFix};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};

pub(super) fn validate(
    selected: &RelationalRootRegionCheckpoint,
    captured: &RelationalCheckpointRegion,
) -> Result<(), DurabilityError> {
    if captured.partition_id == selected.partition_id
        && captured.content_digest == selected.content_digest
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
