#[allow(unused_imports)] use crate::page_validation::consumption_projection::*;
use {worth_store_physical_format::encode_data_frame_page_lsn, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::DurableInlineRecordPlacement, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageLsn, worth_store_physical_format::PhysicalPageSizeClass};
use {worth_store_physical_integrity::validate_inline_page, worth_store_physical_integrity::InlinePageIntegrityValidation, worth_store_physical_integrity::InlineRecordProjectionDenial, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::page_validation::clean_page, crate::support::page_validation::format, crate::support::page_validation::page, crate::support::page_validation::page_scope, crate::support::page_validation::record, crate::support::page_validation::slot, crate::support::page_validation::store};

#[test]
fn sealed_page_projects_exact_record_payload_and_page_lsn_without_raw_bytes() {
    let page_cell = page(3, 4, 5);
    let mut bytes = clean_page(worth_store_physical_format::PhysicalPageSizeClass::KiB16, page_cell);
    encode_data_frame_page_lsn(
        &mut bytes,
        worth_store_physical_format::DurableFrameKind::InlinePage,
        worth_store_physical_format::PhysicalPageLsn::new(91),
    )
    .unwrap();
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_inline_page(
        input,
        page_scope(store(7), worth_store_physical_format::PhysicalPageSizeClass::KiB16, page_cell),
    );
    let worth_store_physical_integrity::InlinePageIntegrityValidation::Intact(validated) = validation else {
        panic!("clean inline page rejected");
    };
    let placement = placement(page_cell, record(0xb2, 2), 2, 22, 6);

    let projection = validated.project_record(input, placement).unwrap();

    assert_eq!(projection.record(), placement.record());
    assert_eq!(projection.page_identity(), page_cell);
    assert_eq!(projection.slot_identity(), placement.slot_cell());
    assert_eq!(projection.page_lsn(), worth_store_physical_format::PhysicalPageLsn::new(91));
    assert_eq!(&bytes[projection.payload_range()], b"beta!!");
}

#[test]
fn page_projection_denies_foreign_incarnation_and_each_owner_identity_mismatch() {
    let page_cell = page(3, 4, 5);
    let bytes = clean_page(worth_store_physical_format::PhysicalPageSizeClass::KiB16, page_cell);
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_inline_page(
        input,
        page_scope(store(7), worth_store_physical_format::PhysicalPageSizeClass::KiB16, page_cell),
    );
    let worth_store_physical_integrity::InlinePageIntegrityValidation::Intact(validated) = validation else {
        panic!("clean inline page rejected");
    };
    let exact = placement(page_cell, record(0xb2, 2), 2, 22, 6);
    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_record(
                worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&equal_copy),
                exact,
            )
            .unwrap_err(),
        worth_store_physical_integrity::InlineRecordProjectionDenial::InputIncarnationMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page(3, 4, 6), record(0xb2, 2), 2, 22, 6))
            .unwrap_err(),
        worth_store_physical_integrity::InlineRecordProjectionDenial::PageIdentityMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xb2, 2), 3, 22, 6))
            .unwrap_err(),
        worth_store_physical_integrity::InlineRecordProjectionDenial::SlotIdentityMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xc3, 3), 2, 22, 6))
            .unwrap_err(),
        worth_store_physical_integrity::InlineRecordProjectionDenial::RecordIdentityMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xb2, 2), 2, 23, 6))
            .unwrap_err(),
        worth_store_physical_integrity::InlineRecordProjectionDenial::SlotGenerationMismatch
    );
    assert_eq!(
        validated
            .project_record(input, placement(page_cell, record(0xb2, 2), 2, 22, 5))
            .unwrap_err(),
        worth_store_physical_integrity::InlineRecordProjectionDenial::PayloadLengthMismatch
    );
}

#[test]
fn clean_page_is_rejected_under_a_foreign_page_incarnation_scope() {
    let exact = page(3, 4, 5);
    let bytes = clean_page(worth_store_physical_format::PhysicalPageSizeClass::KiB16, exact);
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_inline_page(
        input,
        page_scope(store(7), worth_store_physical_format::PhysicalPageSizeClass::KiB16, page(3, 4, 6)),
    );

    let worth_store_physical_integrity::InlinePageIntegrityValidation::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage)) =
        validation
    else {
        panic!("foreign page incarnation must be rejected as localized physical damage")
    };
    assert_eq!(
        damage.cause(),
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch
    );
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        )),
        1
    );
}
