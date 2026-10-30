#[allow(unused_imports)]
use crate::physical_record_access::*;

#[test]
fn point_counters_do_not_scale_with_storage_cardinality() {
    let mut facade = open_facade();
    for value in 1..=8 {
        facade
            .append_physical_record(
                worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                    slot_cell_for(page(value), slot(value as u16)),
                    b"page-fill",
                ),
            )
            .expect("page append");
        facade
            .append_physical_record(
                worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                    extent_cell_for(extent(value)),
                    b"extent-fill",
                ),
            )
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
