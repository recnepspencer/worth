use worth_store_physical_format::{
    encode_data_frame_page_lsn, DurableFrameKind, DurableInlineRecordPlacement, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageLsn, PhysicalPageSizeClass,
};
use worth_store_physical_integrity::{
    validate_inline_page, InlinePageIntegrityValidation, InlineRecordProjectionDenial,
    PhysicalDamageCause, PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    UntrustedPhysicalArtifact,
};

use super::support::{clean_page, format, page, page_scope, record, slot, store};

#[test]
fn sealed_page_projects_exact_record_payload_and_page_lsn_without_raw_bytes() {
    let page_cell = page(3, 4, 5);
    let mut bytes = clean_page(PhysicalPageSizeClass::KiB16, page_cell);
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(91),
    )
    .unwrap();
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_inline_page(
        input,
        page_scope(store(7), PhysicalPageSizeClass::KiB16, page_cell),
    );
    let InlinePageIntegrityValidation::Intact(validated) = validation else {
        panic!("clean inline page rejected");
    };
    let placement = placement(page_cell, record(0xb2, 2), 2, 22, 6);

    let projection = validated.project_record(input, placement).unwrap();

    assert_eq!(projection.record(), placement.record());
    assert_eq!(projection.page_identity(), page_cell);
    assert_eq!(projection.slot_identity(), placement.slot_cell());
    assert_eq!(projection.page_lsn(), PhysicalPageLsn::new(91));
    assert_eq!(&bytes[projection.payload_range()], b"beta!!");
}

#[test]
fn page_projection_denies_foreign_incarnation_and_each_owner_identity_mismatch() {
    let page_cell = page(3, 4, 5);
    let bytes = clean_page(PhysicalPageSizeClass::KiB16, page_cell);
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_inline_page(
        input,
        page_scope(store(7), PhysicalPageSizeClass::KiB16, page_cell),
    );
    let InlinePageIntegrityValidation::Intact(validated) = validation else {
        panic!("clean inline page rejected");
    };
    let exact = placement(page_cell, record(0xb2, 2), 2, 22, 6);
    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_record(
                UntrustedPhysicalArtifact::from_bounded_bytes(&equal_copy),
                exact,
            )
            .unwrap_err(),
        InlineRecordProjectionDenial::InputIncarnationMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page(3, 4, 6), record(0xb2, 2), 2, 22, 6))
            .unwrap_err(),
        InlineRecordProjectionDenial::PageIdentityMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xb2, 2), 3, 22, 6))
            .unwrap_err(),
        InlineRecordProjectionDenial::SlotIdentityMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xc3, 3), 2, 22, 6))
            .unwrap_err(),
        InlineRecordProjectionDenial::RecordIdentityMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xb2, 2), 2, 23, 6))
            .unwrap_err(),
        InlineRecordProjectionDenial::SlotGenerationMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xb2, 2), 2, 22, 5))
            .unwrap_err(),
        InlineRecordProjectionDenial::PayloadLengthMismatch
    );
}

#[test]
fn clean_page_is_rejected_under_a_foreign_page_incarnation_scope() {
    let exact = page(3, 4, 5);
    let bytes = clean_page(PhysicalPageSizeClass::KiB16, exact);
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_inline_page(
        input,
        page_scope(store(7), PhysicalPageSizeClass::KiB16, page(3, 4, 6)),
    );

    let InlinePageIntegrityValidation::Rejected(PhysicalIntegrityRejection::Damaged(damage)) =
        validation
    else {
        panic!("foreign page incarnation must be rejected as localized physical damage")
    };
    assert_eq!(
        damage.cause(),
        PhysicalDamageCause::PhysicalGenerationMismatch
    );
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(
            PhysicalDamageCause::PhysicalGenerationMismatch,
        )),
        1
    );
}

fn placement(
    page: worth_store_physical_format::PageGenerationCell,
    record: worth_store_physical_format::PersistedRecordIdentity,
    slot_number: u16,
    slot_generation: u64,
    payload_bytes: u64,
) -> DurableInlineRecordPlacement {
    let segment = PhysicalGenerationAuthority::for_canonical_physical_format()
        .segment_cell(page.segment_id())
        .with_segment_generation(PhysicalGeneration::from_raw(8).unwrap());
    DurableInlineRecordPlacement::new(
        record,
        segment,
        page,
        slot(page, slot_number, slot_generation),
        format(PhysicalPageSizeClass::KiB16).page_size().bytes(),
        payload_bytes,
    )
    .unwrap()
}
