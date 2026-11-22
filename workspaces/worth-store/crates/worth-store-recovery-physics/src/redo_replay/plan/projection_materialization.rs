use super::projection_admission::IntegrityAdmittedRecoveryProjection;
use super::*;

pub(super) fn validate_extent_closure(
    placement: worth_store_physical_format::DurableExtentRecordPlacement,
    admitted: &IntegrityAdmittedRecoveryProjection<'_>,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let mut matching_manifests = admitted.extent_manifests().iter().filter(|value| {
        value.manifest.record() == placement.record()
            && value.manifest.extent_cell() == placement.extent_cell()
    });
    let Some(first_manifest) = matching_manifests.next() else {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    };
    if matching_manifests.next().is_some() {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let manifest = first_manifest.manifest;
    if manifest.logical_bytes() != placement.payload_bytes() {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let chunks = admitted.extent_chunks_for(placement.record(), placement.extent_cell());
    if chunks.len() != manifest.chunk_count() as usize {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let mut logical_offset = 0_u64;
    for (ordinal, chunk) in (1_u32..).zip(chunks) {
        if chunk.coordinate.ordinal() != ordinal
            || chunk.coordinate.logical_offset() != logical_offset
            || chunk.coordinate.logical_bytes() != manifest.logical_bytes()
            || admitted.frame_bytes(chunk.frame_index).len()
                > manifest.maximum_frame_bytes() as usize
        {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
        logical_offset = logical_offset
            .checked_add(chunk.payload_range.len() as u64)
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
    }
    if logical_offset != manifest.logical_bytes() {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    Ok(())
}

pub(super) fn matches_projected_record_bytes(
    identity: worth_store_physical_format::PersistedRecordIdentity,
    placement: CurrentPhysicalRecordPlacement,
    admitted: &IntegrityAdmittedRecoveryProjection<'_>,
    expected: &[u8],
) -> Result<bool, PhysicalRedoPlanningDenial> {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(value) => {
            let mut records = admitted.inline_frames().iter().flat_map(|frame| {
                frame
                    .records
                    .iter()
                    .filter(move |(found, _)| *found == value)
                    .map(move |(_, range)| (frame.frame_index, range))
            });
            let Some((frame_index, range)) = records.next() else {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            };
            if records.next().is_some() || value.record() != identity {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            Ok(&admitted.frame_bytes(frame_index)[range.clone()] == expected)
        }
        CurrentPhysicalRecordPlacement::Extent(value) => {
            let chunks = admitted.extent_chunks_for(identity, value.extent_cell());
            let mut offset = 0_usize;
            for (expected_ordinal, chunk) in (1_u32..).zip(chunks) {
                if chunk.coordinate.ordinal() != expected_ordinal
                    || chunk.coordinate.logical_offset() != offset as u64
                    || chunk.coordinate.logical_bytes() != value.payload_bytes()
                {
                    return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                }
                let slice = &admitted.frame_bytes(chunk.frame_index)[chunk.payload_range.clone()];
                let end = offset
                    .checked_add(slice.len())
                    .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
                if end > expected.len() || &expected[offset..end] != slice {
                    return Ok(false);
                }
                offset = end;
            }
            if offset as u64 != value.payload_bytes() {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            Ok(offset == expected.len())
        }
    }
}
