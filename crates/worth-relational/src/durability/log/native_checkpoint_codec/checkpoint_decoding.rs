//! The native checkpoint decode entry: frame check, then execution-charged
//! readmission of the borrowed wire.
use super::{DecodedNativeCheckpoint, DurabilityError, RecoveryFailureClass};
use crate::durability::data::RelationalRecoveryStop;
use worth_execution::ExecutionRequest;

pub(crate) fn decode_checkpoint(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let payload = bytes.strip_prefix(super::CHECKPOINT_MAGIC).ok_or_else(|| {
        DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            "unsupported native checkpoint frame; CHK6 required",
        )
    })?;
    let wire: super::super::persisted_checkpoint::CheckpointFileWire<'_> =
        rmp_serde::from_slice(payload).map_err(|e| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                format!("native checkpoint format: {e}"),
            )
        })?;
    wire.readmit(execution)
}
