use crate::physical_runtime::BlobPhysicalAllocation;

use super::super::super::{scan, BlobReclaimFailure, BlobReclaimLimits};

pub(super) fn require_grant(
    limits: BlobReclaimLimits,
    allocation: &BlobPhysicalAllocation<'_>,
) -> Result<(), BlobReclaimFailure> {
    let required = limits
        .memory_bytes()
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if allocation.bytes() < required.get() {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    Ok(())
}

pub(super) fn scratch() -> Result<Vec<u8>, BlobReclaimFailure> {
    let mut scratch = Vec::new();
    scratch
        .try_reserve_exact(scan::FRAME_WINDOW_BYTES)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    scratch.resize(scan::FRAME_WINDOW_BYTES, 0);
    if scratch.capacity() != scan::FRAME_WINDOW_BYTES {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    Ok(scratch)
}
