#[allow(unused_imports)]
use crate::extent_validation::scope_adversaries::*;

#[test]
fn valid_alternate_chunk_identities_cannot_replace_the_certified_scope() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
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
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            48,
            24,
            worth_store_physical_integrity::PhysicalFormatField::RecordIdentity,
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(8, 5),
                fixture.logical_bytes,
                expected.logical_offset(),
                2,
            ),
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            72,
            8,
            worth_store_physical_integrity::PhysicalFormatField::ExtentIdentity,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(4, 6),
                fixture.logical_bytes,
                expected.logical_offset(),
                2,
            ),
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            80,
            8,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        ),
        (
            coordinate(
                fixture.record,
                fixture.extent,
                fixture.logical_bytes + 1,
                expected.logical_offset(),
                2,
            ),
            worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
            88,
            8,
            worth_store_physical_integrity::PhysicalFormatField::Payload,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            coordinate(fixture.record, fixture.extent, fixture.logical_bytes, 0, 2),
            worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
            96,
            8,
            worth_store_physical_integrity::PhysicalFormatField::ChildReference,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            coordinate(
                fixture.record,
                fixture.extent,
                fixture.logical_bytes,
                expected.logical_offset(),
                3,
            ),
            worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
            28,
            8,
            worth_store_physical_integrity::PhysicalFormatField::ChunkOrdinal,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
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
    let fixture = crate::support::extent_validation::ExtentFixture::new();
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
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        20,
        8,
        worth_store_physical_integrity::PhysicalFormatField::EncodedLength,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut payload_length_lie = fixture.tail_chunk_bytes();
    payload_length_lie[104..108].copy_from_slice(&4_u32.to_le_bytes());
    reseal_durable_frame(&mut payload_length_lie);
    assert_chunk_rejected(
        &payload_length_lie,
        scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        104,
        4,
        worth_store_physical_integrity::PhysicalFormatField::EncodedLength,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn chunk_format_declaration_must_match_the_certified_chunk_scope() {
    let wide_format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB32);
    let fixture = ExtentFixture {
        format: wide_format,
        logical_bytes: chunk_payload_capacity(wide_format) + 5,
        ..crate::support::extent_validation::ExtentFixture::new()
    };
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let narrow_scope = worth_store_physical_integrity::PhysicalArtifactScope::extent_chunk(
        fixture.store,
        format(worth_store_physical_format::PhysicalPageSizeClass::KiB16),
        fixture.chunk_coordinate(2),
        fixture.tail_chunk_scope().byte_range(),
        fixture.arena_range(),
    );

    assert_chunk_rejected(
        &bytes,
        narrow_scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
        10,
        10,
        worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn extent_family_scopes_are_rejected_before_cross_family_decoding() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let chunk_bytes = fixture.tail_chunk_bytes();

    let (
        worth_store_physical_integrity::ExtentChunkIntegrityValidation::Rejected(chunk_rejection),
        chunk_counters,
    ) = validate_extent_chunk(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &manifest_bytes,
        ),
        fixture.manifest_scope(),
        &manifest,
    )
    else {
        panic!("manifest scope admitted by chunk validator")
    };
    assert_damage(
        chunk_rejection,
        fixture.manifest_scope(),
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        fixture.manifest_scope().byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(
        chunk_counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk,
        manifest_bytes.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
    );

    let (
        worth_store_physical_integrity::ExtentManifestIntegrityValidation::Rejected(
            manifest_rejection,
        ),
        manifest_counters,
    ) = validate_extent_manifest(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&chunk_bytes),
        fixture.tail_chunk_scope(),
    )
    else {
        panic!("chunk scope admitted by manifest validator")
    };
    assert_damage(
        manifest_rejection,
        fixture.tail_chunk_scope(),
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        fixture.tail_chunk_scope().byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(
        manifest_counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest,
        chunk_bytes.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
    );
}
