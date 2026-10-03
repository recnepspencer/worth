//! Reuse the admitted writer frame for each binding and footer command.

use worth_store_physical_format::{
    CheckpointBindingCompactionEncoder, CheckpointBindingCompactionHeader, CheckpointStreamEncoder,
    CheckpointStreamFooter,
};

use super::PhysicalCheckpointActionFailure as Failure;
use crate::physical_runtime::durability::FundedCheckpointCommandBufferLease;
use crate::physical_runtime::work::PhysicalCheckpointCommandPayload;

pub(super) fn begin_bindings(
    encoder: CheckpointStreamEncoder,
    header: CheckpointBindingCompactionHeader,
    lease: Option<&mut FundedCheckpointCommandBufferLease>,
) -> Result<
    (
        CheckpointBindingCompactionEncoder,
        PhysicalCheckpointCommandPayload,
    ),
    Failure,
> {
    if let Some(lease) = lease {
        let buffer = lease.buffer_mut();
        let bytes = buffer
            .bytes_mut()
            .ok_or(Failure::CheckpointCommandBackingUnavailable)?;
        let encoder = encoder
            .begin_binding_compaction_in_reserved(header, bytes)
            .map_err(|_| Failure::CheckpointCommandBackingUnavailable)?;
        return Ok((
            encoder,
            PhysicalCheckpointCommandPayload::Command(buffer.frame()),
        ));
    }
    let (encoder, bytes) = encoder.begin_binding_compaction(header);
    Ok((encoder, bytes.into_boxed_slice().into()))
}

pub(super) fn binding(
    encoder: &mut CheckpointBindingCompactionEncoder,
    binding: &[u8],
    lease: Option<&mut FundedCheckpointCommandBufferLease>,
) -> Result<PhysicalCheckpointCommandPayload, Failure> {
    if let Some(lease) = lease {
        let buffer = lease.buffer_mut();
        let bytes = buffer
            .bytes_mut()
            .ok_or(Failure::CheckpointCommandBackingUnavailable)?;
        encoder
            .encode_binding_record_in_reserved(binding, bytes)
            .map_err(|_| Failure::CheckpointCommandBackingUnavailable)?;
        return Ok(PhysicalCheckpointCommandPayload::Command(buffer.frame()));
    }
    let bytes = encoder
        .encode_binding_record(binding)
        .map_err(|_| Failure::PreEffect)?;
    Ok(bytes.into_boxed_slice().into())
}

pub(super) fn footer(
    encoder: CheckpointBindingCompactionEncoder,
    lease: Option<&mut FundedCheckpointCommandBufferLease>,
) -> Result<(CheckpointStreamFooter, PhysicalCheckpointCommandPayload), Failure> {
    if let Some(lease) = lease {
        let buffer = lease.buffer_mut();
        let bytes = buffer
            .bytes_mut()
            .ok_or(Failure::CheckpointCommandBackingUnavailable)?;
        let footer = encoder
            .finish_in_reserved(bytes)
            .map_err(|_| Failure::CheckpointCommandBackingUnavailable)?;
        return Ok((
            footer,
            PhysicalCheckpointCommandPayload::Command(buffer.frame()),
        ));
    }
    let (footer, bytes) = encoder.finish();
    Ok((footer, bytes.into_boxed_slice().into()))
}
