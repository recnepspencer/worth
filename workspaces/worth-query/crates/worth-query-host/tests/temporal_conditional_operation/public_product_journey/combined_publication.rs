use std::sync::Arc;

use worth_query_host::facade::{primary_graph, runtime};

use super::super::adapters::{CompletingExternalTransport, ReplacementPredicate};
use super::super::courtroom_lifecycle::assert_conditional_resources_empty;
use super::super::world::CourtroomWorld;

pub(crate) fn application_commits_relational_and_signal_in_one_world_publication() {
    let mut world = CourtroomWorld::publish("blocked");
    let transport = Arc::new(CompletingExternalTransport::default());
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let branch = world.application.current_world();
    let before = world
        .application
        .on_branch(branch)
        .select()
        .unwrap()
        .product()
        .selected_commit()
        .clone();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(warm) = world
        .application
        .on_branch(branch)
        .select()
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe()
    else {
        panic!("the predecessor definition must warm on the selected product")
    };
    assert_eq!(warm.retained_suppressed_wake_count(), 1);
    drop(warm);
    let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());

    let mut receipt = world.change_input_and_conditional_definition_on_branch(
        branch,
        "combined-input",
        Arc::new(replacement),
    );
    let publication = receipt.committed_product_publication();
    assert_eq!(receipt.product_branch(), branch);
    assert_ne!(publication.composite_commit(), &before);
    assert_eq!(
        publication.relational_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(
        publication.signal_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert!(publication.signal_publication().is_some());
    assert_eq!(publication.conditional_definition_generation(), Some(2));
    assert!(receipt.dispatch_outbox().is_some());
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        primary_graph::WorthQueryExternalDispatchPostureKind::Completed
    );
    assert_eq!(transport.contact_count(), 1);
    let composite_commit = publication.composite_commit().clone();

    let performed = receipt
        .take_performed_relational_product_change()
        .expect("fresh combined publication retains one Relational delivery");
    assert_eq!(performed.product_commit(), &composite_commit);
    let selected = world.application.on_branch(branch).select().unwrap();
    let delivery = selected
        .deliver_relational_change_to_conditional(&world.clock, 0, performed)
        .expect("the performed patch and conditional handle belong to this product");
    let runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery) =
        delivery
    else {
        panic!("unexpected delivery outcome: {delivery:?}")
    };
    assert_eq!(delivery.source_envelopes_loaded(), 1);
    assert_eq!(delivery.signal_seeds_emitted(), 1);
    drop(delivery);

    world.clock_control.push(3, 11);
    let execution = world
        .application
        .on_branch(branch)
        .select()
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(execution) =
        execution
    else {
        panic!("the combined definition and truth change must be executable")
    };
    assert_eq!(
        execution.committed_operation_count(),
        1,
        "unexpected conditional execution counts: due={}, retained_due={}, eligible={}, suppressed={}, deferred={}, failed={}, indeterminate={}",
        execution.due_wake_count(),
        execution.retained_due_wake_count(),
        execution.retained_eligible_wake_count(),
        execution.retained_suppressed_wake_count(),
        execution.retained_deferred_wake_count(),
        execution.retained_failed_wake_count(),
        execution.indeterminate_operation_count(),
    );
    assert_eq!(
        execution.authoritative_commit_count(),
        1,
        "the journal advances across the explicitly delivered commit without redelivering it"
    );
    drop(execution);

    world.application.close_conditional_runtime().unwrap();
    assert_conditional_resources_empty(world.application.inspect_conditional_runtime());
}
