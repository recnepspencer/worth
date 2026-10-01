use worth_store_physical_format::{
    encode_data_frame_page_lsn, encode_extent_chunk, DurableExtentRecordPlacement,
    DurableFrameKind, ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PhysicalExtentId,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageLsn,
};
use worth_store_physical_integrity::{
    validate_extent_chunk, validate_extent_chunk_membership, ExtentChunkIntegrityValidation,
    ExtentChunkProjectionDenial, PhysicalArtifactScope, PhysicalDamageCause,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass, SelectedExtentPayloadBuilder,
    UntrustedPhysicalArtifact,
};

use super::support::{
    chunk_payload_capacity, chunk_scope, manifest_scope, record, store, validated_manifest,
    ExtentFixture,
};

#[test]
fn selected_payload_witness_requires_every_exact_validated_chunk() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let first_payload = vec![b'x'; chunk_payload_capacity(fixture.format) as usize];
    let first =
        encode_extent_chunk(fixture.format, fixture.chunk_coordinate(1), &first_payload).unwrap();
    let last = fixture.tail_chunk_bytes();
    let first_scope = chunk_scope(
        fixture.store,
        fixture.format,
        fixture.chunk_coordinate(1),
        first.len() as u64,
    );
    let first_input = UntrustedPhysicalArtifact::from_bounded_bytes(&first);
    let last_input = UntrustedPhysicalArtifact::from_bounded_bytes(&last);
    let (ExtentChunkIntegrityValidation::Intact(first_validated), _) =
        validate_extent_chunk_membership(first_input, first_scope, manifest.membership())
    else {
        panic!("first extent chunk must validate")
    };
    let (ExtentChunkIntegrityValidation::Intact(last_validated), _) =
        validate_extent_chunk_membership(
            last_input,
            fixture.tail_chunk_scope(),
            manifest.membership(),
        )
    else {
        panic!("last extent chunk must validate")
    };
    let foreign_manifest = validated_manifest(
        &manifest_bytes,
        manifest_scope(
            store(9),
            fixture.format,
            fixture.placement(),
            manifest_bytes.len() as u64,
        ),
    );
    let foreign_scope = chunk_scope(
        store(9),
        fixture.format,
        fixture.chunk_coordinate(1),
        first.len() as u64,
    );
    let (ExtentChunkIntegrityValidation::Intact(foreign_validated), _) =
        validate_extent_chunk(first_input, foreign_scope, &foreign_manifest)
    else {
        panic!("same-coordinate foreign chunk must validate under its own store")
    };
    let mut builder =
        SelectedExtentPayloadBuilder::new(manifest.membership(), fixture.placement()).unwrap();
    assert!(
        builder.append(&foreign_validated, first_input).is_none(),
        "another store's valid chunk cannot enter this manifest membership"
    );
    let other_arena = ExtentArenaRange::new(
        ExtentArenaId::new(2).unwrap(),
        fixture.arena_range().offset(),
        fixture.arena_range().length(),
    )
    .unwrap();
    let other_placement = DurableExtentRecordPlacement::legacy_unknown(
        fixture.record,
        fixture.extent,
        fixture.logical_bytes,
        other_arena,
    )
    .unwrap();
    let other_manifest = validated_manifest(
        &manifest_bytes,
        manifest_scope(
            fixture.store,
            fixture.format,
            other_placement,
            manifest_bytes.len() as u64,
        ),
    );
    let other_scope = PhysicalArtifactScope::extent_chunk(
        fixture.store,
        fixture.format,
        fixture.chunk_coordinate(1),
        first_scope.byte_range(),
        other_arena,
    );
    let (ExtentChunkIntegrityValidation::Intact(other_validated), _) =
        validate_extent_chunk(first_input, other_scope, &other_manifest)
    else {
        panic!("same-coordinate other-arena chunk must validate under its own placement")
    };
    assert!(
        builder.append(&other_validated, first_input).is_none(),
        "another arena's valid chunk cannot enter this manifest membership"
    );
    assert!(
        builder.append(&last_validated, last_input).is_none(),
        "no skipped first chunk"
    );
    let copied = first.clone();
    assert!(
        builder
            .append(
                &first_validated,
                UntrustedPhysicalArtifact::from_bounded_bytes(&copied)
            )
            .is_none(),
        "equal bytes from a different C.9 incarnation grant no custody"
    );
    builder.append(&first_validated, first_input).unwrap();
    assert!(
        builder.finish().is_none(),
        "partial payload grants no custody"
    );
    let mut builder =
        SelectedExtentPayloadBuilder::new(manifest.membership(), fixture.placement()).unwrap();
    builder.append(&first_validated, first_input).unwrap();
    builder.append(&last_validated, last_input).unwrap();
    let witness = builder.finish().unwrap();
    let mut complete = first_payload;
    complete.extend_from_slice(b"tail!");
    assert!(witness.matches_frame(&complete));
    complete[0] ^= 1;
    assert!(!witness.matches_frame(&complete));
}

#[test]
fn sealed_extent_chunk_projects_exact_payload_and_page_lsn() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let mut bytes = fixture.tail_chunk_bytes();
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::Extent,
        PhysicalPageLsn::new(144),
    )
    .unwrap();
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_extent_chunk(input, fixture.tail_chunk_scope(), &manifest);
    let ExtentChunkIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent chunk rejected");
    };

    let projection = validated
        .project_chunk(input, fixture.chunk_coordinate(2))
        .unwrap();

    assert_eq!(projection.coordinate(), fixture.chunk_coordinate(2));
    assert_eq!(projection.page_lsn(), PhysicalPageLsn::new(144));
    assert_eq!(&bytes[projection.payload_range()], b"tail!");
}

#[test]
fn extent_projection_denies_foreign_incarnation_generation_and_ordinal() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_extent_chunk(input, fixture.tail_chunk_scope(), &manifest);
    let ExtentChunkIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent chunk rejected");
    };
    let exact = fixture.chunk_coordinate(2);
    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_chunk(
                UntrustedPhysicalArtifact::from_bounded_bytes(&equal_copy),
                exact,
            )
            .unwrap_err(),
        ExtentChunkProjectionDenial::InputIncarnationMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                coordinate(
                    fixture,
                    extent_cell(fixture.extent.extent_id().get(), 6),
                    exact.logical_offset(),
                    exact.ordinal(),
                ),
            )
            .unwrap_err(),
        ExtentChunkProjectionDenial::ExtentGenerationMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                ExtentChunkCoordinate::new(
                    record(0x23, 7),
                    fixture.extent,
                    fixture.logical_bytes,
                    exact.logical_offset(),
                    exact.ordinal(),
                )
                .unwrap(),
            )
            .unwrap_err(),
        ExtentChunkProjectionDenial::RecordIdentityMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                coordinate(
                    fixture,
                    extent_cell(fixture.extent.extent_id().get() + 1, 5),
                    exact.logical_offset(),
                    exact.ordinal(),
                ),
            )
            .unwrap_err(),
        ExtentChunkProjectionDenial::ExtentIdentityMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                ExtentChunkCoordinate::new(
                    fixture.record,
                    fixture.extent,
                    fixture.logical_bytes + 1,
                    exact.logical_offset(),
                    exact.ordinal(),
                )
                .unwrap(),
            )
            .unwrap_err(),
        ExtentChunkProjectionDenial::LogicalLengthMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                coordinate(fixture, fixture.extent, 0, exact.ordinal() + 1),
            )
            .unwrap_err(),
        ExtentChunkProjectionDenial::LogicalOffsetMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                coordinate(
                    fixture,
                    fixture.extent,
                    exact.logical_offset(),
                    exact.ordinal() + 1,
                ),
            )
            .unwrap_err(),
        ExtentChunkProjectionDenial::ChunkOrdinalMismatch
    );
}

#[test]
fn admitted_manifest_membership_rejects_a_foreign_store_scope() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let coordinate = fixture.chunk_coordinate(2);
    let foreign_scope = chunk_scope(store(9), fixture.format, coordinate, bytes.len() as u64);

    let (validation, counters) = validate_extent_chunk_membership(
        UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        foreign_scope,
        manifest.membership(),
    );

    let ExtentChunkIntegrityValidation::Rejected(PhysicalIntegrityRejection::Damaged(damage)) =
        validation
    else {
        panic!("foreign store scope must be rejected as localized physical damage")
    };
    assert_eq!(damage.cause(), PhysicalDamageCause::StoreIdentityMismatch);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(
            PhysicalDamageCause::StoreIdentityMismatch,
        )),
        1
    );
}

fn coordinate(
    fixture: ExtentFixture,
    extent: worth_store_physical_format::RecordExtentGenerationCell,
    logical_offset: u64,
    ordinal: u32,
) -> ExtentChunkCoordinate {
    ExtentChunkCoordinate::new(
        fixture.record,
        extent,
        fixture.logical_bytes,
        logical_offset,
        ordinal,
    )
    .unwrap()
}

fn extent_cell(
    extent: u64,
    generation: u64,
) -> worth_store_physical_format::RecordExtentGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(extent).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(generation).unwrap())
}
