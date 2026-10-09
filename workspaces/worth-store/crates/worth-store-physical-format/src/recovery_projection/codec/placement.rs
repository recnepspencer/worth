use super::*;

pub(super) fn write_placement(target: &mut Vec<u8>, placement: &CurrentPhysicalRecordPlacement) {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(value) => {
            target.push(1);
            write_record(target, value.record());
            target.extend_from_slice(&value.segment().get().to_le_bytes());
            target.extend_from_slice(&value.segment_generation().to_le_bytes());
            target.extend_from_slice(&value.page().get().to_le_bytes());
            target.extend_from_slice(&value.page_generation().to_le_bytes());
            target.extend_from_slice(&value.slot().get().to_le_bytes());
            target.extend_from_slice(&value.slot_generation().to_le_bytes());
            target.extend_from_slice(&value.segment_page_capacity().to_le_bytes());
            target.extend_from_slice(&value.payload_bytes().to_le_bytes());
        }
        CurrentPhysicalRecordPlacement::Extent(value) => {
            target.push(2);
            write_record(target, value.record());
            target.extend_from_slice(&value.extent().get().to_le_bytes());
            target.extend_from_slice(&value.extent_generation().to_le_bytes());
            target.extend_from_slice(&value.payload_bytes().to_le_bytes());
            target.extend_from_slice(&value.arena_range().arena().get().to_le_bytes());
            target.extend_from_slice(&value.arena_range().offset().to_le_bytes());
            target.extend_from_slice(&value.arena_range().length().to_le_bytes());
        }
    }
    target.extend_from_slice(&placement.route_metadata().encode());
}

pub(super) fn read_placement(
    bytes: &[u8],
) -> Result<CurrentPhysicalRecordPlacement, PhysicalRecoveryProjectionDenial> {
    let mut cursor = Cursor::new(bytes);
    let kind = cursor.byte()?;
    let record = read_record(&mut cursor)?;
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let placement = match kind {
        1 => {
            let segment = PhysicalSegmentId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let segment_generation = generation(cursor.u64()?)?;
            let page = PhysicalPageId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let page_generation = generation(cursor.u64()?)?;
            let slot = PhysicalRecordSlot::from_raw(cursor.u16()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let slot_generation = generation(cursor.u64()?)?;
            let capacity = cursor.u32()?;
            let payload = cursor.u64()?;
            CurrentPhysicalRecordPlacement::Inline(
                DurableInlineRecordPlacement::legacy_unknown(
                    record,
                    authority
                        .segment_cell(segment)
                        .with_segment_generation(segment_generation),
                    authority
                        .page_cell(segment, page)
                        .with_page_generation(page_generation),
                    authority
                        .slot_cell(segment, page, slot)
                        .with_slot_generation(slot_generation),
                    capacity,
                    payload,
                )
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?,
            )
        }
        2 => {
            let extent = PhysicalExtentId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let extent_generation = generation(cursor.u64()?)?;
            let payload = cursor.u64()?;
            let arena = ExtentArenaId::new(cursor.u64()?)
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let range = ExtentArenaRange::new(arena, cursor.u64()?, cursor.u64()?)
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            CurrentPhysicalRecordPlacement::Extent(
                DurableExtentRecordPlacement::legacy_unknown(
                    record,
                    authority
                        .record_extent_cell(extent)
                        .with_extent_generation(extent_generation),
                    payload,
                    range,
                )
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?,
            )
        }
        _ => return Err(PhysicalRecoveryProjectionDenial::InvalidPlacement),
    };
    let route_metadata =
        crate::SelectedRecordRouteMetadata::decode(cursor.take(7)?.try_into().unwrap())
            .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
    cursor.end()?;
    Ok(placement.with_route_metadata(route_metadata))
}
