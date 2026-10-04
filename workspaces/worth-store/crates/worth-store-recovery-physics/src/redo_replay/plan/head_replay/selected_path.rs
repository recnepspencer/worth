//! Bounds and the selected-media rejoin shared by every release-head tree
//! claim. A keyed upsert and a terminal head retirement carry the same path
//! roster, so both replays are bounded and re-read through this one owner.

use worth_store_physical_format::{
    PersistedReleaseHeadTreeClaim, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1,
};

use super::SelectedReleaseHeadReplayDenial;

/// Deny before any read when the carried frames exceed the admitted effect
/// bytes, or when recomputing, reading, or retaining them exceeds the heap.
pub(super) fn require_claim_bounds(
    claim: PersistedReleaseHeadTreeClaim<'_>,
    format: PhysicalRecordFormatDeclaration,
    maximum_effect_bytes: u64,
    remaining_additional_heap_bytes: u64,
) -> Result<(), SelectedReleaseHeadReplayDenial> {
    let total = claim
        .framed_bytes()
        .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
    if total > maximum_effect_bytes {
        return Err(SelectedReleaseHeadReplayDenial::BoundExceeded);
    }
    let verification_peak = claim
        .verification_additional_peak_bytes(format)
        .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
    let clone_bytes = claim
        .owned_heap_bytes()
        .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
    let read_buffer = u64::from(format.page_size().bytes());
    if verification_peak.max(read_buffer).max(clone_bytes) > remaining_additional_heap_bytes {
        return Err(SelectedReleaseHeadReplayDenial::BoundExceeded);
    }
    Ok(())
}

/// Re-read every carried source path frame through `read` and require the
/// exact carried bytes, which `require_claim_bounds` has already bounded.
/// Returns the total bytes read from the selected tree.
pub(super) fn reread_source_path<Read, ReadError>(
    claim: PersistedReleaseHeadTreeClaim<'_>,
    format: PhysicalRecordFormatDeclaration,
    read: &mut Read,
) -> Result<u64, SelectedReleaseHeadReplayDenial>
where
    Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
{
    let mut read_bytes = 0_u64;
    for node in claim.source_path() {
        let frame = read(node.reference(), u64::from(format.page_size().bytes()))
            .map_err(|_| SelectedReleaseHeadReplayDenial::Read)?;
        read_bytes = read_bytes
            .checked_add(
                u64::try_from(frame.len())
                    .map_err(|_| SelectedReleaseHeadReplayDenial::BoundExceeded)?,
            )
            .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
        if frame != node.frame() {
            return Err(SelectedReleaseHeadReplayDenial::SourcePath);
        }
    }
    Ok(read_bytes)
}
