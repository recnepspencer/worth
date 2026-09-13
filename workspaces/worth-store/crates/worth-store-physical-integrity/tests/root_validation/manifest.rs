use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    maximum_current_root_entries, PersistedRecordIdentity, PhysicalPageSizeClass,
};
use worth_store_physical_integrity::{
    validate_root_manifest, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    PhysicalIntegrityVersionAxis, RootManifestIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, field_range, format, manifest_bytes,
    manifest_bytes_with_capacity, manifest_scope, populated_manifest_bytes, reseal_durable_frame,
    selector_scope, store, SelectorKind, MANIFEST_BYTES, MANIFEST_OFFSET,
};

#[test]
fn clean_manifest_control_seals_all_owner_handoff_projections() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);
    let bytes = populated_manifest_bytes(11, format);
    let other_incarnation = bytes.clone();
    let scope = manifest_scope(store, format, 11);
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_root_manifest(artifact, scope);
    let RootManifestIntegrityValidation::Intact(validated) = validation else {
        panic!("clean root manifest rejected");
    };

    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.root_generation(), 11);
    assert_eq!(validated.record_format(), format);
    assert_eq!(validated.tree_identity(), 71);
    assert_eq!(validated.node_capacity(), 2);
    assert_eq!(validated.record_count(), 1);
    assert_eq!(validated.next_block(), 2);
    assert_eq!(validated.next_segment_block(), 2);
    assert_eq!(validated.free_space_checksum(), 43);
    let record = PersistedRecordIdentity::new([0x41; 16], 1).unwrap();
    let routing_root = validated.routing_root().unwrap();
    assert_eq!(routing_root.generation(), 11);
    assert_eq!(routing_root.block(), 1);
    assert_eq!(routing_root.level(), 0);
    assert_eq!(routing_root.checksum(), 51);
    assert_eq!(routing_root.first(), record);
    assert_eq!(routing_root.last(), record);
    let segment_root = validated.segment_root().unwrap();
    assert_eq!(segment_root.generation(), 11);
    assert_eq!(segment_root.block(), 1);
    assert_eq!(segment_root.level(), 0);
    assert_eq!(segment_root.checksum(), 52);
    assert_eq!(validated.free_space_root().unwrap().generation(), 11);
    assert_eq!(validated.last_inline_record(), Some(record));
    let tail_segment = validated.last_inline_segment().unwrap();
    assert_eq!(tail_segment.segment_id().get(), 1);
    assert_eq!(tail_segment.generation().get(), 11);
    assert!(validated.matches_input(artifact));
    assert!(
        !validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(
            &other_incarnation,
        ))
    );
    assert!(validated.into_validation_record().matches_scope(scope));
    assert_intact_counters(counters, bytes.len() as u64);
}

#[test]
fn manifest_b_k_l_s_p_t_u_matrix_has_exact_localization_and_counters() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);
    let scope = manifest_scope(store, format, 11);

    let mut covered_byte_flip = manifest_bytes(11, format);
    covered_byte_flip[56] ^= 0x20;
    assert_manifest_damage(
        &covered_byte_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = manifest_bytes(11, format);
    checksum_flip[44] ^= 0x01;
    assert_manifest_damage(
        &checksum_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut length_lie = manifest_bytes(11, format);
    length_lie[24..28].copy_from_slice(&321_u32.to_le_bytes());
    reseal_durable_frame(&mut length_lie);
    assert_manifest_damage(
        &length_lie,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 20, 8),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let generation_substitution = manifest_bytes(12, format);
    assert_manifest_damage(
        &generation_substitution,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 8),
        Some(PhysicalFormatField::PhysicalGeneration),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let mut pointer_corruption = manifest_bytes(11, format);
    pointer_corruption[288..296].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut pointer_corruption);
    assert_manifest_damage(
        &pointer_corruption,
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        field_range(scope, 288, 8),
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let complete = manifest_bytes(11, format);
    let truncated = &complete[..complete.len() - 5];
    assert_manifest_damage(
        truncated,
        scope,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(MANIFEST_OFFSET + MANIFEST_BYTES - 5, 5).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = manifest_bytes(11, format);
    unsupported_schema[9] = 3;
    reseal_durable_frame(&mut unsupported_schema);
    assert_unsupported(
        &unsupported_schema,
        scope,
        PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = manifest_bytes(11, format);
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
fn manifest_family_identity_format_and_malformed_denials_remain_distinct() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);
    let scope = manifest_scope(store, format, 11);

    let mut family_substitution = manifest_bytes(11, format);
    family_substitution[8] = 11;
    reseal_durable_frame(&mut family_substitution);
    assert_manifest_damage(
        &family_substitution,
        scope,
        PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(PhysicalFormatField::ArtifactFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut identity_mismatch = manifest_bytes(11, format);
    identity_mismatch[48..56].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut identity_mismatch);
    assert_manifest_damage(
        &identity_mismatch,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 48, 8),
        Some(PhysicalFormatField::PhysicalGeneration),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let mut envelope_identity_mismatch = manifest_bytes(11, format);
    envelope_identity_mismatch[28..36].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut envelope_identity_mismatch);
    assert_manifest_damage(
        &envelope_identity_mismatch,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 8),
        Some(PhysicalFormatField::PhysicalGeneration),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let mut malformed = manifest_bytes(11, format);
    malformed[66] = 1;
    reseal_durable_frame(&mut malformed);
    assert_manifest_damage(
        &malformed,
        scope,
        PhysicalDamageCause::MalformedStructure,
        scope.byte_range(),
        Some(PhysicalFormatField::Payload),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let other_format = super::support::format(PhysicalPageSizeClass::KiB32);
    let larger_capacity = maximum_current_root_entries(format) + 1;
    let format_substitution = manifest_bytes_with_capacity(11, other_format, larger_capacity);
    assert_manifest_damage(
        &format_substitution,
        scope,
        PhysicalDamageCause::FormatMismatch,
        field_range(scope, 10, 10),
        Some(PhysicalFormatField::FormatDeclaration),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn manifest_validator_rejects_selector_scope_before_interpretation() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);
    let bytes = manifest_bytes(11, format);
    for kind in SelectorKind::ALL {
        let wrong_scope = selector_scope(kind, store, format);
        assert_manifest_damage(
            &bytes,
            wrong_scope,
            PhysicalDamageCause::FamilyMismatch,
            wrong_scope.byte_range(),
            None,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn manifest_validation_record_binds_store_scope_without_inventing_embedded_store_proof() {
    let format = format(PhysicalPageSizeClass::KiB16);
    let bytes = manifest_bytes(11, format);
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let left_scope = manifest_scope(store(7), format, 11);
    let right_scope = manifest_scope(store(8), format, 11);

    let (RootManifestIntegrityValidation::Intact(left), _) =
        validate_root_manifest(artifact, left_scope)
    else {
        panic!("clean left scope rejected");
    };
    let (RootManifestIntegrityValidation::Intact(right), _) =
        validate_root_manifest(artifact, right_scope)
    else {
        panic!("clean right scope rejected");
    };
    let left_record = left.into_validation_record();
    let right_record = right.into_validation_record();

    assert!(left_record.matches_scope(left_scope));
    assert!(!left_record.matches_scope(right_scope));
    assert!(right_record.matches_scope(right_scope));
    assert_ne!(
        left_record.exact_scope_digest(),
        right_record.exact_scope_digest()
    );
    assert_eq!(
        left_record.byte_range_digest(),
        right_record.byte_range_digest()
    );
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
        validate_root_manifest(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let RootManifestIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged root manifest unexpectedly validated");
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::RootManifest,
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
    let (validation, counters) =
        validate_root_manifest(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let RootManifestIntegrityValidation::Rejected(PhysicalIntegrityRejection::Unsupported(
        unsupported,
    )) = validation
    else {
        panic!("expected unsupported manifest version");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(
        counters.family(),
        PhysicalIntegrityArtifactFamily::RootManifest
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}

fn assert_intact_counters(
    counters: worth_store_physical_integrity::PhysicalIntegrityObservationCounters,
    byte_count: u64,
) {
    assert_eq!(
        counters.family(),
        PhysicalIntegrityArtifactFamily::RootManifest
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}
