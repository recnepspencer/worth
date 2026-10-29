#[allow(unused_imports)] use crate::root_tree_family_validation::bootstrap_catalog::*;
use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration};
use {worth_store_physical_integrity::validate_bootstrap_catalog, worth_store_physical_integrity::BootstrapCatalogIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityValidationDigest, worth_store_physical_integrity::PhysicalIntegrityValidationMechanism, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::literal_vectors::root_tree_family_validation::BOOTSTRAP_CATALOG_COMPLETE_CRC32C, crate::literal_vectors::root_tree_family_validation::BOOTSTRAP_CATALOG_LITERAL};
use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::bootstrap_bytes, crate::support::root_tree_family_validation::bootstrap_scope, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::format, crate::support::root_tree_family_validation::independent_crc32c, crate::support::root_tree_family_validation::reseal_durable_frame, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::BOOTSTRAP_OFFSET};

#[test]
fn catalog_literal_vector_is_independent_of_writer_and_validates_directly() {
    assert_eq!(bootstrap_bytes(store(7)), BOOTSTRAP_CATALOG_LITERAL);
    assert_eq!(
        independent_crc32c(&[&BOOTSTRAP_CATALOG_LITERAL]),
        BOOTSTRAP_CATALOG_COMPLETE_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(BOOTSTRAP_CATALOG_LITERAL[44..48].try_into().unwrap()),
        independent_crc32c(&[
            &BOOTSTRAP_CATALOG_LITERAL[..44],
            &BOOTSTRAP_CATALOG_LITERAL[48..],
        ])
    );
    match validate_bootstrap_catalog(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&BOOTSTRAP_CATALOG_LITERAL),
        bootstrap_scope(store(7), BOOTSTRAP_OFFSET),
    )
    .0
    {
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Intact(validated) => {
            assert_eq!(validated.current_root_generation().get(), 11);
            let record = validated.into_validation_record();
            assert_eq!(
                record.byte_range_digest(),
                worth_store_physical_integrity::PhysicalIntegrityValidationDigest::crc32c(BOOTSTRAP_CATALOG_COMPLETE_CRC32C)
            );
            assert_eq!(
                record.mechanism(),
                worth_store_physical_integrity::PhysicalIntegrityValidationMechanism::Crc32cV1
            );
            assert_eq!(record.mechanism().version(), 1);
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Rejected(rejection) => {
            panic!("literal bootstrap catalog rejected: {rejection:?}")
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch) => {
            panic!("literal bootstrap catalog mismatched scope: {mismatch:?}")
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported) => {
            panic!("literal bootstrap catalog has unsupported format: {unsupported:?}")
        }
    }
}

#[test]
fn clean_catalog_seals_generation_record_range_and_incarnation() {
    let store = store(7);
    let bytes = bootstrap_bytes(store);
    let other_incarnation = bytes.clone();
    let scope = bootstrap_scope(store, BOOTSTRAP_OFFSET);
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validated, counters) = match validate_bootstrap_catalog(artifact, scope) {
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Intact(validated), counters) => (validated, counters),
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Rejected(rejection), _) => {
            panic!("clean bootstrap catalog rejected: {rejection:?}")
        }
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch), _) => {
            panic!("clean bootstrap catalog mismatched scope: {mismatch:?}")
        }
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported), _) => {
            panic!("clean bootstrap catalog has unsupported format: {unsupported:?}")
        }
    };
    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.record_format(), format());
    assert_eq!(validated.current_root_generation().get(), 11);
    assert!(validated.matches_input(artifact));
    assert!(
        !validated.matches_input(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &other_incarnation
        ))
    );
    let record = validated.into_validation_record();
    assert!(record.matches_scope(scope));
    assert_eq!(
        record.artifact_family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::BootstrapCatalog
    );
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);

    let alternate_scope = bootstrap_scope(store, BOOTSTRAP_OFFSET + 512);
    let alternate = match validate_bootstrap_catalog(artifact, alternate_scope).0 {
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Intact(validated) => {
            validated.into_validation_record()
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Rejected(rejection) => {
            panic!("range-bound clean catalog rejected: {rejection:?}")
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch) => {
            panic!("range-bound clean catalog mismatched scope: {mismatch:?}")
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported) => {
            panic!("range-bound clean catalog has unsupported format: {unsupported:?}")
        }
    };
    assert_ne!(record.exact_scope_digest(), alternate.exact_scope_digest());
}

#[test]
fn catalog_framing_crc_kind_length_store_generation_and_truncation_localize_exactly() {
    let store = store(7);
    let scope = bootstrap_scope(store, BOOTSTRAP_OFFSET);

    let mut covered_flip = bootstrap_bytes(store);
    covered_flip[48] ^= 0x40;
    assert_catalog_damage(
        &covered_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = bootstrap_bytes(store);
    checksum_flip[44] ^= 1;
    assert_catalog_damage(
        &checksum_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_kind = bootstrap_bytes(store);
    wrong_kind[8] = 8;
    reseal_durable_frame(&mut wrong_kind);
    assert_catalog_damage(
        &wrong_kind,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = bootstrap_bytes(store);
    length_lie[24..28].copy_from_slice(&0_u32.to_le_bytes());
    reseal_durable_frame(&mut length_lie);
    assert_catalog_damage(
        &length_lie,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 20, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let substituted_store = bootstrap_bytes(crate::support::root_tree_family_validation::store(8));
    assert_catalog_damage(
        &substituted_store,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 48, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::StoreIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut generation_lie = bootstrap_bytes(store);
    generation_lie[64..72].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut generation_lie);
    assert_catalog_damage(
        &generation_lie,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 44),
        Some(worth_store_physical_integrity::PhysicalFormatField::RootGeneration),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let complete = bootstrap_bytes(store);
    assert_catalog_damage(
        &complete[..79],
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(BOOTSTRAP_OFFSET + 79, 3).unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn catalog_format_scope_and_version_windows_do_not_collapse_into_damage() {
    let store = store(7);
    let bytes = bootstrap_bytes(store);
    let other_format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .page_size(worth_store_physical_format::PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let other_scope = worth_store_physical_integrity::PhysicalArtifactScope::bootstrap_catalog(
        store,
        other_format,
        worth_store_physical_integrity::PhysicalByteRange::new(BOOTSTRAP_OFFSET, bytes.len() as u64).unwrap(),
    );
    assert_catalog_damage(
        &bytes,
        other_scope,
        worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
        field_range(other_scope, 10, 10),
        Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    for (offset, axis) in [
        (9, worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema),
        (10, worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat),
        (72, worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat),
    ] {
        let mut unsupported = bootstrap_bytes(store);
        if offset == 9 {
            unsupported[offset] = 3;
        } else {
            unsupported[offset..offset + 2].copy_from_slice(&3_u16.to_le_bytes());
        }
        reseal_durable_frame(&mut unsupported);
        let rejection =
            validate_catalog_rejection(&unsupported, bootstrap_scope(store, BOOTSTRAP_OFFSET)).0;
        match rejection {
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(posture) => {
                assert_eq!(posture.axis(), axis);
                assert_eq!(posture.observed(), 3);
            }
            other => panic!("unsupported catalog version collapsed: {other:?}"),
        }
    }
}
