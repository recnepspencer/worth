#[allow(unused_imports)]
use crate::manifest_access::*;

#[test]
fn root_manifest_and_manifest_index_use_public_physical_access() {
    let mut facade = open_facade();
    let page_append = facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell(1),
                b"root-page",
            ),
        )
        .expect("page append");
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                extent_cell(1),
                b"root-extent",
            ),
        )
        .expect("extent append");
    facade.publish_physical_root().expect("root publish");

    let root_report = facade
        .root_manifest_access()
        .current_root_manifest()
        .expect("discover current root");
    assert_eq!(root_report.segment_count(), 1);
    assert_eq!(root_report.page_slot_count(), 1);
    assert_eq!(root_report.extent_count(), 1);
    assert_eq!(root_report.counters().range_lookups(), 1);
    assert!(root_report.counters().bytes_read() > 0);

    let manifest_report = facade
        .manifest_index_access()
        .validate_membership(page_append.reference())
        .expect("page membership");
    assert_eq!(manifest_report.reference(), page_append.reference());
    assert_eq!(manifest_report.counters().index_probes(), 1);
    assert_eq!(
        manifest_report.counters().range_steps(),
        root_report.counters().range_steps()
    );
    assert_eq!(
        manifest_report.counters().page_touches(),
        root_report.counters().page_touches()
    );
    assert!(manifest_report.counters().bytes_read() > 0);
}

#[test]
fn allocation_free_space_and_fragmentation_stay_family_local() {
    let mut facade = open_facade();
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell(2),
                b"alloc-page",
            ),
        )
        .expect("page append");
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                extent_cell(2),
                b"alloc-extent",
            ),
        )
        .expect("extent append");
    facade.publish_physical_root().expect("root publish");

    let policy =
        worth_store_physical_format::PhysicalFreeSpaceSearchPolicy::foreground_bounded(4, 4);
    let root_report = facade
        .root_manifest_access()
        .current_root_manifest()
        .expect("discover current root");

    let allocation = facade
        .allocation_access()
        .allocation_classes()
        .expect("allocation classes");
    assert!(allocation
        .classes()
        .contains(&worth_store_physical_format::AllocationClassKind::OrdinaryRecordPage));
    assert!(allocation
        .classes()
        .contains(&worth_store_physical_format::AllocationClassKind::LargeRecordExtent));
    assert_eq!(allocation.counters().index_probes(), 0);
    assert_eq!(allocation.counters().range_lookups(), 1);
    assert_eq!(
        allocation.counters().range_steps(),
        root_report.counters().range_steps() + allocation.classes().len() as u16
    );
    assert!(allocation.counters().bytes_read() > 0);

    let free_space = facade
        .free_space_access()
        .bounded_candidates(policy)
        .expect("bounded free-space read");
    assert!(free_space.entries().is_empty());
    assert_eq!(free_space.counters().page_touches(), 1);
    assert_eq!(free_space.counters().index_probes(), 0);
    assert_eq!(
        free_space.counters().range_steps(),
        root_report.counters().range_steps() + free_space.entries().len() as u16
    );
    assert!(free_space.counters().bytes_read() > 0);

    let fragmentation = facade
        .fragmentation_access()
        .pressure(policy)
        .expect("fragmentation report");
    assert_eq!(fragmentation.pressure().candidate_classes(), 2);
    assert_eq!(fragmentation.pressure().fragmented_candidates(), 0);
    assert_eq!(fragmentation.counters().index_probes(), 0);
    assert_eq!(
        fragmentation.counters().range_steps(),
        root_report.counters().range_steps()
    );
    assert!(fragmentation.counters().bytes_read() > 0);
}

#[test]
fn reopen_discovers_same_root_manifest_family_truth() {
    let mut facade = open_facade();
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell(3),
                b"reopen-page",
            ),
        )
        .expect("page append");
    let published = facade.publish_physical_root().expect("root publish");
    let original = facade
        .root_manifest_access()
        .current_root_manifest()
        .expect("discover original root");
    let mut reopened = worth_store_physical_format::InMemoryPhysicalFormatModel::restore(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(
        ),
        published.replay_artifact(),
    )
    .expect("public replay reopen");
    let reopened_root = reopened
        .root_manifest_access()
        .current_root_manifest()
        .expect("discover reopened root");

    assert_eq!(original.root_reference(), reopened_root.root_reference());
    assert_eq!(original.page_slot_count(), reopened_root.page_slot_count());
}

#[test]
fn manifest_membership_rejects_unpublished_runtime_append() {
    let mut facade = open_facade();
    let page_append = facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                slot_cell(4),
                b"unpublished-page",
            ),
        )
        .expect("page append");
    let denial = facade
        .manifest_index_access()
        .validate_membership(page_append.reference())
        .expect_err("unpublished append must not be manifest-admitted");

    assert_eq!(
        denial.kind(),
        worth_store_physical_format::InMemoryPhysicalFormatModelDenialKind::MissingPhysicalRoot
    );
}

#[test]
fn allocation_free_space_and_fragmentation_require_published_root_truth() {
    let mut facade = open_facade();
    let policy =
        worth_store_physical_format::PhysicalFreeSpaceSearchPolicy::foreground_bounded(4, 4);

    let allocation_denial = facade
        .allocation_access()
        .allocation_classes()
        .expect_err("allocation must require published root");
    let free_space_denial = facade
        .free_space_access()
        .bounded_candidates(policy)
        .expect_err("free-space must require published root");
    let fragmentation_denial = facade
        .fragmentation_access()
        .pressure(policy)
        .expect_err("fragmentation must require published root");

    assert_eq!(
        allocation_denial.kind(),
        worth_store_physical_format::InMemoryPhysicalFormatModelDenialKind::MissingPhysicalRoot
    );
    assert_eq!(
        free_space_denial.kind(),
        worth_store_physical_format::InMemoryPhysicalFormatModelDenialKind::MissingPhysicalRoot
    );
    assert_eq!(
        fragmentation_denial.kind(),
        worth_store_physical_format::InMemoryPhysicalFormatModelDenialKind::MissingPhysicalRoot
    );
}
