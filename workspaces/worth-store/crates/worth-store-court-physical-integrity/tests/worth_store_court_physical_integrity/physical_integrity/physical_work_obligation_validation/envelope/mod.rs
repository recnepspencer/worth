#[allow(unused_imports)]
use crate::physical_work_obligation_validation::envelope::*;

#[test]
fn validation_record_scope_digest_is_literal_and_sensitive_to_every_variable_axis() {
    let baseline_bytes = operation_3();
    let baseline_scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let baseline = validation_record(&baseline_bytes, baseline_scope);
    assert_eq!(
        baseline.exact_scope_digest(),
        worth_store_physical_integrity::PhysicalIntegrityValidationDigest::sha256(
            OPERATION_3_SCOPE_SHA
        )
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
            worth_store_physical_integrity::PhysicalByteRange::new(OBLIGATION_OFFSET + 1, 160)
                .unwrap(),
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
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut excluded_digest = operation_3();
    let covered_sha = independent_sha256(&excluded_digest[..128]);
    excluded_digest[128] ^= 1;
    assert_eq!(independent_sha256(&excluded_digest[..128]), covered_sha);
    assert_damage(
        &excluded_digest,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn fixed_record_bounds_distinguish_truncation_from_oversize_input() {
    let scope = scope(store(STORE_BYTES), identity(1, 2, 3));
    let bytes = operation_3();
    assert_damage(
        &bytes[..159],
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(OBLIGATION_OFFSET + 159, 1).unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
    let mut oversized = bytes.to_vec();
    oversized.push(0);
    assert_damage(
        &oversized,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        scope.byte_range(),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
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
        worth_store_physical_integrity::PhysicalDamageCause::WrongMagic,
        field_range(scope, 0, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::Magic),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut unknown_kind = operation_3();
    unknown_kind[9] = 0xff;
    reseal(&mut unknown_kind);
    assert_damage(
        &unknown_kind,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::RecordKindMismatch,
        field_range(scope, 9, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::OperationFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut header_reserved = operation_3();
    header_reserved[10] = 1;
    reseal(&mut header_reserved);
    assert_damage(
        &header_reserved,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        field_range(scope, 10, 6),
        Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut reserved = operation_3();
    reserved[107] = 1;
    reseal(&mut reserved);
    assert_damage(
        &reserved,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        field_range(scope, 107, 5),
        Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut target_shape = operation_3();
    target_shape[104] = 0;
    reseal(&mut target_shape);
    assert_damage(
        &target_shape,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        field_range(scope, 56, 72),
        Some(worth_store_physical_integrity::PhysicalFormatField::TargetShape),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut digest_presence = operation_3();
    digest_presence[105] = 2;
    reseal(&mut digest_presence);
    assert_damage(
        &digest_presence,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        field_range(scope, 105, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::PayloadDigestPresence),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut unsupported = operation_3();
    unsupported[8] = 7;
    reseal(&mut unsupported);
    let (rejection, counters) = rejection(&unsupported, scope);
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version) =
        rejection
    else {
        panic!("expected unsupported physical-work version, got {rejection:?}")
    };
    assert_eq!(version.scope(), scope);
    assert_eq!(
        version.axis(),
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalWorkObligation
    );
    assert_eq!(version.observed(), 7);
    assert_eq!(
        counters.rejected_for(
            worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported
        ),
        1
    );
}
