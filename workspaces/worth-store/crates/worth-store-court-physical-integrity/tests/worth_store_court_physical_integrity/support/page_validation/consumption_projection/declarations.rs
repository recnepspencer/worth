pub(crate) use worth_store_physical_format::{
    encode_data_frame_page_lsn, DurableFrameKind, DurableInlineRecordPlacement, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageLsn, PhysicalPageSizeClass,
};
pub(crate) use worth_store_physical_integrity::{
    validate_inline_page, InlinePageIntegrityValidation, InlineRecordProjectionDenial,
    PhysicalDamageCause, PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::page_validation::{
    clean_page, format, page, page_scope, record, slot, store,
};

pub(crate) fn placement(
    page: worth_store_physical_format::PageGenerationCell,
    record: worth_store_physical_format::PersistedRecordIdentity,
    slot_number: u16,
    slot_generation: u64,
    payload_bytes: u64,
) -> DurableInlineRecordPlacement {
    let segment =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .segment_cell(page.segment_id())
            .with_segment_generation(
                worth_store_physical_format::PhysicalGeneration::from_raw(8).unwrap(),
            );
    worth_store_physical_format::DurableInlineRecordPlacement::legacy_unknown(
        record,
        segment,
        page,
        slot(page, slot_number, slot_generation),
        format(worth_store_physical_format::PhysicalPageSizeClass::KiB16)
            .page_size()
            .bytes(),
        payload_bytes,
    )
    .unwrap()
}
