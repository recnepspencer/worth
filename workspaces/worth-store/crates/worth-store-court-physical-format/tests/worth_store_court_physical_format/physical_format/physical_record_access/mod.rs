#[allow(unused_imports)] use crate::physical_record_access::*;
use {worth_store_contracts::AcceptedHandoffReadiness, worth_store_contracts::HandoffEvidenceDigestSet, worth_store_contracts::StableDigest, worth_store_contracts::ROADMAP_2_S1_SCOPE};
use {worth_store_physical_format::access::extent::extent_access_counters, worth_store_physical_format::access::page::page_access_counters, worth_store_physical_format::InMemoryPhysicalFormatModel, worth_store_physical_format::InMemoryPhysicalFormatModelRequest, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalRecordSlot, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::PlatformPhysicalAppendRequest};

#[test]
fn page_and_frame_operations_use_public_physical_access() {
    let mut facade = open_facade();
    let append = facade
        .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
            slot_cell(1),
            b"frame-backed",
        ))
        .expect("page append through public facade");
    let mut page_layout = facade.page_access();
    let located = page_layout
        .read_record(append.reference())
        .expect("public page read");
    assert_eq!(located.record_view().payload().as_bytes(), b"frame-backed");

    let mut frame_layout = facade.frame_access();
    let framed = frame_layout
        .read_frame(append.reference())
        .expect("public frame read");
    assert_eq!(framed.frame_view().payload().as_bytes(), b"frame-backed");
    assert_eq!(framed.counters().point_lookups(), 1);
}

#[test]
fn extent_and_reopen_follow_public_physical_evidence() {
    let mut facade = open_facade();
    let append = facade
        .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
            extent_cell(1),
            b"extent-backed",
        ))
        .expect("extent append through public facade");
    let page_append = facade
        .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
            slot_cell(2),
            b"root-page",
        ))
        .expect("page append before publish");
    let mut extent_layout = facade.extent_access();
    let extent = extent_layout
        .read_record(append.reference())
        .expect("public extent read");
    assert_eq!(extent.record_view().payload().as_bytes(), b"extent-backed");

    let published = facade.publish_physical_root().expect("public root publish");
    let mut reopened = worth_store_physical_format::InMemoryPhysicalFormatModel::restore(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(),
        published.replay_artifact(),
    )
    .expect("public replay reopen");
    let mut reopened_page_layout = reopened.page_access();
    let reopened_page = reopened_page_layout
        .locate_record(page_append.reference())
        .expect("public page locate after reopen");
    assert_eq!(
        reopened_page.record_view().payload().as_bytes(),
        b"root-page"
    );
}

#[test]
fn segment_access_uses_maintained_segment_occupancy() {
    let mut facade = open_facade();
    facade
        .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
            slot_cell(3),
            b"seg-page",
        ))
        .expect("page append");
    facade
        .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
            extent_cell(3),
            b"seg-extent",
        ))
        .expect("extent append");
    let report = facade
        .segment_access()
        .read_segment(segment(1))
        .expect("public segment read");

    assert_eq!(report.page_slots(), 1);
    assert_eq!(report.extents(), 1);
    assert_eq!(report.counters().point_lookups(), 1);
    assert_eq!(report.counters().page_touches(), 0);
    assert_eq!(report.counters().bytes_read(), 0);
}

#[test]
fn point_counters_do_not_scale_with_storage_cardinality() {
    let mut facade = open_facade();
    for value in 1..=8 {
        facade
            .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell_for(page(value), slot(value as u16)),
                b"page-fill",
            ))
            .expect("page append");
        facade
            .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                extent_cell_for(extent(value)),
                b"extent-fill",
            ))
            .expect("extent append");
    }
    let page_reference =
        worth_store_physical_format::PhysicalReferenceAuthority::for_canonical_physical_format()
            .admit_page_slot(slot_cell_for(page(8), slot(8)))
            .reference();
    let extent_reference =
        worth_store_physical_format::PhysicalReferenceAuthority::for_canonical_physical_format()
            .admit_extent(extent_cell_for(extent(8)))
            .reference();

    let page_counters = {
        let mut page_layout = facade.page_access();
        let report = page_layout
            .read_record(page_reference)
            .expect("public page read");
        page_access_counters(report)
    };
    let extent_counters = {
        let mut extent_layout = facade.extent_access();
        let report = extent_layout
            .read_record(extent_reference)
            .expect("public extent read");
        extent_access_counters(report)
    };

    assert_eq!(page_counters.point_lookups(), 1);
    assert_eq!(page_counters.page_touches(), 1);
    assert_eq!(page_counters.index_probes(), 2);
    assert_eq!(page_counters.bytes_read(), 4_096);
    assert_eq!(extent_counters.point_lookups(), 1);
    assert_eq!(extent_counters.page_touches(), 1);
    assert_eq!(extent_counters.index_probes(), 2);
    assert_eq!(extent_counters.bytes_read(), 4_096);
}
