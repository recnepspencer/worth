//! Relation-kind inventory is exact, versioned, and refuses before excess work.

use super::*;

const RELATION_KIND: crate::facade::identity::KindId = crate::facade::identity::KindId(2);

#[test]
fn bounded_relation_kind_read_charges_live_slots_and_records() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let first_target = create_entity(&runtime, "first-target");
    let removed_target = create_entity(&runtime, "removed-target");
    let second_target = create_entity(&runtime, "second-target");
    let first = create_relation(&runtime, source, first_target, "first");
    let removed = create_relation(&runtime, source, removed_target, "removed");
    let second = create_relation(&runtime, source, second_target, "second");
    delete_relation_on_branch(&runtime, removed, BranchId("main".to_owned()));
    let version = runtime.current_version_id();

    let exact = runtime
        .read_truth()
        .bounded_visible_relations_of_kind(RELATION_KIND, version, 4)
        .expect("two live relations cost two examined slots and two records");
    assert_eq!(exact.relation_slots_examined(), 2);
    assert_eq!(exact.relation_records_reserved(), 2);
    assert_eq!(exact.work_units(), 4);
    let ids = exact
        .records()
        .iter()
        .map(|record| record.relation_id)
        .collect::<Vec<_>>();
    assert!(ids.contains(&first));
    assert!(ids.contains(&second));
    assert!(!ids.contains(&removed));

    let refused = runtime
        .read_truth()
        .bounded_visible_relations_of_kind(RELATION_KIND, version, 3)
        .expect_err("one unit short must refuse before completing the inventory");
    let crate::facade::runtime::RelationKindTruthReadDenial::WorkLimitExceeded(refused) = refused
    else {
        panic!("the intact inventory may fail only on its declared work bound");
    };
    assert_eq!(refused.consumed_work_units(), 3);
    assert_eq!(refused.relation_slots_examined(), 2);
    assert_eq!(refused.relation_records_reserved(), 1);
}

#[test]
fn bounded_relation_kind_read_uses_the_requested_historical_version() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let early_target = create_entity(&runtime, "early-target");
    let late_target = create_entity(&runtime, "late-target");
    let early = create_relation(&runtime, source, early_target, "early");
    let before = runtime.current_version_id();
    let late = create_relation(&runtime, source, late_target, "late");

    let historical = runtime
        .read_truth()
        .bounded_visible_relations_of_kind(RELATION_KIND, before, 3)
        .expect("two occupied slots and one historical record cost three units");
    assert_eq!(historical.relation_slots_examined(), 2);
    assert_eq!(historical.relation_records_reserved(), 1);
    assert_eq!(historical.records()[0].relation_id, early);
    assert_ne!(historical.records()[0].relation_id, late);
    let current = runtime
        .read_truth()
        .bounded_visible_relations_of_kind(RELATION_KIND, runtime.current_version_id(), 4)
        .expect("both live relations are visible now");
    assert_eq!(current.relation_records_reserved(), 2);
    assert_eq!(
        current.into_records(),
        runtime
            .read_truth()
            .visible_relations_of_kind(RELATION_KIND, runtime.current_version_id())
    );
}

#[test]
fn refused_historical_relation_scan_charges_only_examined_slots() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let early_target = create_entity(&runtime, "early-target");
    let late_target = create_entity(&runtime, "late-target");
    create_relation(&runtime, source, early_target, "early");
    let before = runtime.current_version_id();
    create_relation(&runtime, source, late_target, "late");

    runtime.performance_access().reset_counters();
    let refused = runtime
        .read_truth()
        .bounded_visible_relations_of_kind(RELATION_KIND, before, 1)
        .expect_err("one unit cannot both examine and reserve the early relation");
    let crate::facade::runtime::RelationKindTruthReadDenial::WorkLimitExceeded(refused) = refused
    else {
        panic!("the intact historical inventory may fail only on work");
    };
    let counters = runtime.performance_access().counters();
    assert_eq!(refused.consumed_work_units(), 1);
    assert_eq!(refused.relation_slots_examined(), 1);
    assert_eq!(refused.relation_records_reserved(), 0);
    assert_eq!(counters.visibility_relation_slot_scans, 1);
}
