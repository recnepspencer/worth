use worth_query_host::facade::primary_graph;

use super::super::courtroom_support::{observe, outcome_kind};
use super::super::world::CourtroomWorld;

pub fn duplicate_reordered_and_foreign_clocks_fail_closed() {
    let world = CourtroomWorld::publish("blocked");
    let selected = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let mut clock = selected.conditional_clock(&world.clock).unwrap();
    let _ = clock.observe();
    world.clock_control.push(1, 10);
    let duplicate = clock.observe();
    assert!(
        matches!(
            duplicate,
            primary_graph::WorthQueryConditionalClockObservationOutcome::Duplicate(_)
        ),
        "{}",
        outcome_kind(&duplicate)
    );
    world.clock_control.push(2, 9);
    assert!(matches!(
        clock.observe(),
        primary_graph::WorthQueryConditionalClockObservationOutcome::Reordered
    ));
    world.clock_control.push(0, 10);
    assert!(matches!(
        clock.observe(),
        primary_graph::WorthQueryConditionalClockObservationOutcome::Stale
    ));
    drop(clock);
    let foreign = CourtroomWorld::publish("ready");
    let denial = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .unwrap()
        .conditional_clock(&foreign.clock)
        .err()
        .expect("foreign clock handle must fail closed");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalClockObservationDenialKind::ForeignRuntime
    );
}

pub fn provider_replacement_requires_fresh_runtime_publication() {
    let incumbent = CourtroomWorld::publish("ready");
    let mut replacement = CourtroomWorld::publish_replacement("ready");
    let denial = replacement
        .application
        .select_product_branch(replacement.application.product_runtime().default_branch())
        .unwrap()
        .conditional_clock(&incumbent.clock)
        .err()
        .expect("replacement runtime must reject incumbent provider clock affinity");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalClockObservationDenialKind::ForeignRuntime
    );
    drop(incumbent);

    let receipt = observe(&mut replacement);
    assert_eq!(receipt.committed_operation_count(), 1);
    assert_eq!(replacement.contacts.snapshot(), (1, 1, 1, 1));
}
