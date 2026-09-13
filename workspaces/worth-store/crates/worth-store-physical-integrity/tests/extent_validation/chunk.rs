use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    encode_extent_chunk, ExtentChunkCoordinate, PhysicalPageSizeClass,
};
use worth_store_physical_integrity::{
    validate_extent_chunk, ExtentChunkIntegrityValidation, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, chunk_payload_capacity, chunk_scope, extent_cell,
    field_range, format, record, reseal_durable_frame, store, validated_manifest, ExtentFixture,
    CHUNK_OFFSET,
};

#[test]
fn clean_chunk_seals_manifest_membership_metadata_incarnation_and_validation_record() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let other_incarnation = bytes.clone();
    let scope = fixture.tail_chunk_scope();
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_extent_chunk(artifact, scope, &manifest);
    let ExtentChunkIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent chunk rejected")
    };

    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.record_format(), fixture.format);
    assert_eq!(validated.record(), fixture.record);
    assert_eq!(validated.extent_cell(), fixture.extent);
    assert_eq!(validated.logical_bytes(), fixture.logical_bytes);
    assert_eq!(
        validated.logical_offset(),
        chunk_payload_capacity(fixture.format)
    );
    assert_eq!(validated.ordinal(), 2);
    assert!(validated.matches_input(artifact));
    assert!(
        !validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(
            &other_incarnation
        ))
    );
    assert!(validated.into_validation_record().matches_scope(scope));
    assert_eq!(
        counters.family(),
        PhysicalIntegrityArtifactFamily::ExtentChunk
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}

#[test]
fn chunk_framing_version_kind_checksum_and_truncation_are_localized() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let scope = fixture.tail_chunk_scope();

    let mut payload_flip = fixture.tail_chunk_bytes();
    payload_flip[114] ^= 1;
    assert_chunk_damage(
        &payload_flip,
        scope,
        &manifest,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = fixture.tail_chunk_bytes();
    checksum_flip[44] ^= 1;
    assert_chunk_damage(
        &checksum_flip,
        scope,
        &manifest,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_kind = fixture.tail_chunk_bytes();
    wrong_kind[8] = 6;
    reseal_durable_frame(&mut wrong_kind);
    assert_chunk_damage(
        &wrong_kind,
        scope,
        &manifest,
        PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(PhysicalFormatField::ArtifactFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let complete = fixture.tail_chunk_bytes();
    assert_chunk_damage(
        &complete[..complete.len() - 2],
        scope,
        &manifest,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(CHUNK_OFFSET + complete.len() as u64 - 2, 2).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = fixture.tail_chunk_bytes();
    unsupported_schema[9] = 3;
    reseal_durable_frame(&mut unsupported_schema);
    assert_unsupported(
        &unsupported_schema,
        scope,
        &manifest,
        PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = fixture.tail_chunk_bytes();
    unsupported_format[10..12].copy_from_slice(&2_u16.to_le_bytes());
    reseal_durable_frame(&mut unsupported_format);
    assert_unsupported(
        &unsupported_format,
        scope,
        &manifest,
        PhysicalIntegrityVersionAxis::PhysicalFormat,
        2,
    );
}

#[test]
fn chunk_record_extent_generation_ordinal_and_placement_substitution_are_localized() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let base = fixture.chunk_coordinate(2);
    let cases = [
        (
            coordinate(
                record(0x33, 8),
                fixture.extent,
                fixture.logical_bytes,
                base.logical_offset(),
                2,
            ),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            48,
            24,
            PhysicalFormatField::RecordIdentity,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(8, 5),
                fixture.logical_bytes,
                base.logical_offset(),
                2,
            ),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            72,
            8,
            PhysicalFormatField::ExtentIdentity,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(4, 6),
                fixture.logical_bytes,
                base.logical_offset(),
                2,
            ),
            PhysicalDamageCause::PhysicalGenerationMismatch,
            80,
            8,
            PhysicalFormatField::PhysicalGeneration,
        ),
        (
            coordinate(
                fixture.record,
                fixture.extent,
                fixture.logical_bytes + 1,
                base.logical_offset(),
                2,
            ),
            PhysicalDamageCause::ChildReferenceMismatch,
            88,
            8,
            PhysicalFormatField::Payload,
        ),
        (
            coordinate(fixture.record, fixture.extent, fixture.logical_bytes, 0, 3),
            PhysicalDamageCause::ChildReferenceMismatch,
            28,
            8,
            PhysicalFormatField::ChunkOrdinal,
        ),
        (
            coordinate(fixture.record, fixture.extent, fixture.logical_bytes, 0, 2),
            PhysicalDamageCause::ChildReferenceMismatch,
            96,
            8,
            PhysicalFormatField::ChildReference,
        ),
    ];
    for (coordinate, cause, offset, length, field) in cases {
        let bytes = encode_extent_chunk(fixture.format, coordinate, b"tail!").unwrap();
        let scope = chunk_scope(
            fixture.store,
            fixture.format,
            coordinate,
            bytes.len() as u64,
        );
        assert_chunk_damage(
            &bytes,
            scope,
            &manifest,
            cause,
            field_range(scope, offset, length),
            Some(field),
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn chunk_manifest_store_format_and_canonical_length_membership_cannot_be_substituted() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let coordinate = fixture.chunk_coordinate(2);
    let bytes = fixture.tail_chunk_bytes();

    let other_store_scope = chunk_scope(store(8), fixture.format, coordinate, bytes.len() as u64);
    assert_chunk_damage(
        &bytes,
        other_store_scope,
        &manifest,
        PhysicalDamageCause::StoreIdentityMismatch,
        other_store_scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );

    let other_format = format(PhysicalPageSizeClass::KiB32);
    let other_format_bytes = encode_extent_chunk(other_format, coordinate, b"tail!").unwrap();
    let other_format_scope = chunk_scope(
        fixture.store,
        other_format,
        coordinate,
        other_format_bytes.len() as u64,
    );
    assert_chunk_damage(
        &other_format_bytes,
        other_format_scope,
        &manifest,
        PhysicalDamageCause::FormatMismatch,
        field_range(other_format_scope, 10, 10),
        Some(PhysicalFormatField::FormatDeclaration),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let short_bytes = encode_extent_chunk(fixture.format, coordinate, b"tail").unwrap();
    let short_scope = chunk_scope(
        fixture.store,
        fixture.format,
        coordinate,
        short_bytes.len() as u64,
    );
    assert_chunk_damage(
        &short_bytes,
        short_scope,
        &manifest,
        PhysicalDamageCause::FramingLengthMismatch,
        field_range(short_scope, 104, 4),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );
}

fn coordinate(
    record: worth_store_physical_format::PersistedRecordIdentity,
    extent: worth_store_physical_format::RecordExtentGenerationCell,
    logical_bytes: u64,
    logical_offset: u64,
    ordinal: u32,
) -> ExtentChunkCoordinate {
    ExtentChunkCoordinate::new(record, extent, logical_bytes, logical_offset, ordinal).unwrap()
}

fn assert_chunk_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (validation, counters) = validate_extent_chunk(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    );
    let ExtentChunkIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged extent chunk unexpectedly validated")
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::ExtentChunk,
        bytes.len() as u64,
        cause,
    );
}

fn assert_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (
        ExtentChunkIntegrityValidation::Rejected(PhysicalIntegrityRejection::Unsupported(
            unsupported,
        )),
        counters,
    ) = validate_extent_chunk(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    )
    else {
        panic!("expected unsupported extent chunk version")
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}
