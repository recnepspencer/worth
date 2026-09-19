//! What a caller pays to enumerate a whole kind, and what it is told when it
//! cannot pay it.

use super::*;

const ENTITY_KIND: crate::facade::identity::KindId = crate::facade::identity::KindId(1);
const RELATION_KIND: crate::facade::identity::KindId = crate::facade::identity::KindId(2);

#[test]
fn a_bounded_kind_read_admits_exactly_the_work_the_live_kind_costs() {
    let runtime = runtime_with_test_schema();
    let mut live = Vec::new();
    for ordinal in 0..8 {
        live.push(create_entity(&runtime, &format!("part-{ordinal}")));
    }
    let retired = live.remove(3);
    delete_entity(&runtime, retired);
    let version = runtime.current_version_id();

    // Two units per live entity: one to look at its slot, one to reserve the
    // record it materialized into.
    let exact = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(ENTITY_KIND, version, 14)
        .expect("fourteen units pay for seven live entities of the kind");
    assert_eq!(exact.entity_slots_examined(), 7);
    assert_eq!(exact.entity_records_reserved(), 7);
    assert_eq!(exact.work_units(), 14);
    let seen = exact
        .into_records()
        .into_iter()
        .map(|record| record.entity_id)
        .collect::<Vec<_>>();
    assert_eq!(seen.len(), live.len());
    for entity in &live {
        assert!(
            seen.contains(entity),
            "a live entity of the kind is missing"
        );
    }
    assert!(
        !seen.contains(&retired),
        "a deleted entity is not live state and must not be validated as if it were"
    );

    let refused = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(ENTITY_KIND, version, 13)
        .expect_err("one unit short cannot enumerate the whole kind");
    assert_eq!(refused.consumed_work_units(), 13);
    assert_eq!(refused.entity_slots_examined(), 7);
    assert_eq!(refused.entity_records_reserved(), 6);
}

#[test]
fn a_kind_with_no_entities_still_costs_the_slots_it_had_to_look_at() {
    let runtime = runtime_with_test_schema();
    for ordinal in 0..5 {
        create_entity(&runtime, &format!("part-{ordinal}"));
    }
    let version = runtime.current_version_id();

    // The relation kind is a real registered kind that no entity carries. The
    // scan cannot know that without looking, so the five slots are charged and
    // the answer is empty: cost follows the state that exists, not the answer.
    let empty = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(RELATION_KIND, version, 5)
        .expect("five units pay for looking at five entity slots");
    assert_eq!(empty.entity_slots_examined(), 5);
    assert_eq!(empty.entity_records_reserved(), 0);
    assert!(empty.into_records().is_empty());

    let refused = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(RELATION_KIND, version, 4)
        .expect_err("four units cannot look at five slots");
    assert_eq!(refused.entity_slots_examined(), 4);
    assert_eq!(refused.entity_records_reserved(), 0);
}

#[test]
fn a_maximum_of_zero_refuses_before_examining_anything() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "part");

    let refused = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(ENTITY_KIND, runtime.current_version_id(), 0)
        .expect_err("a budget of zero admits no work at all");
    assert_eq!(refused.entity_slots_examined(), 0);
    assert_eq!(refused.entity_records_reserved(), 0);
    assert_eq!(refused.consumed_work_units(), 0);
}

#[test]
fn the_unbounded_kind_read_answers_exactly_as_the_bounded_one() {
    let runtime = runtime_with_test_schema();
    for ordinal in 0..6 {
        create_entity(&runtime, &format!("part-{ordinal}"));
    }
    delete_entity(&runtime, create_entity(&runtime, "retired"));
    let version = runtime.current_version_id();

    let unbounded = runtime
        .read_truth()
        .visible_entities_of_kind(ENTITY_KIND, version)
        .into_iter()
        .map(|record| record.entity_id)
        .collect::<Vec<_>>();
    let bounded = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(ENTITY_KIND, version, usize::MAX)
        .expect("an unbounded budget cannot be exhausted")
        .into_records()
        .into_iter()
        .map(|record| record.entity_id)
        .collect::<Vec<_>>();
    assert_eq!(unbounded, bounded);
    assert_eq!(unbounded.len(), 6);
}

#[test]
fn a_bounded_kind_read_answers_the_version_it_was_asked_for() {
    let runtime = runtime_with_test_schema();
    let early = create_entity(&runtime, "early");
    let before = runtime.current_version_id();
    let late = create_entity(&runtime, "late");

    let historical = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(ENTITY_KIND, before, usize::MAX)
        .expect("an unbounded budget cannot be exhausted")
        .into_records()
        .into_iter()
        .map(|record| record.entity_id)
        .collect::<Vec<_>>();
    assert!(historical.contains(&early));
    assert!(
        !historical.contains(&late),
        "a bounded read at an earlier version must not see later state"
    );

    let current = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(ENTITY_KIND, runtime.current_version_id(), usize::MAX)
        .expect("an unbounded budget cannot be exhausted")
        .into_records()
        .into_iter()
        .map(|record| record.entity_id)
        .collect::<Vec<_>>();
    assert!(current.contains(&early));
    assert!(current.contains(&late));
}

#[test]
fn a_refused_historical_kind_read_charges_only_slots_it_examined() {
    let runtime = runtime_with_test_schema();
    let _early = create_entity(&runtime, "early");
    let historical_version = runtime.current_version_id();
    let _late = create_entity(&runtime, "late");

    runtime.performance_access().reset_counters();
    let refused = runtime
        .read_truth()
        .bounded_visible_entities_of_kind(RELATION_KIND, historical_version, 1)
        .expect_err("one work unit cannot examine the whole historical arena");
    let counters = runtime.performance_access().counters();

    assert_eq!(refused.entity_slots_examined(), 1);
    assert_eq!(
        counters.visibility_entity_slot_scans, 1,
        "instrumentation must charge actual historical work, not the arena width refused before scanning"
    );
}
