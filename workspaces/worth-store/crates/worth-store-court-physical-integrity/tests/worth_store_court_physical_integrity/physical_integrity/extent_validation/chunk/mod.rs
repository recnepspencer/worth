#[allow(unused_imports)]
use crate::extent_validation::chunk::*;

#[test]
fn clean_chunk_seals_manifest_membership_metadata_incarnation_and_validation_record() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let bytes = fixture.tail_chunk_bytes();
    let other_incarnation = bytes.clone();
    let scope = fixture.tail_chunk_scope();
    let artifact =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_extent_chunk(artifact, scope, &manifest);
    let worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(validated) =
        validation
    else {
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
    assert!(!validated.matches_input(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &other_incarnation
        )
    ));
    assert!(validated.into_validation_record().matches_scope(scope));
    assert_eq!(
        counters.family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}

#[test]
fn chunk_framing_version_kind_checksum_and_truncation_are_localized() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let scope = fixture.tail_chunk_scope();

    let mut payload_flip = fixture.tail_chunk_bytes();
    payload_flip[114] ^= 1;
    assert_chunk_damage(
        &payload_flip,
        scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = fixture.tail_chunk_bytes();
    checksum_flip[44] ^= 1;
    assert_chunk_damage(
        &checksum_flip,
        scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_kind = fixture.tail_chunk_bytes();
    wrong_kind[8] = 6;
    reseal_durable_frame(&mut wrong_kind);
    assert_chunk_damage(
        &wrong_kind,
        scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let complete = fixture.tail_chunk_bytes();
    assert_chunk_damage(
        &complete[..complete.len() - 2],
        scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(
            CHUNK_OFFSET + complete.len() as u64 - 2,
            2,
        )
        .unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = fixture.tail_chunk_bytes();
    unsupported_schema[9] = 3;
    reseal_durable_frame(&mut unsupported_schema);
    assert_unsupported(
        &unsupported_schema,
        scope,
        &manifest,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = fixture.tail_chunk_bytes();
    unsupported_format[10..12].copy_from_slice(&3_u16.to_le_bytes());
    reseal_durable_frame(&mut unsupported_format);
    assert_unsupported(
        &unsupported_format,
        scope,
        &manifest,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat,
        3,
    );
}

#[test]
fn chunk_record_extent_generation_ordinal_and_placement_substitution_are_localized() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
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
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            48,
            24,
            worth_store_physical_integrity::PhysicalFormatField::RecordIdentity,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(8, 5),
                fixture.logical_bytes,
                base.logical_offset(),
                2,
            ),
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            72,
            8,
            worth_store_physical_integrity::PhysicalFormatField::ExtentIdentity,
        ),
        (
            coordinate(
                fixture.record,
                extent_cell(4, 6),
                fixture.logical_bytes,
                base.logical_offset(),
                2,
            ),
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            80,
            8,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
        (
            coordinate(
                fixture.record,
                fixture.extent,
                fixture.logical_bytes + 1,
                base.logical_offset(),
                2,
            ),
            worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
            88,
            8,
            worth_store_physical_integrity::PhysicalFormatField::Payload,
        ),
        (
            coordinate(fixture.record, fixture.extent, fixture.logical_bytes, 0, 3),
            worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
            28,
            8,
            worth_store_physical_integrity::PhysicalFormatField::ChunkOrdinal,
        ),
        (
            coordinate(fixture.record, fixture.extent, fixture.logical_bytes, 0, 2),
            worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
            96,
            8,
            worth_store_physical_integrity::PhysicalFormatField::ChildReference,
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
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn chunk_manifest_store_format_and_canonical_length_membership_cannot_be_substituted() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let coordinate = fixture.chunk_coordinate(2);
    let bytes = fixture.tail_chunk_bytes();

    let other_store_scope = chunk_scope(store(8), fixture.format, coordinate, bytes.len() as u64);
    assert_chunk_damage(
        &bytes,
        other_store_scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch,
        other_store_scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let other_format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB32);
    let other_format_bytes = encode_extent_chunk(other_format, coordinate, b"tail!").unwrap();
    let other_format_scope = worth_store_physical_integrity::PhysicalArtifactScope::extent_chunk(
        fixture.store,
        other_format,
        coordinate,
        worth_store_physical_integrity::PhysicalByteRange::new(
            fixture.tail_chunk_scope().byte_range().offset(),
            other_format_bytes.len() as u64,
        )
        .unwrap(),
        fixture.arena_range(),
    );
    assert_chunk_damage(
        &other_format_bytes,
        other_format_scope,
        &manifest,
        worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
        field_range(other_format_scope, 10, 10),
        Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
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
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        field_range(short_scope, 104, 4),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
}
