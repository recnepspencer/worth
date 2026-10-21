//! Actual native managed-text decode entry: no context-free owned payload stage.
use super::*;
use crate::durability::data::RelationalRecoveryStop;
use worth_execution::{ExecutionByteBuffer, ExecutionImmutableBytes, ExecutionRequest};

pub(super) fn read_payload(
    execution: &ExecutionRequest<'_, '_>,
    path: &Path,
) -> Result<ExecutionImmutableBytes, RelationalRecoveryStop> {
    use std::io::Read;
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let mut input = fs::File::open(path).map_err(super::super::local_store::io_error)?;
    let length = usize::try_from(
        input
            .metadata()
            .map_err(super::super::local_store::io_error)?
            .len(),
    )
    .map_err(|_| {
        DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "native input length overflows",
        )
    })?;
    let mut bytes =
        ExecutionByteBuffer::allocate(length, execution.allocation_policy()).map_err(|e| {
            crate::symbols::data::field_text_recovery_stop(
                crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(e),
            )
        })?;
    let mut chunk = [0u8; 8192];
    loop {
        let count = input
            .read(&mut chunk)
            .map_err(super::super::local_store::io_error)?;
        if count == 0 {
            break;
        }
        execution
            .check_live()
            .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
        bytes.extend_from_slice(&chunk[..count]).map_err(|e| {
            crate::symbols::data::field_text_recovery_stop(
                crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(e),
            )
        })?;
    }
    bytes.seal().map_err(|e| {
        crate::symbols::data::field_text_recovery_stop(
            crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(e),
        )
    })
}
pub(crate) fn decode_checkpoint(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<DurableCheckpointFile, RelationalRecoveryStop> {
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
pub(crate) fn read_checkpoint_file(
    execution: &ExecutionRequest<'_, '_>,
    path: &Path,
) -> Result<DurableCheckpointFile, RelationalRecoveryStop> {
    use std::io::Read;
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let mut input = fs::File::open(path).map_err(super::super::local_store::io_error)?;
    let mut magic = [0; 15];
    input.read_exact(&mut magic).map_err(|_| {
        DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "truncated checkpoint frame",
        )
    })?;
    if magic.as_slice() != super::CHECKPOINT_MAGIC {
        return Err(DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            "unsupported native checkpoint frame; CHK6 required",
        )
        .into());
    }
    let bytes = read_payload(execution, path)?;
    decode_checkpoint(execution, bytes.bytes())
}
