//! Retained WAL expiry semantics are checked before any redo effect. A WAL
//! binding proves terminal bytes, not the declaration or checkpoint horizon.

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_extent_chunk, BlobAbandonmentReasonV1, BlobRecordV1, BlobSessionAbandonedV1,
    BlobSessionDeclarationV1, CurrentPhysicalRecordPlacement, DurableExtentManifest,
    ExtentArenaFrameLayout, ExtentChunkCoordinate, PersistedPhysicalRecoveryOperation,
    BLOB_RECORD_HEADER_BYTES, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use super::super::manifest_entry_budget::ManifestEntryBudget;
use super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::historical_publication::{self, HistoricalFailure};

// BlobSessionDeclarationV1's fixed 108-byte payload plus its 48-byte envelope.
const DECLARATION_FRAME_BYTES: u64 = (BLOB_RECORD_HEADER_BYTES + 108) as u64;

pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    for index in 0..basis.redo.projections().len() {
        let projection = &basis.redo.projections()[index];
        let PersistedPhysicalRecoveryOperation::SessionAbandoned(binding) =
            projection.materialization().operation()
        else {
            continue;
        };
        let Some(bytes) = basis
            .redo
            .blob_semantic_record_bytes(projection.operation())
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if <[u8; 32]>::from(Sha256::digest(bytes)) != binding.record_payload_sha256() {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let Ok(BlobRecordV1::SessionAbandoned(terminal)) =
            worth_store_physical_format::decode_blob_record(bytes)
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        let BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence,
        } = terminal.reason()
        else {
            continue;
        };
        let Some(current_checkpoint) = context.selection.checkpoint() else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        let current_sequence = current_checkpoint
            .checkpoint()
            .source()
            .identity()
            .sequence();
        if current_sequence < checkpoint_sequence {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let selected_generation = context
            .selection
            .root()
            .selected()
            .selector()
            .root_generation();
        let format = context.authority.record_format;
        let (next, declaration) = historical_publication::observe(
            context,
            basis,
            selected_generation,
            terminal.declaration_record(),
            |discovery, _, route, budget, trace, scratch| {
                read_selected_declaration(
                    discovery, format, route, terminal, budget, trace, scratch,
                )
            },
        )?;
        context = next;
        if !crosses_declared_horizon(declaration, checkpoint_sequence, Some(current_sequence)) {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    }
    Ok(context)
}

fn crosses_declared_horizon(
    declaration: BlobSessionDeclarationV1,
    witness: NonZeroU64,
    selected_checkpoint: Option<NonZeroU64>,
) -> bool {
    declaration.max_checkpoint_sequence() < witness.get()
        && selected_checkpoint.is_some_and(|current| current >= witness)
}

fn read_selected_declaration(
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    route: Option<CurrentPhysicalRecordPlacement>,
    terminal: BlobSessionAbandonedV1,
    budget: &mut ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
) -> Result<BlobSessionDeclarationV1, HistoricalFailure> {
    let Some(CurrentPhysicalRecordPlacement::Extent(placement)) = route else {
        return Err(HistoricalFailure::Invalid);
    };
    if placement.record() != terminal.declaration_record()
        || placement.payload_bytes() != DECLARATION_FRAME_BYTES
    {
        return Err(HistoricalFailure::Invalid);
    }
    let limit = u64::from(format.page_size().bytes());
    budget.consume(1)?;
    let observed = discovery
        .read_extent_manifest(placement.arena_range(), limit)
        .map_err(historical_publication::discovery_failure)?;
    let bytes = observed.bytes().ok_or(HistoricalFailure::Invalid)?;
    let manifest_range =
        PhysicalByteRange::new(placement.arena_range().offset(), bytes.len() as u64)
            .map_err(|_| HistoricalFailure::Invalid)?;
    let scope = PhysicalArtifactScope::extent_manifest(
        discovery.store_identity(),
        format,
        placement,
        manifest_range,
    );
    let admitted =
        crate::integrity_ingress::admit_extent_manifest_projection(&observed, scope, trace)
            .map_err(|_| HistoricalFailure::Invalid)?;
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
        manifest.record() == placement.record()
            && manifest.extent_cell() == placement.extent_cell()
            && manifest.logical_bytes() == placement.payload_bytes()
            && manifest.chunk_count() == 1
    })
    .ok_or(HistoricalFailure::Invalid)?;
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment())
        .filter(|layout| layout.admits(placement.arena_range(), 1))
        .ok_or(HistoricalFailure::Invalid)?;
    drop(observed);
    let coordinate = ExtentChunkCoordinate::new(
        manifest.record(),
        manifest.extent_cell(),
        manifest.logical_bytes(),
        0,
        1,
    )
    .ok_or(HistoricalFailure::Invalid)?;
    let relative = layout.chunk_offset(1).ok_or(HistoricalFailure::Invalid)?;
    let frame_length = u32::try_from(
        DURABLE_EXTENT_FRAME_HEADER_BYTES as u64
            + EXTENT_CHUNK_METADATA_BYTES as u64
            + manifest.logical_bytes(),
    )
    .map_err(|_| HistoricalFailure::Invalid)?;
    let frame = discovery
        .read_extent_range(placement.arena_range(), relative, frame_length, limit)
        .map_err(historical_publication::discovery_failure)?;
    let absolute = placement
        .arena_range()
        .offset()
        .checked_add(relative)
        .ok_or(HistoricalFailure::Invalid)?;
    let frame_range = PhysicalByteRange::new(absolute, u64::from(frame_length))
        .map_err(|_| HistoricalFailure::Invalid)?;
    let scope = PhysicalArtifactScope::extent_chunk(
        discovery.store_identity(),
        format,
        coordinate,
        frame_range,
        placement.arena_range(),
    );
    crate::integrity_ingress::admit_extent_chunk_projection(
        &frame,
        scope,
        admitted.membership,
        trace,
    )
    .map_err(|_| HistoricalFailure::Invalid)?;
    let bytes = frame.bytes().ok_or(HistoricalFailure::Invalid)?;
    let (payload, observed_format) =
        decode_extent_chunk(bytes, coordinate).map_err(|_| HistoricalFailure::Invalid)?;
    if observed_format != format
        || <[u8; 32]>::from(Sha256::digest(payload)) != terminal.declaration_digest()
    {
        return Err(HistoricalFailure::Invalid);
    }
    let declaration =
        BlobSessionDeclarationV1::decode(payload).map_err(|_| HistoricalFailure::Invalid)?;
    if declaration.store() != terminal.store()
        || declaration.session() != terminal.session()
        || declaration.store() != discovery.store_identity().bytes()
    {
        return Err(HistoricalFailure::Invalid);
    }
    // The decoded payload borrows the one bounded frame; it is not a second
    // object-sized buffer. The shared reader already charges root/route pages.
    *scratch = (*scratch).max(u64::from(frame_length));
    Ok(declaration)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(maximum: u64) -> BlobSessionDeclarationV1 {
        BlobSessionDeclarationV1::new(
            [1; 16],
            [2; 16],
            [3; 16],
            [4; 32],
            64 << 10,
            128 << 10,
            1 << 20,
            maximum,
        )
        .unwrap()
    }

    #[test]
    fn selected_later_checkpoint_authenticates_crossed_declaration_horizon() {
        let witness = NonZeroU64::new(7).unwrap();
        assert!(crosses_declared_horizon(
            declaration(5),
            witness,
            NonZeroU64::new(9),
        ));
        assert!(!crosses_declared_horizon(
            declaration(7),
            witness,
            NonZeroU64::new(9),
        ));
        assert!(!crosses_declared_horizon(
            declaration(5),
            witness,
            NonZeroU64::new(6),
        ));
        assert!(!crosses_declared_horizon(declaration(5), witness, None));
    }

    #[test]
    fn reading_the_selected_declaration_is_charged_one_entry_before_its_read() {
        use crate::orchestration::planning::selected_world_fixture::selected_world;
        use worth_store_physical_format::{
            DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange, PersistedRecordIdentity,
            PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
            SelectedRecordContentClass, SelectedRecordRouteMetadata,
        };
        let record = PersistedRecordIdentity::new([1; 16], 9).unwrap();
        let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(9).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
        let range = ExtentArenaRange::new(ExtentArenaId::new(9).unwrap(), 0, 64).unwrap();
        let metadata =
            SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
        let route = CurrentPhysicalRecordPlacement::Extent(
            DurableExtentRecordPlacement::new_selected(
                record,
                cell,
                DECLARATION_FRAME_BYTES,
                range,
                metadata,
            )
            .unwrap(),
        );
        let terminal = BlobSessionAbandonedV1::new(
            [1; 16],
            [2; 16],
            record,
            [3; 32],
            BlobAbandonmentReasonV1::ExplicitAbort,
        )
        .unwrap();
        let read = |name: &str, observed: u64| {
            selected_world(name, 4).read(|source| {
                let mut budget = ManifestEntryBudget::new(8, observed);
                let mut trace = Default::default();
                let outcome = read_selected_declaration(
                    source.discovery,
                    source.format,
                    Some(route),
                    terminal,
                    &mut budget,
                    &mut trace,
                    &mut 0,
                )
                .map(|_| ());
                let reads = source.discovery.counters().addressed_artifacts_read;
                (outcome, budget.remaining(), reads)
            })
        };
        // None left: refused as that limit, before the read.
        assert_eq!(
            read("declaration-read-none-left", 8),
            (Err(HistoricalFailure::ManifestEntries), 0, 0)
        );
        // One left pays for the read. This world holds no such extent.
        let (outcome, remaining, _) = read("declaration-read-one-left", 7);
        assert_eq!((outcome, remaining), (Err(HistoricalFailure::Invalid), 0));
    }
}
