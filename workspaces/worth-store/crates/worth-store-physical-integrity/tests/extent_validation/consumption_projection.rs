use worth_store_physical_format::{
    encode_data_frame_page_lsn, DurableFrameKind, ExtentChunkCoordinate, PhysicalExtentId,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageLsn,
};
use worth_store_physical_integrity::{
    validate_extent_chunk, validate_extent_chunk_membership, ExtentChunkIntegrityValidation,
    ExtentChunkProjectionDenial, PhysicalDamageCause, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact,
};

use super::support::{chunk_scope, record, store, validated_manifest, ExtentFixture};

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
