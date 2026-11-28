use crate::durability::artifact_payload_emission::request::{extend_length, named_to};
use worth_relational_contracts::internal_port::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_relational_contracts::internal_port::durability::data::recovery_errors::{
    DurabilityError, RecoveryFailureClass,
};
pub mod control;
pub(crate) use control::CheckpointEncodingControl;
use worth_execution::ExecutionImmutableBytes;

pub(super) fn encoding_failure(error: impl std::fmt::Display) -> Denial {
    DurabilityError::new(
        RecoveryFailureClass::DurableIoFailure,
        format!("failed to encode native checkpoint: {error}"),
    )
    .into()
}

pub(in super::super) fn frame_mismatch(detail: &str) -> Denial {
    DurabilityError::new(RecoveryFailureClass::CheckpointFrameSizeMismatch, detail).into()
}

pub(crate) fn count_value(
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
    value: &impl serde::Serialize,
) -> Result<usize, Denial> {
    let mut total = 0;
    named_to(
        value,
        |input| {
            for chunk in input.chunks(4096) {
                control.checkpoint(chunk.len() as u64)?;
            }
            total = extend_length(total, input).ok_or_else(|| {
                DurabilityError::new(
                    RecoveryFailureClass::CheckpointSizeOverflow,
                    "native checkpoint encoded byte length is not representable",
                )
            })?;
            Ok(())
        },
        encoding_failure,
    )?;
    control.checkpoint(0)?;
    Ok(total)
}

pub(crate) fn encode_value(
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
    prefix: &[u8],
    value: &impl serde::Serialize,
) -> Result<ExecutionImmutableBytes, Denial> {
    let total = count_value(control, value)?
        .checked_add(prefix.len())
        .ok_or_else(|| frame_mismatch("native checkpoint frame size overflow"))?;
    let mut bytes = control.allocate(total)?;
    let mut emit = |input: &[u8]| {
        for chunk in input.chunks(4096) {
            control.checkpoint(chunk.len() as u64)?;
            bytes.extend_from_slice(chunk).map_err(Denial::Allocation)?;
        }
        Ok(())
    };
    emit(prefix)?;
    named_to(value, emit, encoding_failure)?;
    control.checkpoint(0)?;
    if bytes.len() != total {
        return Err(frame_mismatch(
            "native checkpoint count and emission lengths differ",
        ));
    }
    bytes.seal().map_err(Denial::Allocation)
}
