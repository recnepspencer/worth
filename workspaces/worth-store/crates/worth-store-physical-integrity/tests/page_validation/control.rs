use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::{
    validate_inline_page, InlinePageIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalIntegrityValidationDigest,
    UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, clean_page, format, page, page_scope, reseal, store, PAGE_OFFSET, PAGE_SIZES,
};

#[test]
fn nonzero_data_page_lsn_remains_intact_and_is_carried_into_the_validated_view() {
    for page_size in PAGE_SIZES {
        let identity = page(31, 47, 11);
        let mut bytes = clean_page(page_size, identity);
        bytes[36..44].copy_from_slice(&0x0807_0605_0403_0201_u64.to_le_bytes());
        reseal(&mut bytes);
        let scope = page_scope(store(7), page_size, identity);
        let (InlinePageIntegrityValidation::Intact(validated), counters) =
            validate_inline_page(UntrustedPhysicalArtifact::from_bounded_bytes(&bytes), scope)
        else {
            panic!("a legitimate data-page LSN was treated as metadata reserved bytes");
        };
        assert_eq!(validated.page_lsn().get(), 0x0807_0605_0403_0201);
        assert_eq!(counters.intact_frames(), 1);
        assert_eq!(counters.rejected_frames(), 0);
    }
}

#[test]
fn all_declared_page_sizes_seal_geometry_scope_and_exact_incarnation() {
    for page_size in PAGE_SIZES {
        let page_identity = page(31, 47, 11);
        let bytes = clean_page(page_size, page_identity);
        let other_incarnation = bytes.clone();
        let scope = page_scope(store(7), page_size, page_identity);
        let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
        let (validation, counters) = validate_inline_page(artifact, scope);
        let InlinePageIntegrityValidation::Intact(validated) = validation else {
            panic!("clean inline page rejected at {page_size:?}");
        };

        assert_eq!(validated.scope(), scope);
        assert_eq!(validated.record_format(), format(page_size));
        assert_eq!(validated.page_identity(), page_identity);
        assert_eq!(validated.slot_count(), 2);
        assert_eq!(
            validated.free_bytes(),
            page_size.bytes() - 48 - 24 - 2 * 40 - 11
        );
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
            PhysicalIntegrityArtifactFamily::PageFrame
        );
        assert!(matches!(
            record.byte_range_digest(),
            PhysicalIntegrityValidationDigest::Crc32c(_)
        ));
        assert_eq!(
            counters.family(),
            PhysicalIntegrityArtifactFamily::PageFrame
        );
        assert_eq!(counters.inspected_frames(), 1);
        assert_eq!(counters.inspected_bytes(), u64::from(page_size.bytes()));
        assert_eq!(counters.intact_frames(), 1);
        assert_eq!(counters.rejected_frames(), 0);
    }
}

#[test]
fn validation_record_binds_store_page_generation_and_range_without_source_authority() {
    let page_size = worth_store_physical_format::PhysicalPageSizeClass::KiB16;
    let identity = page(5, 9, 13);
    let bytes = clean_page(page_size, identity);
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let left_scope = page_scope(store(7), page_size, identity);
    let right_scope = page_scope(store(8), page_size, identity);

    let (InlinePageIntegrityValidation::Intact(left), _) =
        validate_inline_page(artifact, left_scope)
    else {
        panic!("clean left-scoped page rejected");
    };
    let (InlinePageIntegrityValidation::Intact(right), _) =
        validate_inline_page(artifact, right_scope)
    else {
        panic!("clean right-scoped page rejected");
    };
    let left = left.into_validation_record();
    let right = right.into_validation_record();

    assert!(left.matches_scope(left_scope));
    assert!(!left.matches_scope(right_scope));
    assert!(right.matches_scope(right_scope));
    assert_ne!(left.exact_scope_digest(), right.exact_scope_digest());
    assert_eq!(left.byte_range_digest(), right.byte_range_digest());
}

#[test]
fn clean_page_under_root_scope_rejects_before_page_interpretation() {
    let page_size = worth_store_physical_format::PhysicalPageSizeClass::KiB16;
    let identity = page(5, 9, 13);
    let bytes = clean_page(page_size, identity);
    let wrong_scope = PhysicalArtifactScope::root_manifest(
        store(7),
        format(page_size),
        13,
        PhysicalByteRange::new(PAGE_OFFSET, u64::from(page_size.bytes())).unwrap(),
    )
    .unwrap();

    assert_damage(
        &bytes,
        wrong_scope,
        PhysicalDamageCause::FamilyMismatch,
        wrong_scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
}
