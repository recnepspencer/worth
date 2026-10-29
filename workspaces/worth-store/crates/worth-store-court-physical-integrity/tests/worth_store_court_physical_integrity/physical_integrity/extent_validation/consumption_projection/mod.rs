#[allow(unused_imports)] use crate::extent_validation::consumption_projection::*;
use {worth_store_physical_format::encode_data_frame_page_lsn, worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageLsn};
use {worth_store_physical_integrity::validate_extent_chunk, worth_store_physical_integrity::validate_extent_chunk_membership, worth_store_physical_integrity::ExtentChunkIntegrityValidation, worth_store_physical_integrity::ExtentChunkProjectionDenial, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::SelectedExtentPayloadBuilder, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::extent_validation::chunk_payload_capacity, crate::support::extent_validation::chunk_scope, crate::support::extent_validation::manifest_scope, crate::support::extent_validation::record, crate::support::extent_validation::store, crate::support::extent_validation::validated_manifest, crate::support::extent_validation::ExtentFixture};

#[test]
fn selected_payload_witness_requires_every_exact_validated_chunk() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
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
    let first_input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&first);
    let last_input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&last);
    let (worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(first_validated), _) =
        validate_extent_chunk_membership(first_input, first_scope, manifest.membership())
    else {
        panic!("first extent chunk must validate")
    };
    let (worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(last_validated), _) =
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
    let (worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(foreign_validated), _) =
        validate_extent_chunk(first_input, foreign_scope, &foreign_manifest)
    else {
        panic!("same-coordinate foreign chunk must validate under its own store")
    };
    let mut builder =
        worth_store_physical_integrity::SelectedExtentPayloadBuilder::new(manifest.membership(), fixture.placement()).unwrap();
    assert!(
        builder.append(&foreign_validated, first_input).is_none(),
        "another store's valid chunk cannot enter this manifest membership"
    );
    let other_arena = worth_store_physical_format::ExtentArenaRange::new(
        worth_store_physical_format::ExtentArenaId::new(2).unwrap(),
        fixture.arena_range().offset(),
        fixture.arena_range().length(),
    )
    .unwrap();
    let other_placement = worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
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
    let other_scope = worth_store_physical_integrity::PhysicalArtifactScope::extent_chunk(
        fixture.store,
        fixture.format,
        fixture.chunk_coordinate(1),
        first_scope.byte_range(),
        other_arena,
    );
    let (worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(other_validated), _) =
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
                worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&copied)
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
        worth_store_physical_integrity::SelectedExtentPayloadBuilder::new(manifest.membership(), fixture.placement()).unwrap();
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
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let mut bytes = fixture.tail_chunk_bytes();
    encode_data_frame_page_lsn(
        &mut bytes,
        worth_store_physical_format::DurableFrameKind::Extent,
        worth_store_physical_format::PhysicalPageLsn::new(144),
    )
    .unwrap();
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_extent_chunk(input, fixture.tail_chunk_scope(), &manifest);
    let worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent chunk rejected");
    };

    let projection = validated
        .project_chunk(input, fixture.chunk_coordinate(2))
        .unwrap();

    assert_eq!(projection.coordinate(), fixture.chunk_coordinate(2));
    assert_eq!(projection.page_lsn(), worth_store_physical_format::PhysicalPageLsn::new(144));
    assert_eq!(&bytes[projection.payload_range()], b"tail!");
}

#[test]
fn extent_projection_denies_foreign_incarnation_generation_and_ordinal() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_extent_chunk(input, fixture.tail_chunk_scope(), &manifest);
    let worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent chunk rejected");
    };
    let exact = fixture.chunk_coordinate(2);
    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_chunk(
                worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&equal_copy),
                exact,
            )
            .unwrap_err(),
        worth_store_physical_integrity::ExtentChunkProjectionDenial::InputIncarnationMismatch
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
        worth_store_physical_integrity::ExtentChunkProjectionDenial::ExtentGenerationMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                worth_store_physical_format::ExtentChunkCoordinate::new(
                    record(0x23, 7),
                    fixture.extent,
                    fixture.logical_bytes,
                    exact.logical_offset(),
                    exact.ordinal(),
                )
                .unwrap(),
            )
            .unwrap_err(),
        worth_store_physical_integrity::ExtentChunkProjectionDenial::RecordIdentityMismatch
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
        worth_store_physical_integrity::ExtentChunkProjectionDenial::ExtentIdentityMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                worth_store_physical_format::ExtentChunkCoordinate::new(
                    fixture.record,
                    fixture.extent,
                    fixture.logical_bytes + 1,
                    exact.logical_offset(),
                    exact.ordinal(),
                )
                .unwrap(),
            )
            .unwrap_err(),
        worth_store_physical_integrity::ExtentChunkProjectionDenial::LogicalLengthMismatch
    );
    assert_eq!(
        validated
            .project_chunk(
                input,
                coordinate(fixture, fixture.extent, 0, exact.ordinal() + 1),
            )
            .unwrap_err(),
        worth_store_physical_integrity::ExtentChunkProjectionDenial::LogicalOffsetMismatch
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
        worth_store_physical_integrity::ExtentChunkProjectionDenial::ChunkOrdinalMismatch
    );
}

#[test]
fn admitted_manifest_membership_rejects_a_foreign_store_scope() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let coordinate = fixture.chunk_coordinate(2);
    let foreign_scope = chunk_scope(store(9), fixture.format, coordinate, bytes.len() as u64);

    let (validation, counters) = validate_extent_chunk_membership(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        foreign_scope,
        manifest.membership(),
    );

    let worth_store_physical_integrity::ExtentChunkIntegrityValidation::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage)) =
        validation
    else {
        panic!("foreign store scope must be rejected as localized physical damage")
    };
    assert_eq!(damage.cause(), worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(
            worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch,
        )),
        1
    );
}
