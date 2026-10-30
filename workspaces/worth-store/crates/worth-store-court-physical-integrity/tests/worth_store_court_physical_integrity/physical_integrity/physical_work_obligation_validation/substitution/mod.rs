use worth_store_physical_integrity::validate_physical_work_obligation;

use crate::support::physical_work_obligation_validation::{
    assert_damage, field_range, identity, reseal, scope, store,
};
use crate::vectors::{operation_3, STORE_BYTES};

#[test]
fn checksum_valid_store_and_filename_identity_substitution_fail_exact_scope() {
    let store = store(STORE_BYTES);
    let scope = scope(store, identity(1, 2, 3));

    let mut other_store = operation_3();
    other_store[16..32].copy_from_slice(&[0x44; 16]);
    reseal(&mut other_store);
    assert_damage(
        &other_store,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 16, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::StoreIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let filename_substitution = operation_3();
    let other_filename_scope =
        crate::support::physical_work_obligation_validation::scope(store, identity(1, 2, 4));
    assert_damage(
        &filename_substitution,
        other_filename_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        field_range(other_filename_scope, 48, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::OperationIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn each_filename_identity_component_localizes_independently() {
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let cases = [
        (
            32,
            9_u64,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            worth_store_physical_integrity::PhysicalFormatField::RuntimeIdentity,
        ),
        (
            40,
            9_u64,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
        (
            48,
            9_u64,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            worth_store_physical_integrity::PhysicalFormatField::OperationIdentity,
        ),
    ];
    for (offset, value, cause, field) in cases {
        let mut bytes = operation_3();
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        reseal(&mut bytes);
        assert_damage(
            &bytes,
            scope,
            cause,
            field_range(scope, offset as u64, 8),
            Some(field),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn family_and_exact_range_substitution_are_rejected_before_projection() {
    let store = store(STORE_BYTES);
    let bytes = operation_3();
    let wrong_family = worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(
        store,
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
        worth_store_physical_integrity::PhysicalByteRange::new(8_192, 160).unwrap(),
    );
    assert_damage(
        &bytes,
        wrong_family,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        wrong_family.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let wrong_range =
        worth_store_physical_integrity::PhysicalArtifactScope::physical_work_obligation(
            store,
            identity(1, 2, 3),
            worth_store_physical_integrity::PhysicalByteRange::new(8_192, 159).unwrap(),
        );
    assert_damage(
        &bytes,
        wrong_range,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        wrong_range.byte_range(),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn sealed_view_cannot_be_reused_for_an_equal_but_distinct_byte_incarnation() {
    let bytes = operation_3();
    let other = bytes;
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let artifact =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_physical_work_obligation(artifact, scope);
    let worth_store_physical_integrity::PhysicalWorkObligationIntegrityValidation::Intact(
        validated,
    ) = validation
    else {
        panic!("clean literal rejected")
    };
    assert!(validated.matches_input(artifact));
    assert!(!validated.matches_input(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&other)
    ));
}
