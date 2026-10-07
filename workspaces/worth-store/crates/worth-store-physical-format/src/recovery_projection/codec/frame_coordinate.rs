use super::*;

pub(super) fn write_subject_coordinate(
    target: &mut Vec<u8>,
    subject: PersistedPhysicalDataFrameSubject,
    coordinate: RecordFrameCoordinate,
) {
    match subject {
        PersistedPhysicalDataFrameSubject::InlinePage(page) => {
            target.push(1);
            target.extend_from_slice(&page.segment_id().get().to_le_bytes());
            target.extend_from_slice(&page.page_id().get().to_le_bytes());
            target.extend_from_slice(&page.generation().get().to_le_bytes());
            let RecordArtifactFile::Segment {
                segment,
                generation,
            } = coordinate.artifact()
            else {
                unreachable!("an admitted inline projection frame owns a segment coordinate")
            };
            debug_assert_eq!(segment, page.segment_id().get());
            target.extend_from_slice(&generation.to_le_bytes());
        }
        PersistedPhysicalDataFrameSubject::ExtentChunk(chunk) => {
            target.push(2);
            write_record(target, chunk.record());
            target.extend_from_slice(&chunk.extent_cell().extent_id().get().to_le_bytes());
            target.extend_from_slice(&chunk.extent_cell().generation().get().to_le_bytes());
            target.extend_from_slice(&chunk.logical_bytes().to_le_bytes());
            target.extend_from_slice(&chunk.logical_offset().to_le_bytes());
            target.extend_from_slice(&chunk.ordinal().to_le_bytes());
            let RecordArtifactFile::ExtentArena { arena } = coordinate.artifact() else {
                unreachable!("admitted extent frame has an arena coordinate")
            };
            target.extend_from_slice(&arena.to_le_bytes());
        }
    }
    target.extend_from_slice(&coordinate.offset().to_le_bytes());
    target.extend_from_slice(&coordinate.length().to_le_bytes());
}

pub(super) fn read_subject_coordinate(
    cursor: &mut Cursor<'_>,
) -> Result<
    (PersistedPhysicalDataFrameSubject, RecordFrameCoordinate),
    PhysicalRecoveryProjectionDenial,
> {
    let kind = cursor.byte()?;
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let (subject, artifact) = match kind {
        1 => {
            let segment = PhysicalSegmentId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidFrame)?;
            let page = PhysicalPageId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidFrame)?;
            let page_generation = generation(cursor.u64()?)?;
            let artifact_generation = generation(cursor.u64()?)?;
            (
                PersistedPhysicalDataFrameSubject::InlinePage(
                    authority
                        .page_cell(segment, page)
                        .with_page_generation(page_generation),
                ),
                RecordArtifactFile::Segment {
                    segment: segment.get(),
                    generation: artifact_generation.get(),
                },
            )
        }
        2 => {
            let record = read_record(cursor)?;
            let extent = PhysicalExtentId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidFrame)?;
            let generation = generation(cursor.u64()?)?;
            let logical_bytes = cursor.u64()?;
            let logical_offset = cursor.u64()?;
            let ordinal = cursor.u32()?;
            let arena = cursor.u64()?;
            let cell = authority
                .record_extent_cell(extent)
                .with_extent_generation(generation);
            let chunk =
                ExtentChunkCoordinate::new(record, cell, logical_bytes, logical_offset, ordinal)
                    .ok_or(PhysicalRecoveryProjectionDenial::InvalidFrame)?;
            (
                PersistedPhysicalDataFrameSubject::ExtentChunk(chunk),
                RecordArtifactFile::ExtentArena { arena },
            )
        }
        _ => return Err(PhysicalRecoveryProjectionDenial::InvalidFrame),
    };
    let coordinate = RecordFrameCoordinate::new(artifact, cursor.u64()?, cursor.u32()?)
        .ok_or(PhysicalRecoveryProjectionDenial::InvalidFrame)?;
    Ok((subject, coordinate))
}
