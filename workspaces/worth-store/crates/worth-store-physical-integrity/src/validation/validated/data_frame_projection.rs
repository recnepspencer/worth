use worth_store_physical_format::PhysicalPageLsn;

use super::super::UntrustedPhysicalArtifact;

const DATA_FRAME_PAGE_LSN_OFFSET: usize = 36;
const DATA_FRAME_PAGE_LSN_BYTES: usize = 8;

/// Reads a field only after the family validator has admitted the complete
/// canonical data frame. This is a projection, not another frame decode or
/// checksum pass.
pub(super) fn page_lsn(inspected: UntrustedPhysicalArtifact<'_>) -> Option<PhysicalPageLsn> {
    let end = DATA_FRAME_PAGE_LSN_OFFSET.checked_add(DATA_FRAME_PAGE_LSN_BYTES)?;
    let encoded = inspected.bytes().get(DATA_FRAME_PAGE_LSN_OFFSET..end)?;
    Some(PhysicalPageLsn::new(u64::from_le_bytes(
        encoded.try_into().ok()?,
    )))
}
