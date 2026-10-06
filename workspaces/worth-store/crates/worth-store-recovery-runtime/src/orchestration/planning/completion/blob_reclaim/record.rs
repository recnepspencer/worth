//! Bounded C.5 extent read for a selected reclaim control record.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_extent_chunk, CurrentPhysicalRecordPlacement, DurableExtentManifest,
    ExtentArenaFrameLayout, ExtentChunkCoordinate, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_physical_integrity::{
    IntegrityValidatedSelectedExtentPayload, PhysicalArtifactScope, PhysicalByteRange,
    SelectedExtentPayloadBuilder,
};

use super::super::super::manifest_entry_budget::ManifestEntryBudget;
use super::super::super::selected_source_inventory::{ResidentAllowance, ResidentTraceDenial};
use super::super::historical_publication::{self, HistoricalFailure};
use crate::entry::PhysicalRecoverySelectedRecordReadDenial;

impl From<PhysicalRecoverySelectedRecordReadDenial> for HistoricalFailure {
    fn from(denial: PhysicalRecoverySelectedRecordReadDenial) -> Self {
        match denial {
            PhysicalRecoverySelectedRecordReadDenial::ManifestRead(failure)
            | PhysicalRecoverySelectedRecordReadDenial::ChunkRead { failure, .. } => {
                historical_publication::discovery_failure(failure)
            }
            PhysicalRecoverySelectedRecordReadDenial::ManifestEntryLimit => Self::ManifestEntries,
            // A resident allowance that refused names its own limit.
            PhysicalRecoverySelectedRecordReadDenial::ResidentBoundExceeded
            | PhysicalRecoverySelectedRecordReadDenial::InvalidRoute
            | PhysicalRecoverySelectedRecordReadDenial::ManifestIntegrity(_)
            | PhysicalRecoverySelectedRecordReadDenial::ChunkIntegrity { .. }
            | PhysicalRecoverySelectedRecordReadDenial::Allocation { .. }
            | PhysicalRecoverySelectedRecordReadDenial::InvalidPayload => Self::Invalid,
        }
    }
}

impl From<ResidentTraceDenial> for PhysicalRecoverySelectedRecordReadDenial {
    fn from(denial: ResidentTraceDenial) -> Self {
        match denial {
            ResidentTraceDenial::ResidentBoundExceeded => Self::ResidentBoundExceeded,
            ResidentTraceDenial::Allocation { requested, cause } => {
                Self::Allocation { requested, cause }
            }
        }
    }
}

pub(super) fn selected_record(
    context: super::super::super::context::PlanningContext,
    basis: &mut super::super::super::resolved_basis::ResolvedPlanningBasis,
    generation: u64,
    identity: PersistedRecordIdentity,
    maximum: u64,
) -> Result<
    (super::super::super::context::PlanningContext, Vec<u8>),
    crate::entry::PhysicalRecoveryOutcome,
> {
    let format = context.authority.record_format;
    historical_publication::observe(
        context,
        basis,
        generation,
        identity,
        |discovery, _, route, budget, trace, scratch| {
            read(
                discovery, format, route, identity, maximum, budget, trace, scratch,
            )
        },
    )
}

pub(in crate::orchestration::planning) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    route: Option<CurrentPhysicalRecordPlacement>,
    record: PersistedRecordIdentity,
    maximum_payload_bytes: u64,
    budget: &mut ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
) -> Result<Vec<u8>, HistoricalFailure> {
    read_impl(
        discovery,
        format,
        route,
        record,
        maximum_payload_bytes,
        budget,
        trace,
        scratch,
        false,
        None,
    )
    .map(|(bytes, _)| bytes)
    .map_err(HistoricalFailure::from)
}

pub(in crate::orchestration::planning) fn read_with_witness(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    route: Option<CurrentPhysicalRecordPlacement>,
    record: PersistedRecordIdentity,
    maximum_payload_bytes: u64,
    budget: &mut ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    resident: &mut ResidentAllowance,
) -> Result<(Vec<u8>, IntegrityValidatedSelectedExtentPayload), HistoricalFailure> {
    let (bytes, witness) = read_impl(
        discovery,
        format,
        route,
        record,
        maximum_payload_bytes,
        budget,
        trace,
        scratch,
        true,
        Some(resident),
    )
    .map_err(HistoricalFailure::from)?;
    Ok((bytes, witness.ok_or(HistoricalFailure::Invalid)?))
}

#[allow(clippy::too_many_arguments)]
pub(in crate::orchestration::planning) fn read_with_witness_diagnostic(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    route: Option<CurrentPhysicalRecordPlacement>,
    record: PersistedRecordIdentity,
    maximum_payload_bytes: u64,
    budget: &mut ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    resident: &mut ResidentAllowance,
) -> Result<
    (Vec<u8>, IntegrityValidatedSelectedExtentPayload),
    PhysicalRecoverySelectedRecordReadDenial,
> {
    let (bytes, witness) = read_impl(
        discovery,
        format,
        route,
        record,
        maximum_payload_bytes,
        budget,
        trace,
        scratch,
        true,
        Some(resident),
    )?;
    Ok((
        bytes,
        witness.ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?,
    ))
}

fn read_impl(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    route: Option<CurrentPhysicalRecordPlacement>,
    record: PersistedRecordIdentity,
    maximum_payload_bytes: u64,
    budget: &mut ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    with_witness: bool,
    mut resident: Option<&mut ResidentAllowance>,
) -> Result<
    (Vec<u8>, Option<IntegrityValidatedSelectedExtentPayload>),
    PhysicalRecoverySelectedRecordReadDenial,
> {
    let Some(CurrentPhysicalRecordPlacement::Extent(placement)) = route else {
        return Err(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload);
    };
    if placement.record() != record
        || placement.payload_bytes() == 0
        || placement.payload_bytes() > maximum_payload_bytes
    {
        return Err(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload);
    }
    let page_limit = u64::from(format.page_size().bytes());
    // Reading one record charges one entry, however many chunks hold it.
    budget
        .consume(1)
        .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::ManifestEntryLimit)?;
    if let Some(ledger) = resident.as_deref_mut() {
        ledger
            .trace_slots(trace, 1)
            .map_err(PhysicalRecoverySelectedRecordReadDenial::from)?;
        ledger
            .transient(page_limit)
            .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::ResidentBoundExceeded)?;
    }
    let observed = discovery
        .read_extent_manifest(placement.arena_range(), page_limit)
        .map_err(PhysicalRecoverySelectedRecordReadDenial::ManifestRead)?;
    let bytes = observed
        .bytes()
        .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
    let range = PhysicalByteRange::new(placement.arena_range().offset(), bytes.len() as u64)
        .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
    let scope = PhysicalArtifactScope::extent_manifest(
        discovery.store_identity(),
        format,
        placement,
        range,
    );
    let admitted =
        crate::integrity_ingress::admit_extent_manifest_projection(&observed, scope, trace)
            .map_err(PhysicalRecoverySelectedRecordReadDenial::ManifestIntegrity)?;
    let projection = admitted.projection;
    let manifest = DurableExtentManifest::new(
        projection.record_format,
        projection.record,
        projection.extent_cell,
        projection.logical_bytes,
        projection.maximum_frame_bytes,
        projection.chunk_count,
        projection.alignment,
    )
    .filter(|manifest| {
        manifest.record() == record
            && manifest.extent_cell() == placement.extent_cell()
            && manifest.logical_bytes() == placement.payload_bytes()
    })
    .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment())
        .filter(|layout| layout.admits(placement.arena_range(), manifest.chunk_count()))
        .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
    let mut witness = if with_witness {
        Some(
            SelectedExtentPayloadBuilder::new(admitted.membership, placement)
                .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?,
        )
    } else {
        None
    };
    drop(observed);
    let payload_bytes = usize::try_from(placement.payload_bytes())
        .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
    let mut payload = Vec::new();
    payload.try_reserve_exact(payload_bytes).map_err(|cause| {
        PhysicalRecoverySelectedRecordReadDenial::Allocation {
            requested: placement.payload_bytes(),
            cause,
        }
    })?;
    if let Some(ledger) = resident.as_deref_mut() {
        let extra_capacity = payload
            .capacity()
            .checked_sub(payload_bytes)
            .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        ledger
            .bytes(extra_capacity as u64)
            .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::ResidentBoundExceeded)?;
    }
    for ordinal in 1..=manifest.chunk_count() {
        let coordinate = ExtentChunkCoordinate::new(
            record,
            manifest.extent_cell(),
            manifest.logical_bytes(),
            payload.len() as u64,
            ordinal,
        )
        .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        let length = (manifest.logical_bytes() as usize - payload.len())
            .min(manifest.chunk_payload_capacity() as usize);
        let frame_length = DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES + length;
        let relative = layout
            .chunk_offset(ordinal)
            .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        if let Some(ledger) = resident.as_deref_mut() {
            ledger
                .trace_slots(trace, 1)
                .map_err(PhysicalRecoverySelectedRecordReadDenial::from)?;
            ledger
                .transient(frame_length as u64)
                .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::ResidentBoundExceeded)?;
        }
        let frame = discovery
            .read_extent_range(
                placement.arena_range(),
                relative,
                u32::try_from(frame_length)
                    .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?,
                page_limit,
            )
            .map_err(
                |failure| PhysicalRecoverySelectedRecordReadDenial::ChunkRead { ordinal, failure },
            )?;
        let absolute = placement
            .arena_range()
            .offset()
            .checked_add(relative)
            .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        let frame_range = PhysicalByteRange::new(absolute, frame_length as u64)
            .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        let scope = PhysicalArtifactScope::extent_chunk(
            discovery.store_identity(),
            format,
            coordinate,
            frame_range,
            placement.arena_range(),
        );
        if let Some(builder) = witness.as_mut() {
            crate::integrity_ingress::admit_extent_chunk_projection_for_selected_record(
                &frame,
                scope,
                admitted.membership,
                trace,
                builder,
            )
            .map_err(
                |denial| PhysicalRecoverySelectedRecordReadDenial::ChunkIntegrity {
                    ordinal,
                    denial,
                },
            )?
            .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        } else {
            crate::integrity_ingress::admit_extent_chunk_projection(
                &frame,
                scope,
                admitted.membership,
                trace,
            )
            .map_err(|denial| {
                PhysicalRecoverySelectedRecordReadDenial::ChunkIntegrity { ordinal, denial }
            })?;
        }
        let frame_bytes = frame
            .bytes()
            .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        let (chunk, observed_format) = decode_extent_chunk(frame_bytes, coordinate)
            .map_err(|_| PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)?;
        if observed_format != format || chunk.len() != length {
            return Err(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload);
        }
        payload.extend_from_slice(chunk);
        *scratch = (*scratch).max(payload.capacity() as u64 + frame_length as u64);
    }
    if payload.len() as u64 != manifest.logical_bytes() {
        return Err(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload);
    }
    let witness = witness
        .map(|builder| {
            builder
                .finish()
                .ok_or(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload)
        })
        .transpose()?;
    if witness.is_some_and(|witness| !witness.matches_frame(&payload)) {
        return Err(PhysicalRecoverySelectedRecordReadDenial::InvalidPayload);
    }
    Ok((payload, witness))
}
