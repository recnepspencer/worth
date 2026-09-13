use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{DurableExtentManifest, PhysicalPageSizeClass};
use worth_store_physical_integrity::{
    validate_extent_manifest, ExtentManifestIntegrityValidation, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, chunk_payload_capacity, extent_cell, field_range,
    format, manifest_scope, record, reseal_durable_frame, store, ExtentFixture, MANIFEST_OFFSET,
};

#[test]
fn clean_manifest_seals_geometry_placement_incarnation_and_validation_record() {
    let fixture = ExtentFixture::new();
    let bytes = fixture.manifest_bytes();
    let other_incarnation = bytes.clone();
    let scope = fixture.manifest_scope();
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_extent_manifest(artifact, scope);
    let ExtentManifestIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent manifest rejected")
    };

    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.record_format(), fixture.format);
    assert_eq!(validated.record(), fixture.record);
    assert_eq!(validated.extent_cell(), fixture.extent);
    assert_eq!(validated.logical_bytes(), fixture.logical_bytes);
    assert_eq!(validated.maximum_frame_bytes(), 16_384);
    assert_eq!(
        u64::from(validated.chunk_payload_capacity()),
        chunk_payload_capacity(fixture.format)
    );
    assert_eq!(validated.chunk_count(), 2);
    assert!(validated.matches_input(artifact));
    assert!(
        !validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(
            &other_incarnation
        ))
    );
    assert!(validated.into_validation_record().matches_scope(scope));
    assert_eq!(
        counters.family(),
        PhysicalIntegrityArtifactFamily::ExtentManifest
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), 104);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}

#[test]
fn manifest_framing_version_kind_checksum_and_truncation_are_localized() {
    let fixture = ExtentFixture::new();
    let scope = fixture.manifest_scope();

    let mut payload_flip = fixture.manifest_bytes();
    payload_flip[80] ^= 1;
    assert_manifest_damage(
        &payload_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = fixture.manifest_bytes();
    checksum_flip[44] ^= 1;
    assert_manifest_damage(
        &checksum_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_kind = fixture.manifest_bytes();
    wrong_kind[8] = 4;
    reseal_durable_frame(&mut wrong_kind);
    assert_manifest_damage(
        &wrong_kind,
        scope,
        PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(PhysicalFormatField::ArtifactFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = fixture.manifest_bytes();
    length_lie[24..28].copy_from_slice(&55_u32.to_le_bytes());
    reseal_durable_frame(&mut length_lie);
    assert_manifest_damage(
        &length_lie,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 20, 8),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let complete = fixture.manifest_bytes();
    assert_manifest_damage(
        &complete[..100],
        scope,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(MANIFEST_OFFSET + 100, 4).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = fixture.manifest_bytes();
    unsupported_schema[9] = 3;
    reseal_durable_frame(&mut unsupported_schema);
    assert_unsupported(
        &unsupported_schema,
        scope,
        PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = fixture.manifest_bytes();
    unsupported_format[10..12].copy_from_slice(&2_u16.to_le_bytes());
    reseal_durable_frame(&mut unsupported_format);
    assert_unsupported(
        &unsupported_format,
        scope,
        PhysicalIntegrityVersionAxis::PhysicalFormat,
        2,
    );
}

#[test]
fn manifest_record_extent_generation_length_and_geometry_substitution_are_distinct() {
    let fixture = ExtentFixture::new();
    let scope = fixture.manifest_scope();
    let cases = [
        (
            manifest_bytes(
                fixture,
                record(0x33, 8),
                fixture.extent,
                fixture.logical_bytes,
            ),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            48,
            24,
            PhysicalFormatField::RecordIdentity,
        ),
        (
            manifest_bytes(
                fixture,
                fixture.record,
                extent_cell(8, 5),
                fixture.logical_bytes,
            ),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            72,
            8,
            PhysicalFormatField::ExtentIdentity,
        ),
        (
            manifest_bytes(
                fixture,
                fixture.record,
                extent_cell(4, 6),
                fixture.logical_bytes,
            ),
            PhysicalDamageCause::PhysicalGenerationMismatch,
            28,
            8,
            PhysicalFormatField::PhysicalGeneration,
        ),
        (
            manifest_bytes(
                fixture,
                fixture.record,
                fixture.extent,
                fixture.logical_bytes - 1,
            ),
            PhysicalDamageCause::ChildReferenceMismatch,
            80,
            8,
            PhysicalFormatField::Payload,
        ),
    ];
    for (bytes, cause, offset, length, field) in cases {
        assert_manifest_damage(
            &bytes,
            scope,
            cause,
            field_range(scope, offset, length),
            Some(field),
            PhysicalBlastRadius::ReachableSubtree,
        );
    }

    let mut bad_chunk_count = fixture.manifest_bytes();
    bad_chunk_count[92..96].copy_from_slice(&1_u32.to_le_bytes());
    reseal_durable_frame(&mut bad_chunk_count);
    assert_manifest_damage(
        &bad_chunk_count,
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        field_range(scope, 92, 4),
        Some(PhysicalFormatField::Payload),
        PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn manifest_validation_record_binds_store_format_placement_and_range_scope() {
    let fixture = ExtentFixture::new();
    let bytes = fixture.manifest_bytes();
    let left_scope = fixture.manifest_scope();
    let right_scope = manifest_scope(store(8), fixture.format, fixture.placement(), 104);
    let left = intact(&bytes, left_scope).into_validation_record();
    let right = intact(&bytes, right_scope).into_validation_record();

    assert!(left.matches_scope(left_scope));
    assert!(!left.matches_scope(right_scope));
    assert_ne!(left.exact_scope_digest(), right.exact_scope_digest());
    assert_eq!(left.byte_range_digest(), right.byte_range_digest());

    let other_format = format(PhysicalPageSizeClass::KiB32);
    let other_manifest = DurableExtentManifest::new(
        other_format,
        fixture.record,
        fixture.extent,
        fixture.logical_bytes,
        other_format.page_size().bytes(),
        1,
    )
    .unwrap()
    .encode(other_format);
    assert_manifest_damage(
        &other_manifest,
        scope_with_length(fixture, other_manifest.len() as u64),
        PhysicalDamageCause::FormatMismatch,
        field_range(
            scope_with_length(fixture, other_manifest.len() as u64),
            10,
            10,
        ),
        Some(PhysicalFormatField::FormatDeclaration),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut header_only_substitution = fixture.manifest_bytes();
    header_only_substitution[10..20].copy_from_slice(&other_format.canonical_identity_bytes());
    reseal_durable_frame(&mut header_only_substitution);
    assert_manifest_damage(
        &header_only_substitution,
        left_scope,
        PhysicalDamageCause::FormatMismatch,
        field_range(left_scope, 10, 10),
        Some(PhysicalFormatField::FormatDeclaration),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

fn manifest_bytes(
    fixture: ExtentFixture,
    record: worth_store_physical_format::PersistedRecordIdentity,
    extent: worth_store_physical_format::RecordExtentGenerationCell,
    logical_bytes: u64,
) -> Vec<u8> {
    DurableExtentManifest::new(
        fixture.format,
        record,
        extent,
        logical_bytes,
        fixture.format.page_size().bytes(),
        2,
    )
    .unwrap()
    .encode(fixture.format)
}

fn scope_with_length(
    fixture: ExtentFixture,
    length: u64,
) -> worth_store_physical_integrity::PhysicalArtifactScope {
    manifest_scope(fixture.store, fixture.format, fixture.placement(), length)
}

fn intact<'a>(
    bytes: &'a [u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
) -> worth_store_physical_integrity::IntegrityValidatedExtentManifest<'a> {
    let (ExtentManifestIntegrityValidation::Intact(validated), _) =
        validate_extent_manifest(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("expected intact manifest")
    };
    validated
}

fn assert_manifest_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (validation, counters) =
        validate_extent_manifest(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let ExtentManifestIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged extent manifest unexpectedly validated")
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::ExtentManifest,
        bytes.len() as u64,
        cause,
    );
}

fn assert_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (
        ExtentManifestIntegrityValidation::Rejected(PhysicalIntegrityRejection::Unsupported(
            unsupported,
        )),
        counters,
    ) = validate_extent_manifest(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("expected unsupported extent manifest version")
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}
