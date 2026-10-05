//! Bounds and the selected-media rejoin shared by every release-head tree
//! claim. A keyed upsert and a terminal head retirement carry the same path
//! roster, so both replays are bounded and re-read through this one owner.

use worth_store_physical_format::{
    PersistedReleaseHeadTreeClaim, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1,
};

use super::{ExceededHeadReplayBound, HeadReplayBound, SelectedReleaseHeadReplayDenial};

/// Deny before any read when the carried frames exceed the admitted effect
/// bytes, or when recomputing, reading, or retaining them exceeds the heap.
pub(super) fn require_claim_bounds(
    claim: PersistedReleaseHeadTreeClaim<'_>,
    format: PhysicalRecordFormatDeclaration,
    maximum_effect_bytes: u64,
    remaining_additional_heap_bytes: u64,
) -> Result<(), SelectedReleaseHeadReplayDenial> {
    ExceededHeadReplayBound::within(
        HeadReplayBound::EffectBytes,
        claim.framed_bytes(),
        maximum_effect_bytes,
    )?;
    let read_buffer = u64::from(format.page_size().bytes());
    let held = claim
        .verification_additional_peak_bytes(format)
        .zip(claim.owned_heap_bytes())
        .map(|(verification, clone)| verification.max(read_buffer).max(clone));
    ExceededHeadReplayBound::within(
        HeadReplayBound::HeapBytes,
        held,
        remaining_additional_heap_bytes,
    )?;
    Ok(())
}

/// Re-read every carried source path frame through `read` and require the
/// exact carried bytes, which `require_claim_bounds` has already bounded.
/// Returns the total bytes read from the selected tree: the carried frames'
/// own, so their count is one `require_claim_bounds` has already taken.
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
        if frame != node.frame() {
            return Err(SelectedReleaseHeadReplayDenial::SourcePath);
        }
        read_bytes = read_bytes.saturating_add(frame.len() as u64);
    }
    Ok(read_bytes)
}
