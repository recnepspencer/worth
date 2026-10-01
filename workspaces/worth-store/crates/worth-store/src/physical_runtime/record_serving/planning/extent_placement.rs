use std::collections::BTreeMap;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentManifest, DurableExtentRecordPlacement,
    ExtentArenaFrameLayout, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
    DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};

use super::super::{
    arena::{ArenaReservation, ReleasedControlArenaPlacement, SharedArenaAllocationOwner},
    planning::batch_placement::ExtentInput,
    publication::extent_publication::ExtentDataPlan,
    publication::CandidateDataArtifact,
    AdmittedPhysicalRecordFormat, RecordAllocationFrontier, RecordAppendDenial, RecordAppendError,
};

pub(in crate::physical_runtime::record_serving) fn lower_extents(
    format: AdmittedPhysicalRecordFormat,
    frontier: &mut RecordAllocationFrontier,
    arena_owner: &SharedArenaAllocationOwner,
    extents: Vec<ExtentInput>,
    released_control_placement: Option<&ReleasedControlArenaPlacement>,
    data: &mut Vec<CandidateDataArtifact>,
    manifests: &mut Vec<(RecordFrameCoordinate, Vec<u8>)>,
    placements: &mut BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<Vec<ArenaReservation>, RecordAppendError> {
    if released_control_placement.is_some() && extents.len() != 1 {
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::ReclaimFenceUnavailable,
        ));
    }
    let mut reservations = Vec::new();
    for extent in extents {
        if let Some(reservation) = lower_extent(
            format,
            frontier,
            arena_owner,
            extent,
            released_control_placement,
            data,
            manifests,
            placements,
        )? {
            reservations.push(reservation);
        }
    }
    Ok(reservations)
}

fn lower_extent(
    format: AdmittedPhysicalRecordFormat,
    frontier: &mut RecordAllocationFrontier,
    arena_owner: &SharedArenaAllocationOwner,
    extent: ExtentInput,
    released_control_placement: Option<&ReleasedControlArenaPlacement>,
    data: &mut Vec<CandidateDataArtifact>,
    manifests: &mut Vec<(RecordFrameCoordinate, Vec<u8>)>,
    placements: &mut BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<Option<ArenaReservation>, RecordAppendError> {
    let extent_id = frontier.allocate_extent().ok_or(RecordAppendError::Denied(
        RecordAppendDenial::PhysicalIdentityExhausted,
    ))?;
    let maximum_frame_bytes = format.declaration().page_size().bytes();
    let chunk_payload_capacity = maximum_frame_bytes
        .checked_sub((DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u32)
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::RecordTooLarge,
        ))?;
    let chunk_count = u32::try_from(extent.length.div_ceil(u64::from(chunk_payload_capacity)))
        .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::RecordTooLarge))?;
    let extent_generation = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(extent_id)
        .with_extent_generation(
            PhysicalGeneration::from_raw(1).expect("initial extent generation is nonzero"),
        );
    let alignment = arena_owner
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .alignment();
    let layout = ExtentArenaFrameLayout::new(format.declaration(), alignment).ok_or(
        RecordAppendError::Denied(RecordAppendDenial::RecordTooLarge),
    )?;
    let bytes = layout
        .allocated_bytes(chunk_count)
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::RecordTooLarge,
        ))?;
    let (reservation, range) = if let Some(prepared) = released_control_placement {
        let claim = prepared.reservation();
        if !claim.belongs_to(arena_owner)
            || !claim.is_live()
            || prepared.encoded_bytes() != extent.length
            || claim.range().length() != bytes
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::ReclaimFenceUnavailable,
            ));
        }
        (None, claim.range())
    } else {
        let claim = ArenaReservation::reserve(arena_owner, bytes).map_err(|cause| {
            RecordAppendError::Denied(RecordAppendDenial::ArenaAllocationUnavailable(cause))
        })?;
        let range = claim.range();
        (Some(claim), range)
    };
    let manifest = DurableExtentManifest::new(
        format.declaration(),
        extent.record,
        extent_generation,
        extent.length,
        maximum_frame_bytes,
        chunk_count,
        alignment,
    )
    .ok_or(RecordAppendError::Denied(
        RecordAppendDenial::RecordTooLarge,
    ))?;
    let artifact = RecordArtifactFile::ExtentArena {
        arena: range.arena().get(),
    };
    let manifest_bytes = manifest.encode(format.declaration());
    manifests.push((
        RecordFrameCoordinate::new(artifact, range.offset(), manifest_bytes.len() as u32).ok_or(
            RecordAppendError::Denied(RecordAppendDenial::RecordTooLarge),
        )?,
        manifest_bytes,
    ));
    data.push(CandidateDataArtifact::Extent(ExtentDataPlan {
        artifact,
        range,
        manifest,
        source: extent.source,
    }));
    placements.insert(
        extent.record,
        CurrentPhysicalRecordPlacement::Extent(
            DurableExtentRecordPlacement::new_selected(
                extent.record,
                extent_generation,
                extent.length,
                range,
                extent.route_metadata,
            )
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::RecordTooLarge,
            ))?,
        ),
    );
    Ok(reservation)
}
