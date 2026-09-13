use super::projection_admission::IntegrityAdmittedRecoveryProjection;
use super::*;

pub(super) fn validate_extent_closure(
    placement: worth_store_physical_format::DurableExtentRecordPlacement,
    admitted: &IntegrityAdmittedRecoveryProjection<'_>,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let manifests = admitted
        .extent_manifests()
        .iter()
        .filter(|value| {
            value.manifest.record() == placement.record()
                && value.manifest.extent_cell() == placement.extent_cell()
        })
        .collect::<Vec<_>>();
    if manifests.len() != 1 {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let manifest = manifests[0].manifest;
    if manifest.logical_bytes() != placement.payload_bytes() {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let mut chunks = admitted
        .extent_chunks()
        .iter()
        .filter(|chunk| {
            chunk.coordinate.record() == placement.record()
                && chunk.coordinate.extent_cell() == placement.extent_cell()
        })
        .collect::<Vec<_>>();
    chunks.sort_by_key(|chunk| chunk.coordinate.ordinal());
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

pub(super) fn projected_record_bytes(
    identity: worth_store_physical_format::PersistedRecordIdentity,
    placement: CurrentPhysicalRecordPlacement,
    admitted: &IntegrityAdmittedRecoveryProjection<'_>,
) -> Result<Vec<u8>, PhysicalRedoPlanningDenial> {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(value) => {
            let records = admitted
                .inline_frames()
                .iter()
                .flat_map(|frame| {
                    frame
                        .records
                        .iter()
                        .filter(move |(found, _)| *found == value)
                        .map(move |(_, range)| (frame.frame_index, range))
                })
                .collect::<Vec<_>>();
            if records.len() != 1 || value.record() != identity {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            Ok(admitted.frame_bytes(records[0].0)[records[0].1.clone()].to_vec())
        }
        CurrentPhysicalRecordPlacement::Extent(value) => {
            let mut chunks = admitted
                .extent_chunks()
                .iter()
                .filter(|chunk| {
                    chunk.coordinate.record() == identity
                        && chunk.coordinate.extent_cell() == value.extent_cell()
                })
                .collect::<Vec<_>>();
            chunks.sort_by_key(|chunk| chunk.coordinate.ordinal());
            let mut payload = Vec::new();
            for (expected_ordinal, chunk) in (1_u32..).zip(chunks) {
                if chunk.coordinate.ordinal() != expected_ordinal
                    || chunk.coordinate.logical_offset() != payload.len() as u64
                    || chunk.coordinate.logical_bytes() != value.payload_bytes()
                {
                    return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                }
                payload.extend_from_slice(
                    &admitted.frame_bytes(chunk.frame_index)[chunk.payload_range.clone()],
                );
            }
            if payload.len() as u64 != value.payload_bytes() {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            Ok(payload)
        }
    }
}
