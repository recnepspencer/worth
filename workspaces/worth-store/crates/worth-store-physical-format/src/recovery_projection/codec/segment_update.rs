use super::*;

pub(super) fn write_segment_update(target: &mut Vec<u8>, update: &RecordSegmentPageManifestEntry) {
    target.extend_from_slice(&update.page_cell().segment_id().get().to_le_bytes());
    target.extend_from_slice(&update.page().get().to_le_bytes());
    target.extend_from_slice(&update.page_generation().to_le_bytes());
    target.extend_from_slice(&update.data_generation().to_le_bytes());
    target.extend_from_slice(&update.data_page_count().to_le_bytes());
    target.extend_from_slice(&update.frame_index().to_le_bytes());
}

pub(super) fn read_segment_update(
    bytes: &[u8],
) -> Result<RecordSegmentPageManifestEntry, PhysicalRecoveryProjectionDenial> {
    let mut cursor = Cursor::new(bytes);
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(cursor.u64()?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidSegmentUpdate)?;
    let page = PhysicalPageId::from_raw(cursor.u64()?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidSegmentUpdate)?;
    let page_generation = generation(cursor.u64()?)?;
    let data_generation = generation(cursor.u64()?)?;
    let count = cursor.u32()?;
    let index = cursor.u32()?;
    cursor.end()?;
    RecordSegmentPageManifestEntry::new(
        authority
            .page_cell(segment, page)
            .with_page_generation(page_generation),
        authority
            .segment_cell(segment)
            .with_segment_generation(data_generation),
        count,
        index,
    )
    .ok_or(PhysicalRecoveryProjectionDenial::InvalidSegmentUpdate)
}
