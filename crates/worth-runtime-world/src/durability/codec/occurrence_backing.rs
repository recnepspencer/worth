//! Fill the original pre-effect backing with actual post-effect evidence.
use sha2::{Digest, Sha256};
use worth_execution::ExecutionByteBuffer;

use super::{write_record, Denial, WorldDurableMediumDenial, WorldRecord, HEADER, MAGIC, MAXIMUM};

pub(in crate::durability) fn admitted_length(
    open: usize,
    intent: crate::durability::WorldOpenComponentIntent,
) -> Result<usize, Denial> {
    use crate::durability::WorldOpenComponentIntent;
    let owner_bytes = match intent {
        WorldOpenComponentIntent::RelationalOnly => 48,
        WorldOpenComponentIntent::SignalOnly => {
            8 + worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES
        }
        WorldOpenComponentIntent::RelationalAndSignal => {
            48 + 8
                + worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES
        }
    };
    open.checked_add(1 + owner_bytes + 8)
        .filter(|total| *total <= MAXIMUM)
        .ok_or(Denial::SizeLimit)
}

pub(in crate::durability) fn fill(
    backing: &mut ExecutionByteBuffer,
    record: &WorldRecord,
) -> Result<(), WorldDurableMediumDenial> {
    let mut digest = Sha256::new();
    let mut size = 0usize;
    write_record(record, &mut |bytes| {
        size = size
            .checked_add(bytes.len())
            .filter(|size| *size <= MAXIMUM - HEADER)
            .ok_or(Denial::SizeLimit)?;
        digest.update(bytes);
        Ok(())
    })
    .map_err(WorldDurableMediumDenial::Frame)?;
    let digest: [u8; 32] = digest.finalize().into();
    let mut cursor = 0;
    for bytes in [
        MAGIC.as_slice(),
        super::WORLD_ARTIFACT_VERSION.get().to_le_bytes().as_slice(),
        (size as u64).to_le_bytes().as_slice(),
        digest.as_slice(),
    ] {
        write_extent(backing, &mut cursor, bytes).map_err(WorldDurableMediumDenial::Allocation)?;
    }
    let mut stopped = None;
    write_record(record, &mut |bytes| {
        write_extent(backing, &mut cursor, bytes).map_err(|cause| {
            stopped = Some(cause);
            Denial::Malformed
        })
    })
    .map_err(|cause| {
        stopped.map_or(
            WorldDurableMediumDenial::Frame(cause),
            WorldDurableMediumDenial::Allocation,
        )
    })?;
    Ok(())
}

fn write_extent(
    backing: &mut ExecutionByteBuffer,
    cursor: &mut usize,
    bytes: &[u8],
) -> Result<(), worth_execution::ExecutionAllocationDenial> {
    let written = backing.len().saturating_sub(*cursor).min(bytes.len());
    if written != 0 {
        backing.overwrite(*cursor, &bytes[..written])?;
    }
    backing.extend_from_slice(&bytes[written..])?;
    *cursor += bytes.len();
    Ok(())
}
