//! Lower the admitted checkpoint source into its first funded command.

use worth_store_physical_format::CheckpointStreamEncoder;

use super::{PhysicalCheckpointActionFailure as Failure, PhysicalCheckpointCaptureBasis};
use crate::physical_runtime::durability::{
    FundedCheckpointCommandBufferLease, SelectedCheckpointCustodySnapshot,
};
use crate::physical_runtime::work::PhysicalCheckpointCommandPayload;

pub(super) struct PreparedCheckpointHeader {
    pub(super) encoder: CheckpointStreamEncoder,
    pub(super) payload: PhysicalCheckpointCommandPayload,
    pub(super) command_buffer: Option<FundedCheckpointCommandBufferLease>,
}

pub(super) fn prepare(
    basis: PhysicalCheckpointCaptureBasis,
    custody: Option<&SelectedCheckpointCustodySnapshot>,
) -> Result<PreparedCheckpointHeader, Failure> {
    if custody.is_none() {
        let (encoder, header) = CheckpointStreamEncoder::begin(basis.source());
        return Ok(PreparedCheckpointHeader {
            encoder,
            payload: header.into_boxed_slice().into(),
            command_buffer: None,
        });
    }
    let mut command_buffer = custody
        .and_then(SelectedCheckpointCustodySnapshot::take_command_buffer)
        .ok_or(Failure::CheckpointCommandBackingUnavailable)?;
    let buffer = command_buffer.buffer_mut();
    let bytes = buffer
        .bytes_mut()
        .ok_or(Failure::CheckpointCommandBackingUnavailable)?;
    let encoder = CheckpointStreamEncoder::begin_certified_in_reserved(basis.source(), bytes)
        .map_err(|_| Failure::CheckpointCommandBackingUnavailable)?;
    let payload = PhysicalCheckpointCommandPayload::Command(buffer.frame());
    Ok(PreparedCheckpointHeader {
        encoder,
        payload,
        command_buffer: Some(command_buffer),
    })
}
