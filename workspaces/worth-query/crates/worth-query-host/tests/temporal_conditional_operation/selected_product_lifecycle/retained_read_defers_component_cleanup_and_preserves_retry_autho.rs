//! retained read defers component cleanup and preserves retry authority.
use super::*;

#[test]
fn retained_read_defers_component_cleanup_and_preserves_retry_authority() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = CourtroomWorld::publish("blocked");
    let definitions_before = world
        .application
        .installed_conditional_definition_count_for_test();
    let targets_before = world
        .application
        .installed_conditional_target_reference_count_for_test();
    let signal_nodes_before = world.application.owned_signal_active_node_count_for_test();
    let survivor = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .expect("the independently selected survivor must publish")
        })
        .expect("installed host call opens");
    let warm = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(survivor)
                .select(&phase)
                .expect("the survivor must be selectable before sibling cleanup")
                .conditional_clock(
                    &phase
                        .checked_execution_request_for(world.application.product_runtime())
                        .expect("phase belongs to application"),
                    &world.clock,
                )
                .expect("the surviving selection must bind its managed clock")
        })
        .expect("installed host call opens")
        .observe(request_execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(warm) = warm else {
        panic!("the surviving selection must establish its predecessor cursor")
    };
    assert_eq!(warm.retained_suppressed_wake_count(), 1);
    drop(warm);
    let selected_survivor = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(survivor)
                .select(&phase)
                .expect("the survivor must remain explicitly selected through sibling cleanup")
        })
        .expect("installed host call opens");
    let branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.reuse_exact_relational_basis().fork_signal())
                .create(&phase)
                .expect("the Signal fork must publish")
        })
        .expect("installed host call opens");
    let retained = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(branch)
                .select(&phase)
                .expect("the new product must be selectable")
        })
        .expect("installed host call opens");
    for _generation in 0..2 {
        let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
        let selected = world
            .application
            .with_host_advancement(|phase| {
                world.application.on_branch(branch).select(&phase).unwrap()
            })
            .expect("installed host call opens");
        let outcome = selected
            .publish_conditional_definition(
                &world.clock,
                Arc::new(replacement),
                &runtime::RuntimeWorldCancellationSource::new().token(),
            )
            .expect("the retained branch must publish its definition successor");
        assert!(matches!(
            outcome,
            primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(_)
        ));
    }
    assert_eq!(
        world
            .application
            .installed_conditional_definition_count_for_test(),
        definitions_before + 2
    );
    assert_eq!(
        world
            .application
            .installed_conditional_target_reference_count_for_test(),
        targets_before + 2
    );
    assert_eq!(
        world.application.owned_signal_active_node_count_for_test(),
        signal_nodes_before,
        "definition replacement must preserve the branch-local Signal node population"
    );

    let WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure) = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(branch)
                .close(&phase)
                .expect_err("the selected read must retain the branch-private World history")
        })
        .expect("installed host call opens")
    else {
        panic!("the close must preserve retryable owner cleanup")
    };
    assert!(matches!(
        failure.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    assert_eq!(
        world
            .application
            .installed_conditional_definition_count_for_test(),
        definitions_before + 2,
        "history denial must preserve every definition generation for retry"
    );
    assert_eq!(
        world
            .application
            .installed_conditional_target_reference_count_for_test(),
        targets_before + 2,
        "history denial must preserve every exact Bridge target reference"
    );
    assert_eq!(
        world.application.owned_signal_active_node_count_for_test(),
        signal_nodes_before,
        "history denial must leave the live Signal node installed"
    );

    drop(failure);
    assert_eq!(world.application.branches().pending_cleanup().len(), 1);
    drop(retained);
    let mut pending = world.application.branches().pending_cleanup();
    assert_eq!(pending.len(), 1);
    let cleanup = pending.remove(0);
    let cleanup = cleanup
        .retry()
        .expect("cleanup must resume with the original owner authority");
    assert!(cleanup.is_complete());
    assert_eq!(cleanup.retired_component_count(), 1);
    assert_eq!(
        world
            .application
            .installed_conditional_definition_count_for_test(),
        definitions_before
    );
    assert_eq!(
        world
            .application
            .installed_conditional_target_reference_count_for_test(),
        targets_before
    );
    assert_eq!(
        world.application.owned_signal_active_node_count_for_test(),
        signal_nodes_before
    );
    assert!(world.application.branches().pending_cleanup().is_empty());

    let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
    let published = selected_survivor
        .publish_conditional_definition(
            &world.clock,
            Arc::new(replacement),
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the surviving selection must publish after sibling cleanup");
    let primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(published) =
        published
    else {
        panic!("the survivor must publish its replacement definition")
    };
    assert_eq!(published.product_branch(), survivor);
    assert_eq!(published.definition_generation(), 2);
    drop(published);

    let mut publication = world
        .change_input_on_branch(survivor, "ready")
        .require_committed()
        .expect("the surviving branch must publish after sibling cleanup");
    assert_eq!(publication.product_branch(), survivor);
    let change = publication
        .take_performed_relational_product_change()
        .expect("the survivor publication must carry World's performed patch");
    let selected_successor = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(survivor)
                .select(&phase)
                .expect("the survivor replacement occurrence must remain selectable")
        })
        .expect("installed host call opens");
    let delivery = world
        .application
        .with_application_advancement(&world::request_scope(), |phase| {
            selected_successor.deliver_relational_change_to_conditional(
                &phase,
                &world.clock,
                0,
                change,
            )
        })
        .expect("the declared host request admits delivery")
        .expect("the survivor patch must bind to its replacement cursor");
    let runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery) =
        delivery
    else {
        panic!("the survivor patch must reach conditional execution")
    };
    assert_eq!(delivery.source_envelopes_loaded(), 1);
    assert_eq!(delivery.signal_seeds_emitted(), 1);
    drop(delivery);

    world.clock_control.push(3, 11);
    let execution = selected_successor
        .conditional_clock(request_execution, &world.clock)
        .expect("the replacement occurrence must retain its managed clock")
        .observe(request_execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(execution) =
        execution
    else {
        panic!("the survivor replacement cursor must execute")
    };
    assert_eq!(execution.committed_operation_count(), 1);
    assert_eq!(execution.authoritative_commit_count(), 1);
}
