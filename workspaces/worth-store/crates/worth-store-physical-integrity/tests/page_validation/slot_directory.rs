use worth_store_physical_format::{encode_inline_page, InlineRecordAppend, PhysicalPageSizeClass};
use worth_store_physical_integrity::{
    validate_inline_page, InlinePageIntegrityValidation, PhysicalBlastRadius, PhysicalDamageCause,
    PhysicalFormatField, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, field_range, format, page, page_scope, record, reseal, slot, store,
};

#[test]
fn canonical_slot_directory_and_zero_gaps_admit_owner_geometry() {
    let page_size = PhysicalPageSizeClass::KiB16;
    let identity = page(7, 9, 11);
    let bytes = encode_inline_page(
        format(page_size),
        identity,
        &[
            InlineRecordAppend::new(record(0x31, 1), slot(identity, 1, 3), b"one"),
            InlineRecordAppend::new(record(0x32, 2), slot(identity, 2, 4), b"two-two"),
        ],
    )
    .unwrap();
    let scope = page_scope(store(5), page_size, identity);
    let (validation, _) =
        validate_inline_page(UntrustedPhysicalArtifact::from_bounded_bytes(&bytes), scope);
    let InlinePageIntegrityValidation::Intact(validated) = validation else {
        panic!("canonical slot directory rejected");
    };
    assert_eq!(validated.slot_count(), 2);
    assert_eq!(validated.free_bytes(), 16_384 - 48 - 24 - 80 - 10);
}

#[test]
fn slot_count_record_identity_generation_location_and_gap_damage_are_exact() {
    let page_size = PhysicalPageSizeClass::KiB16;
    let identity = page(7, 9, 11);
    let scope = page_scope(store(5), page_size, identity);
    let canonical = || {
        encode_inline_page(
            format(page_size),
            identity,
            &[
                InlineRecordAppend::new(record(0x31, 1), slot(identity, 1, 3), b"one"),
                InlineRecordAppend::new(record(0x32, 2), slot(identity, 2, 4), b"two-two"),
            ],
        )
        .unwrap()
    };

    let mut reserved_prefix = canonical();
    reserved_prefix[66] = 1;
    reseal(&mut reserved_prefix);
    assert_damage(
        &reserved_prefix,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 66, 6),
        Some(PhysicalFormatField::Reserved),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut slot_count = canonical();
    slot_count[64..66].copy_from_slice(&u16::MAX.to_le_bytes());
    reseal(&mut slot_count);
    assert_damage(
        &slot_count,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 64, 2),
        Some(PhysicalFormatField::Payload),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut record_identity = canonical();
    record_identity[72..96].fill(0);
    reseal(&mut record_identity);
    assert_damage(
        &record_identity,
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        field_range(scope, 72, 24),
        Some(PhysicalFormatField::RecordIdentity),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut duplicate_record = canonical();
    duplicate_record.copy_within(72..96, 112);
    reseal(&mut duplicate_record);
    assert_damage(
        &duplicate_record,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 112, 24),
        Some(PhysicalFormatField::RecordIdentity),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut zero_slot_generation = canonical();
    zero_slot_generation[104..112].fill(0);
    reseal(&mut zero_slot_generation);
    assert_damage(
        &zero_slot_generation,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 104, 8),
        Some(PhysicalFormatField::PhysicalGeneration),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut overlapping_payload = canonical();
    overlapping_payload[96..100].copy_from_slice(&103_u32.to_le_bytes());
    overlapping_payload[100..104].copy_from_slice(&2_u32.to_le_bytes());
    reseal(&mut overlapping_payload);
    assert_damage(
        &overlapping_payload,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 96, 8),
        Some(PhysicalFormatField::Payload),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut nonzero_gap = canonical();
    nonzero_gap[200] = 0x5a;
    reseal(&mut nonzero_gap);
    assert_damage(
        &nonzero_gap,
        scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(scope, 200, 1),
        Some(PhysicalFormatField::Reserved),
        PhysicalBlastRadius::DamagedRange,
    );
}
