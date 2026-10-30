#[allow(unused_imports)]
use crate::physical_record_access::*;

#[test]
fn page_and_frame_operations_use_public_physical_access() {
    let mut facade = open_facade();
    let append = facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell(1),
                b"frame-backed",
            ),
        )
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
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                extent_cell(1),
                b"extent-backed",
            ),
        )
        .expect("extent append through public facade");
    let page_append = facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell(2),
                b"root-page",
            ),
        )
        .expect("page append before publish");
    let mut extent_layout = facade.extent_access();
    let extent = extent_layout
        .read_record(append.reference())
        .expect("public extent read");
    assert_eq!(extent.record_view().payload().as_bytes(), b"extent-backed");

    let published = facade.publish_physical_root().expect("public root publish");
    let mut reopened = worth_store_physical_format::InMemoryPhysicalFormatModel::restore(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(
        ),
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
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell(3),
                b"seg-page",
            ),
        )
        .expect("page append");
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                extent_cell(3),
                b"seg-extent",
            ),
        )
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
