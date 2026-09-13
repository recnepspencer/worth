use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    encode_extent_chunk, ExtentChunkCoordinate, PhysicalPageSizeClass,
};
use worth_store_physical_integrity::{
    validate_extent_chunk, validate_extent_manifest, ExtentChunkIntegrityValidation,
    ExtentManifestIntegrityValidation, PhysicalBlastRadius, PhysicalDamageCause,
    PhysicalFormatField, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, chunk_payload_capacity, chunk_scope, extent_cell,
    field_range, format, record, reseal_durable_frame, validated_manifest, ExtentFixture,
};

#[test]
fn valid_alternate_chunk_identities_cannot_replace_the_certified_scope() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let expected = fixture.chunk_coordinate(2);
    let cases = [
        (
            coordinate(
                record(0x33, 8),
                fixture.extent,
                fixture.logical_bytes,
                expected.logical_offset(),
                2,
            ),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            48,
            24,
            PhysicalFormatField::RecordIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(8, 5),
                fixture.logical_bytes,
                expected.logical_offset(),
                2,
            ),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            72,
            8,
            PhysicalFormatField::ExtentIdentity,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(4, 6),
                fixture.logical_bytes,
                expected.logical_offset(),
                2,
            ),
            PhysicalDamageCause::PhysicalGenerationMismatch,
            80,
            8,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        (
            coordinate(
                fixture.record,
                fixture.extent,
                fixture.logical_bytes + 1,
                expected.logical_offset(),
                2,
            ),
            PhysicalDamageCause::ChildReferenceMismatch,
            88,
            8,
            PhysicalFormatField::Payload,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            coordinate(fixture.record, fixture.extent, fixture.logical_bytes, 0, 2),
            PhysicalDamageCause::ChildReferenceMismatch,
            96,
            8,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            coordinate(
                fixture.record,
                fixture.extent,
                fixture.logical_bytes,
                expected.logical_offset(),
                3,
            ),
            PhysicalDamageCause::SequenceMismatch,
            28,
            8,
            PhysicalFormatField::ChunkOrdinal,
            PhysicalBlastRadius::CanonicalFrame,
        ),
    ];

    for (alternate, cause, offset, length, field, blast_radius) in cases {
        let bytes = encode_extent_chunk(fixture.format, alternate, b"tail!").unwrap();
        assert_chunk_rejected(
            &bytes,
            fixture.tail_chunk_scope(),
            &manifest,
            cause,
            offset,
            length,
            field,
            blast_radius,
        );
    }
}

#[test]
fn chunk_total_and_payload_length_lies_are_distinctly_localized() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let scope = fixture.tail_chunk_scope();

    let mut total_length_lie = fixture.tail_chunk_bytes();
    total_length_lie[24..28].copy_from_slice(&55_u32.to_le_bytes());
    reseal_durable_frame(&mut total_length_lie);
    assert_chunk_rejected(
        &total_length_lie,
        scope,
        &manifest,
        PhysicalDamageCause::FramingLengthMismatch,
        20,
        8,
        PhysicalFormatField::EncodedLength,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut payload_length_lie = fixture.tail_chunk_bytes();
    payload_length_lie[104..108].copy_from_slice(&4_u32.to_le_bytes());
    reseal_durable_frame(&mut payload_length_lie);
    assert_chunk_rejected(
        &payload_length_lie,
        scope,
        &manifest,
        PhysicalDamageCause::FramingLengthMismatch,
        104,
        4,
        PhysicalFormatField::EncodedLength,
        PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn chunk_format_declaration_must_match_the_certified_chunk_scope() {
    let wide_format = format(PhysicalPageSizeClass::KiB32);
    let fixture = ExtentFixture {
        format: wide_format,
        logical_bytes: chunk_payload_capacity(wide_format) + 5,
        ..ExtentFixture::new()
    };
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let narrow_scope = chunk_scope(
        fixture.store,
        format(PhysicalPageSizeClass::KiB16),
        fixture.chunk_coordinate(2),
        bytes.len() as u64,
    );

    assert_chunk_rejected(
        &bytes,
        narrow_scope,
        &manifest,
        PhysicalDamageCause::FormatMismatch,
        10,
        10,
        PhysicalFormatField::FormatDeclaration,
        PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn extent_family_scopes_are_rejected_before_cross_family_decoding() {
    let fixture = ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let chunk_bytes = fixture.tail_chunk_bytes();

    let (ExtentChunkIntegrityValidation::Rejected(chunk_rejection), chunk_counters) =
        validate_extent_chunk(
            UntrustedPhysicalArtifact::from_bounded_bytes(&manifest_bytes),
            fixture.manifest_scope(),
            &manifest,
        )
    else {
        panic!("manifest scope admitted by chunk validator")
    };
    assert_damage(
        chunk_rejection,
        fixture.manifest_scope(),
        PhysicalDamageCause::FamilyMismatch,
        fixture.manifest_scope().byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(
        chunk_counters,
        PhysicalIntegrityArtifactFamily::ExtentChunk,
        manifest_bytes.len() as u64,
        PhysicalDamageCause::FamilyMismatch,
    );

    let (ExtentManifestIntegrityValidation::Rejected(manifest_rejection), manifest_counters) =
        validate_extent_manifest(
            UntrustedPhysicalArtifact::from_bounded_bytes(&chunk_bytes),
            fixture.tail_chunk_scope(),
        )
    else {
        panic!("chunk scope admitted by manifest validator")
    };
    assert_damage(
        manifest_rejection,
        fixture.tail_chunk_scope(),
        PhysicalDamageCause::FamilyMismatch,
        fixture.tail_chunk_scope().byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(
        manifest_counters,
        PhysicalIntegrityArtifactFamily::ExtentManifest,
        chunk_bytes.len() as u64,
        PhysicalDamageCause::FamilyMismatch,
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

#[allow(clippy::too_many_arguments)]
fn assert_chunk_rejected(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
    cause: PhysicalDamageCause,
    offset: u64,
    length: u64,
    field: PhysicalFormatField,
    blast_radius: PhysicalBlastRadius,
) {
    let (ExtentChunkIntegrityValidation::Rejected(rejection), counters) = validate_extent_chunk(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    ) else {
        panic!("substituted extent chunk unexpectedly validated")
    };
    assert_damage(
        rejection,
        scope,
        cause,
        field_range(scope, offset, length),
        Some(field),
        blast_radius,
    );
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::ExtentChunk,
        bytes.len() as u64,
        cause,
    );
}
