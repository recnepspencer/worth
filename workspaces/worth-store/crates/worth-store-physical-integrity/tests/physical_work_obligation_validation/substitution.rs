use worth_store_physical_format::PhysicalRecordFormatDeclaration;
use worth_store_physical_integrity::{
    validate_physical_work_obligation, PhysicalArtifactScope, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalWorkObligationIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::support::{assert_damage, field_range, identity, reseal, scope, store};
use super::vectors::{operation_3, STORE_BYTES};

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
        PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 16, 16),
        Some(PhysicalFormatField::StoreIdentity),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let filename_substitution = operation_3();
    let other_filename_scope = super::support::scope(store, identity(1, 2, 4));
    assert_damage(
        &filename_substitution,
        other_filename_scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        field_range(other_filename_scope, 48, 8),
        Some(PhysicalFormatField::OperationIdentity),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn each_filename_identity_component_localizes_independently() {
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let cases = [
        (
            32,
            9_u64,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PhysicalFormatField::RuntimeIdentity,
        ),
        (
            40,
            9_u64,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            PhysicalFormatField::PhysicalGeneration,
        ),
        (
            48,
            9_u64,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PhysicalFormatField::OperationIdentity,
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
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn family_and_exact_range_substitution_are_rejected_before_projection() {
    let store = store(STORE_BYTES);
    let bytes = operation_3();
    let wrong_family = PhysicalArtifactScope::current_root_selector(
        store,
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        PhysicalByteRange::new(8_192, 160).unwrap(),
    );
    assert_damage(
        &bytes,
        wrong_family,
        PhysicalDamageCause::FamilyMismatch,
        wrong_family.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );

    let wrong_range = PhysicalArtifactScope::physical_work_obligation(
        store,
        identity(1, 2, 3),
        PhysicalByteRange::new(8_192, 159).unwrap(),
    );
    assert_damage(
        &bytes,
        wrong_range,
        PhysicalDamageCause::FramingLengthMismatch,
        wrong_range.byte_range(),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn sealed_view_cannot_be_reused_for_an_equal_but_distinct_byte_incarnation() {
    let bytes = operation_3();
    let other = bytes;
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_physical_work_obligation(artifact, scope);
    let PhysicalWorkObligationIntegrityValidation::Intact(validated) = validation else {
        panic!("clean literal rejected")
    };
    assert!(validated.matches_input(artifact));
    assert!(!validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(&other)));
}
