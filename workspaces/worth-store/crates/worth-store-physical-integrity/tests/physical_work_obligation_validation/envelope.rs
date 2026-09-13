use worth_store_physical_format::integrity_declarations::families::{
    PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES, PHYSICAL_WORK_OBLIGATION_V6_VERSION,
};
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::physical_work_obligation::{
    PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
};
use worth_store_physical_integrity::{
    validate_physical_work_obligation, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, PhysicalIntegrityVersionAxis,
    PhysicalWorkObligationIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, field_range, identity, independent_sha256, rejection, reseal, scope, store,
    OBLIGATION_OFFSET,
};
use super::vectors::{
    operation_3, operation_4, OPERATION_3_SCOPE_SHA, OPERATION_3_SHA, OPERATION_4_SHA, STORE_BYTES,
};

#[test]
fn frozen_literals_seal_family_specific_views_and_validation_records() {
    let store = store(STORE_BYTES);
    let cases = [
        (operation_3(), 3, OPERATION_3_SHA),
        (operation_4(), 4, OPERATION_4_SHA),
    ];
    for (bytes, operation, covered_digest) in cases {
        assert_eq!(bytes.len(), PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES);
        assert_eq!(bytes[8], PHYSICAL_WORK_OBLIGATION_V6_VERSION);
        assert_eq!(independent_sha256(&bytes[..128]), covered_digest);
        assert_eq!(&bytes[128..160], covered_digest.as_slice());

        let scope = scope(store, identity(1, 2, operation));
        let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
        let other_incarnation = bytes;
        let (validation, counters) = validate_physical_work_obligation(artifact, scope);
        let PhysicalWorkObligationIntegrityValidation::Intact(validated) = validation else {
            panic!("frozen physical-work literal was rejected")
        };
        assert_eq!(validated.scope(), scope);
        assert_eq!(validated.identity(), identity(1, 2, operation));
        assert!(validated.matches_input(artifact));
        assert!(
            !validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(
                &other_incarnation,
            ))
        );
        match operation {
            3 => {
                assert_eq!(
                    validated.operation_code(),
                    PhysicalWorkObligationOperationCode::DurabilityBarrier
                );
                assert_eq!(
                    validated.target(),
                    PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization
                );
                assert_eq!(validated.payload_digest(), None);
            }
            4 => {
                assert_eq!(
                    validated.operation_code(),
                    PhysicalWorkObligationOperationCode::WalAppend
                );
                assert_eq!(
                    validated.target(),
                    PhysicalWorkObligationTargetCode::WalArtifactInterval {
                        segment: 7,
                        generation: 8,
                        offset: 9,
                        byte_count: 10,
                    }
                );
                assert_eq!(validated.payload_digest(), Some([0xab; 32]));
            }
            _ => unreachable!(),
        }
        let record = validated.into_validation_record();
        assert!(record.matches_scope(scope));
        assert_eq!(
            record.mechanism(),
            PhysicalIntegrityValidationMechanism::Sha256V1
        );
        assert_eq!(
            record.byte_range_digest(),
            PhysicalIntegrityValidationDigest::sha256(independent_sha256(&bytes))
        );
        assert_eq!(
            counters.family(),
            PhysicalIntegrityArtifactFamily::PhysicalWorkObligation
        );
        assert_eq!(counters.inspected_frames(), 1);
        assert_eq!(counters.inspected_bytes(), 160);
        assert_eq!(counters.intact_frames(), 1);
        assert_eq!(counters.rejected_frames(), 0);
    }
}

#[test]
fn validation_record_scope_digest_is_literal_and_sensitive_to_every_variable_axis() {
    let baseline_bytes = operation_3();
    let baseline_scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let baseline = validation_record(&baseline_bytes, baseline_scope);
    assert_eq!(
        baseline.exact_scope_digest(),
        PhysicalIntegrityValidationDigest::sha256(OPERATION_3_SCOPE_SHA)
    );

    let mut other_store_bytes = baseline_bytes;
    other_store_bytes[16..32].copy_from_slice(&[0x44; 16]);
    reseal(&mut other_store_bytes);
    let other_store = validation_record(
        &other_store_bytes,
        scope(store([0x44; 16]), identity(1, 2, 3)),
    );

    let mut other_runtime_bytes = baseline_bytes;
    other_runtime_bytes[32..40].copy_from_slice(&9_u64.to_le_bytes());
    reseal(&mut other_runtime_bytes);
    let other_runtime = validation_record(
        &other_runtime_bytes,
        scope(store(STORE_BYTES), identity(9, 2, 3)),
    );

    let mut other_generation_bytes = baseline_bytes;
    other_generation_bytes[40..48].copy_from_slice(&9_u64.to_le_bytes());
    reseal(&mut other_generation_bytes);
    let other_generation = validation_record(
        &other_generation_bytes,
        scope(store(STORE_BYTES), identity(1, 9, 3)),
    );

    let mut other_operation_bytes = baseline_bytes;
    other_operation_bytes[48..56].copy_from_slice(&9_u64.to_le_bytes());
    reseal(&mut other_operation_bytes);
    let other_operation = validation_record(
        &other_operation_bytes,
        scope(store(STORE_BYTES), identity(1, 2, 9)),
    );

    let other_range_scope =
        worth_store_physical_integrity::PhysicalArtifactScope::physical_work_obligation(
            store(STORE_BYTES),
            identity(1, 2, 3),
            PhysicalByteRange::new(OBLIGATION_OFFSET + 1, 160).unwrap(),
        );
    let other_range = validation_record(&baseline_bytes, other_range_scope);

    for changed in [
        other_store,
        other_runtime,
        other_generation,
        other_operation,
        other_range,
    ] {
        assert_ne!(baseline.exact_scope_digest(), changed.exact_scope_digest());
    }
}

#[test]
fn covered_byte_and_excluded_digest_damage_preserve_honest_checksum_localization() {
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let mut covered = operation_3();
    covered[56] ^= 1;
    assert_damage(
        &covered,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut excluded_digest = operation_3();
    let covered_sha = independent_sha256(&excluded_digest[..128]);
    excluded_digest[128] ^= 1;
    assert_eq!(independent_sha256(&excluded_digest[..128]), covered_sha);
    assert_damage(
        &excluded_digest,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn fixed_record_bounds_distinguish_truncation_from_oversize_input() {
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let bytes = operation_3();
    assert_damage(
        &bytes[..159],
        scope,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(OBLIGATION_OFFSET + 159, 1).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );
    let mut oversized = bytes.to_vec();
    oversized.push(0);
    assert_damage(
        &oversized,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        scope.byte_range(),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn magic_kind_structure_target_shape_and_version_have_typed_localization() {
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));

    let mut wrong_magic = operation_3();
    wrong_magic[0] ^= 1;
    reseal(&mut wrong_magic);
    assert_damage(
        &wrong_magic,
        scope,
        PhysicalDamageCause::WrongMagic,
        field_range(scope, 0, 8),
        Some(PhysicalFormatField::Magic),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut unknown_kind = operation_3();
    unknown_kind[9] = 0xff;
    reseal(&mut unknown_kind);
    assert_damage(
        &unknown_kind,
        scope,
        PhysicalDamageCause::RecordKindMismatch,
        field_range(scope, 9, 1),
        Some(PhysicalFormatField::OperationFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut header_reserved = operation_3();
    header_reserved[10] = 1;
    reseal(&mut header_reserved);
    assert_damage(
        &header_reserved,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 10, 6),
        Some(PhysicalFormatField::Reserved),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut reserved = operation_3();
    reserved[107] = 1;
    reseal(&mut reserved);
    assert_damage(
        &reserved,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 107, 5),
        Some(PhysicalFormatField::Reserved),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut target_shape = operation_3();
    target_shape[104] = 0;
    reseal(&mut target_shape);
    assert_damage(
        &target_shape,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 56, 72),
        Some(PhysicalFormatField::TargetShape),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut digest_presence = operation_3();
    digest_presence[105] = 2;
    reseal(&mut digest_presence);
    assert_damage(
        &digest_presence,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 105, 1),
        Some(PhysicalFormatField::PayloadDigestPresence),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut unsupported = operation_3();
    unsupported[8] = 7;
    reseal(&mut unsupported);
    let (rejection, counters) = rejection(&unsupported, scope);
    let PhysicalIntegrityRejection::Unsupported(version) = rejection else {
        panic!("expected unsupported physical-work version, got {rejection:?}")
    };
    assert_eq!(version.scope(), scope);
    assert_eq!(
        version.axis(),
        PhysicalIntegrityVersionAxis::PhysicalWorkObligation
    );
    assert_eq!(version.observed(), 7);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}

fn validation_record(
    bytes: &[u8; 160],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
) -> PhysicalIntegrityValidationRecord {
    let (validation, _) = validate_physical_work_obligation(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    );
    let PhysicalWorkObligationIntegrityValidation::Intact(validated) = validation else {
        panic!("scope-digest fixture was rejected")
    };
    validated.into_validation_record()
}
