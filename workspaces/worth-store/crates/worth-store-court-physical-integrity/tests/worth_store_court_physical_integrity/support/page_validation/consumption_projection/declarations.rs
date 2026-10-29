pub(crate) use {worth_store_physical_format::encode_data_frame_page_lsn, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::DurableInlineRecordPlacement, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageLsn, worth_store_physical_format::PhysicalPageSizeClass};
pub(crate) use {worth_store_physical_integrity::validate_inline_page, worth_store_physical_integrity::InlinePageIntegrityValidation, worth_store_physical_integrity::InlineRecordProjectionDenial, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::page_validation::clean_page, crate::support::page_validation::format, crate::support::page_validation::page, crate::support::page_validation::page_scope, crate::support::page_validation::record, crate::support::page_validation::slot, crate::support::page_validation::store};

pub(crate) fn placement(
    page: worth_store_physical_format::PageGenerationCell,
    record: worth_store_physical_format::PersistedRecordIdentity,
    slot_number: u16,
    slot_generation: u64,
    payload_bytes: u64,
) -> DurableInlineRecordPlacement {
    let segment = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .segment_cell(page.segment_id())
        .with_segment_generation(worth_store_physical_format::PhysicalGeneration::from_raw(8).unwrap());
    worth_store_physical_format::DurableInlineRecordPlacement::legacy_unknown(
        record,
        segment,
        page,
        slot(page, slot_number, slot_generation),
        format(worth_store_physical_format::PhysicalPageSizeClass::KiB16).page_size().bytes(),
        payload_bytes,
    )
    .unwrap()
}