#[allow(unused_imports)] use crate::root_validation::manifest::*;
use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_format::maximum_current_root_entries, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalPageSizeClass};
use {worth_store_physical_integrity::validate_root_manifest, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::RootManifestIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::root_validation::assert_damage, crate::support::root_validation::assert_rejected_counters, crate::support::root_validation::field_range, crate::support::root_validation::format, crate::support::root_validation::manifest_bytes, crate::support::root_validation::manifest_bytes_with_capacity, crate::support::root_validation::manifest_scope, crate::support::root_validation::populated_manifest_bytes, crate::support::root_validation::reseal_durable_frame, crate::support::root_validation::selector_scope, crate::support::root_validation::store, crate::support::root_validation::SelectorKind, crate::support::root_validation::MANIFEST_BYTES, crate::support::root_validation::MANIFEST_OFFSET};

#[test]
fn clean_manifest_control_seals_all_owner_handoff_projections() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let bytes = populated_manifest_bytes(11, format);
    let other_incarnation = bytes.clone();
    let scope = manifest_scope(store, format, 11);
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_root_manifest(artifact, scope);
    let worth_store_physical_integrity::RootManifestIntegrityValidation::Intact(validated) = validation else {
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
    let record = worth_store_physical_format::PersistedRecordIdentity::new([0x41; 16], 1).unwrap();
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
        !validated.matches_input(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &other_incarnation,
        ))
    );
    assert!(validated.into_validation_record().matches_scope(scope));
    assert_intact_counters(counters, bytes.len() as u64);
}

#[test]
fn manifest_b_k_l_s_p_t_u_matrix_has_exact_localization_and_counters() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let scope = manifest_scope(store, format, 11);

    let mut covered_byte_flip = manifest_bytes(11, format);
    covered_byte_flip[56] ^= 0x20;
    assert_manifest_damage(
        &covered_byte_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = manifest_bytes(11, format);
    checksum_flip[44] ^= 0x01;
    assert_manifest_damage(
        &checksum_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut length_lie = manifest_bytes(11, format);
    length_lie[24..28].copy_from_slice(&321_u32.to_le_bytes());
    reseal_durable_frame(&mut length_lie);
    assert_manifest_damage(
        &length_lie,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 20, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let generation_substitution = manifest_bytes(12, format);
    assert_manifest_damage(
        &generation_substitution,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut pointer_corruption = manifest_bytes(11, format);
    pointer_corruption[288..296].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut pointer_corruption);
    assert_manifest_damage(
        &pointer_corruption,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        field_range(scope, 288, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::ChildReference),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let complete = manifest_bytes(11, format);
    let truncated = &complete[..complete.len() - 5];
    assert_manifest_damage(
        truncated,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(MANIFEST_OFFSET + MANIFEST_BYTES - 5, 5).unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = manifest_bytes(11, format);
    // Schemas 3 through 7 are admitted; 8 is the first unknown schema.
    unsupported_schema[9] = 8;
    reseal_durable_frame(&mut unsupported_schema);
    assert_unsupported(
        &unsupported_schema,
        scope,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema,
        8,
    );

    let mut unsupported_format = manifest_bytes(11, format);
    unsupported_format[10..12].copy_from_slice(&3_u16.to_le_bytes());
    reseal_durable_frame(&mut unsupported_format);
    assert_unsupported(
        &unsupported_format,
        scope,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat,
        3,
    );
}

#[test]
fn manifest_family_identity_format_and_malformed_denials_remain_distinct() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let scope = manifest_scope(store, format, 11);

    let mut family_substitution = manifest_bytes(11, format);
    family_substitution[8] = 11;
    reseal_durable_frame(&mut family_substitution);
    assert_manifest_damage(
        &family_substitution,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut identity_mismatch = manifest_bytes(11, format);
    identity_mismatch[48..56].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut identity_mismatch);
    assert_manifest_damage(
        &identity_mismatch,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 48, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut envelope_identity_mismatch = manifest_bytes(11, format);
    envelope_identity_mismatch[28..36].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut envelope_identity_mismatch);
    assert_manifest_damage(
        &envelope_identity_mismatch,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut malformed = manifest_bytes(11, format);
    malformed[66] = 1;
    reseal_durable_frame(&mut malformed);
    assert_manifest_damage(
        &malformed,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        scope.byte_range(),
        Some(worth_store_physical_integrity::PhysicalFormatField::Payload),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let other_format = crate::support::root_validation::format(worth_store_physical_format::PhysicalPageSizeClass::KiB32);
    let larger_capacity = maximum_current_root_entries(format) + 1;
    let format_substitution = manifest_bytes_with_capacity(11, other_format, larger_capacity);
    assert_manifest_damage(
        &format_substitution,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
        field_range(scope, 10, 10),
        Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn manifest_validator_rejects_selector_scope_before_interpretation() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let bytes = manifest_bytes(11, format);
    for kind in crate::support::root_validation::SelectorKind::ALL {
        let wrong_scope = selector_scope(kind, store, format);
        assert_manifest_damage(
            &bytes,
            wrong_scope,
            worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
            wrong_scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn manifest_validation_record_binds_store_scope_without_inventing_embedded_store_proof() {
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let bytes = manifest_bytes(11, format);
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let left_scope = manifest_scope(store(7), format, 11);
    let right_scope = manifest_scope(store(8), format, 11);

    let (worth_store_physical_integrity::RootManifestIntegrityValidation::Intact(left), _) =
        validate_root_manifest(artifact, left_scope)
    else {
        panic!("clean left scope rejected");
    };
    let (worth_store_physical_integrity::RootManifestIntegrityValidation::Intact(right), _) =
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
