#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::wal_reopen::member_denial::*;

use {
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::build_three_segment_inventory,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::reopen_failure,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::segment_path,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::shift_segment_lsns,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::wal_policy,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::SEGMENT_BYTES,
};

#[test]
fn contiguous_outer_topology_rejects_a_resealed_invalid_inner_member_without_repair() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory(&store_root);
    let first = segment_path(&store_root, 1, 1);
    reseal_with_invalid_inner_redo(&first);
    let before = std::fs::read(&first).unwrap();

    assert_eq!(
        reopen_failure(&store_root, wal_policy(SEGMENT_BYTES, 3)),
        worth_store::physical_runtime::PhysicalWalOpenFailure::MemberPayloadRejected,
    );
    assert_eq!(std::fs::read(&first).unwrap(), before);
}

#[test]
fn earlier_invalid_inner_member_does_not_hide_a_later_outer_lsn_gap_or_repair_media() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory(&store_root);
    let first = segment_path(&store_root, 1, 1);
    let third = segment_path(&store_root, 3, 1);
    reseal_with_invalid_inner_redo(&first);
    shift_segment_lsns(&third, 1);
    let first_before = std::fs::read(&first).unwrap();
    let third_before = std::fs::read(&third).unwrap();

    assert_eq!(
        reopen_failure(&store_root, wal_policy(SEGMENT_BYTES, 3)),
        worth_store::physical_runtime::PhysicalWalOpenFailure::Topology(
            worth_store_wal::WalTopologyDenialKind::Gap
        ),
    );
    assert_eq!(std::fs::read(&first).unwrap(), first_before);
    assert_eq!(std::fs::read(&third).unwrap(), third_before);
}
