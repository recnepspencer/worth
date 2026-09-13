use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{PhysicalPageSizeClass, PhysicalRecordFormatDeclaration};
use worth_store_physical_integrity::{
    validate_bootstrap_catalog, BootstrapCatalogIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejection, PhysicalIntegrityValidationDigest,
    PhysicalIntegrityValidationMechanism, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

use super::literal_vectors::{BOOTSTRAP_CATALOG_COMPLETE_CRC32C, BOOTSTRAP_CATALOG_LITERAL};
use super::support::{
    assert_damage, assert_rejected_counters, bootstrap_bytes, bootstrap_scope, field_range, format,
    independent_crc32c, reseal_durable_frame, store, BOOTSTRAP_OFFSET,
};

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
        UntrustedPhysicalArtifact::from_bounded_bytes(&BOOTSTRAP_CATALOG_LITERAL),
        bootstrap_scope(store(7), BOOTSTRAP_OFFSET),
    )
    .0
    {
        BootstrapCatalogIntegrityValidation::Intact(validated) => {
            assert_eq!(validated.current_root_generation().get(), 11);
            let record = validated.into_validation_record();
            assert_eq!(
                record.byte_range_digest(),
                PhysicalIntegrityValidationDigest::crc32c(BOOTSTRAP_CATALOG_COMPLETE_CRC32C)
            );
            assert_eq!(
                record.mechanism(),
                PhysicalIntegrityValidationMechanism::Crc32cV1
            );
            assert_eq!(record.mechanism().version(), 1);
        }
        BootstrapCatalogIntegrityValidation::Rejected(rejection) => {
            panic!("literal bootstrap catalog rejected: {rejection:?}")
        }
        BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch) => {
            panic!("literal bootstrap catalog mismatched scope: {mismatch:?}")
        }
        BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported) => {
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
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validated, counters) = match validate_bootstrap_catalog(artifact, scope) {
        (BootstrapCatalogIntegrityValidation::Intact(validated), counters) => (validated, counters),
        (BootstrapCatalogIntegrityValidation::Rejected(rejection), _) => {
            panic!("clean bootstrap catalog rejected: {rejection:?}")
        }
        (BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch), _) => {
            panic!("clean bootstrap catalog mismatched scope: {mismatch:?}")
        }
        (BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported), _) => {
            panic!("clean bootstrap catalog has unsupported format: {unsupported:?}")
        }
    };
    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.record_format(), format());
    assert_eq!(validated.current_root_generation().get(), 11);
    assert!(validated.matches_input(artifact));
    assert!(
        !validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(
            &other_incarnation
        ))
    );
    let record = validated.into_validation_record();
    assert!(record.matches_scope(scope));
    assert_eq!(
        record.artifact_family(),
        PhysicalIntegrityArtifactFamily::BootstrapCatalog
    );
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);

    let alternate_scope = bootstrap_scope(store, BOOTSTRAP_OFFSET + 512);
    let alternate = match validate_bootstrap_catalog(artifact, alternate_scope).0 {
        BootstrapCatalogIntegrityValidation::Intact(validated) => {
            validated.into_validation_record()
        }
        BootstrapCatalogIntegrityValidation::Rejected(rejection) => {
            panic!("range-bound clean catalog rejected: {rejection:?}")
        }
        BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch) => {
            panic!("range-bound clean catalog mismatched scope: {mismatch:?}")
        }
        BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported) => {
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
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = bootstrap_bytes(store);
    checksum_flip[44] ^= 1;
    assert_catalog_damage(
        &checksum_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_kind = bootstrap_bytes(store);
    wrong_kind[8] = 8;
    reseal_durable_frame(&mut wrong_kind);
    assert_catalog_damage(
        &wrong_kind,
        scope,
        PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(PhysicalFormatField::ArtifactFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = bootstrap_bytes(store);
    length_lie[24..28].copy_from_slice(&0_u32.to_le_bytes());
    reseal_durable_frame(&mut length_lie);
    assert_catalog_damage(
        &length_lie,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 20, 8),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let substituted_store = bootstrap_bytes(super::support::store(8));
    assert_catalog_damage(
        &substituted_store,
        scope,
        PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 48, 16),
        Some(PhysicalFormatField::StoreIdentity),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut generation_lie = bootstrap_bytes(store);
    generation_lie[64..72].copy_from_slice(&12_u64.to_le_bytes());
    reseal_durable_frame(&mut generation_lie);
    assert_catalog_damage(
        &generation_lie,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 44),
        Some(PhysicalFormatField::RootGeneration),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let complete = bootstrap_bytes(store);
    assert_catalog_damage(
        &complete[..79],
        scope,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(BOOTSTRAP_OFFSET + 79, 3).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn catalog_format_scope_and_version_windows_do_not_collapse_into_damage() {
    let store = store(7);
    let bytes = bootstrap_bytes(store);
    let other_format = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let other_scope = PhysicalArtifactScope::bootstrap_catalog(
        store,
        other_format,
        PhysicalByteRange::new(BOOTSTRAP_OFFSET, bytes.len() as u64).unwrap(),
    );
    assert_catalog_damage(
        &bytes,
        other_scope,
        PhysicalDamageCause::FormatMismatch,
        field_range(other_scope, 10, 10),
        Some(PhysicalFormatField::FormatDeclaration),
        PhysicalBlastRadius::CompleteArtifact,
    );

    for (offset, axis) in [
        (9, PhysicalIntegrityVersionAxis::EnvelopeSchema),
        (10, PhysicalIntegrityVersionAxis::PhysicalFormat),
        (72, PhysicalIntegrityVersionAxis::PhysicalFormat),
    ] {
        let mut unsupported = bootstrap_bytes(store);
        if offset == 9 {
            unsupported[offset] = 3;
        } else {
            unsupported[offset..offset + 2].copy_from_slice(&2_u16.to_le_bytes());
        }
        reseal_durable_frame(&mut unsupported);
        let rejection =
            validate_catalog_rejection(&unsupported, bootstrap_scope(store, BOOTSTRAP_OFFSET)).0;
        match rejection {
            PhysicalIntegrityRejection::Unsupported(posture) => {
                assert_eq!(posture.axis(), axis);
                assert_eq!(posture.observed(), if offset == 9 { 3 } else { 2 });
            }
            other => panic!("unsupported catalog version collapsed: {other:?}"),
        }
    }
}

fn assert_catalog_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = validate_catalog_rejection(bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::BootstrapCatalog,
        bytes.len() as u64,
        cause,
    );
}

fn validate_catalog_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    worth_store_physical_integrity::PhysicalIntegrityObservationCounters,
) {
    match validate_bootstrap_catalog(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope) {
        (BootstrapCatalogIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        (BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch), counters) => {
            (mismatch.rejection(), counters)
        }
        (BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported), counters) => {
            (unsupported.rejection(), counters)
        }
        (BootstrapCatalogIntegrityValidation::Intact(_), _) => {
            panic!("damaged bootstrap catalog validated")
        }
    }
}
