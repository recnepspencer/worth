use worth_query_host::facade::primary_graph;

use super::super::courtroom_support::{observe, outcome_kind};
use super::super::world::CourtroomWorld;

pub fn duplicate_reordered_and_foreign_clocks_fail_closed(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let world = CourtroomWorld::publish("blocked");
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let mut clock = selected
        .conditional_clock(request_execution, &world.clock)
        .unwrap();
    let _ = clock.observe(request_execution);
    world.clock_control.push(1, 10);
    let duplicate = clock.observe(request_execution);
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
        clock.observe(request_execution),
        primary_graph::WorthQueryConditionalClockObservationOutcome::Reordered
    ));
    world.clock_control.push(0, 10);
    assert!(matches!(
        clock.observe(request_execution),
        primary_graph::WorthQueryConditionalClockObservationOutcome::Stale
    ));
    drop(clock);
    let foreign = CourtroomWorld::publish("ready");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
                .conditional_clock(request_execution, &foreign.clock)
                .err()
                .expect("foreign clock handle must fail closed")
        })
        .expect("installed host call opens");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalClockObservationDenialKind::ForeignRuntime
    );
}

pub fn provider_replacement_requires_fresh_runtime_publication(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let incumbent = CourtroomWorld::publish("ready");
    let replacement = CourtroomWorld::publish_replacement("ready");
    let denial = replacement
        .application
        .with_host_advancement(|phase| {
            replacement
                .application
                .on_branch(replacement.application.current_world())
                .select(&phase)
                .unwrap()
                .conditional_clock(request_execution, &incumbent.clock)
                .err()
                .expect("replacement runtime must reject incumbent provider clock affinity")
        })
        .expect("installed host call opens");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalClockObservationDenialKind::ForeignRuntime
    );
    drop(incumbent);

    let receipt = observe(request_execution, &replacement);
    assert_eq!(receipt.committed_operation_count(), 1);
    assert_eq!(replacement.contacts.snapshot(), (1, 1, 1, 1));
}
