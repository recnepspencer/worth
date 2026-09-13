use std::sync::Arc;

use worth_query_host::facade::product::{
    WorthQueryApplicationProductBranchCleanupDenial, WorthQueryApplicationProductBranchCloseDenial,
    WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchCloseDenial,
    WorthQueryProductBranchOwnerCleanupDenial,
};
use worth_query_host::facade::{primary_graph, runtime};

use super::adapters::ReplacementPredicate;
use super::world::CourtroomWorld;

#[test]
fn public_transaction_reuses_its_admitted_selection_at_snapshot_capacity() {
    let world = CourtroomWorld::publish_with_active_snapshot_limit("ready", 4);
    let branch = world.application.current_world();
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();

    let committed = world
        .change_input_on_branch(branch, "single-admitted-selection")
        .require_committed()
        .expect("the carried selection must commit without opening another snapshot");

    assert_eq!(committed.product_branch(), branch);
    assert_eq!(
        observer.observe().acquisitions() - before.acquisitions(),
        4,
        "the admitted mutation path must not add a fifth basis acquisition by reselecting at commit"
    );
}

#[test]
fn foreign_and_retired_tokens_fail_before_query_basis_work() {
    let world = CourtroomWorld::publish("ready");
    let foreign = CourtroomWorld::publish("ready");
    let observer = world.application.application_query_basis_observer();

    let before_foreign = observer.observe();
    let foreign_denial = world
        .application
        .on_branch(foreign.application.current_world())
        .select()
        .err()
        .expect("a branch token from another World owner must be refused");
    assert_eq!(
        foreign_denial,
        WorthQueryProductBranchAdmissionDenial::ForeignOwner
    );
    assert_eq!(
        observer.observe().acquisitions(),
        before_foreign.acquisitions(),
        "foreign-token denial must precede Query basis acquisition"
    );

    let source = world.application.current_world();
    let retired = world
        .application
        .branches()
        .fork(source)
        .components(|components| {
            components
                .reuse_exact_relational_basis()
                .reuse_exact_signal_basis()
        })
        .create()
        .expect("the owned reuse branch must publish");
    let close = world
        .application
        .on_branch(retired)
        .close()
        .expect("an unretained public product can close");
    assert!(close.is_complete());

    let before_retired = observer.observe();
    let retired_denial = world
        .application
        .on_branch(retired)
        .select()
        .err()
        .expect("a retired occurrence token must be refused");
    assert_eq!(
        retired_denial,
        WorthQueryProductBranchAdmissionDenial::RetiredBranch
    );
    assert_eq!(
        observer.observe().acquisitions(),
        before_retired.acquisitions(),
        "retired-token denial must precede Query basis acquisition"
    );
    assert!(matches!(
        world.application.on_branch(retired).close().unwrap_err(),
        WorthQueryApplicationProductBranchCloseDenial::Product(
            WorthQueryProductBranchCloseDenial::Selection(
                WorthQueryProductBranchAdmissionDenial::RetiredBranch
            )
        )
    ));
}

#[test]
fn retained_read_defers_component_cleanup_and_preserves_retry_authority() {
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
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the independently selected survivor must publish");
    let warm = world
        .application
        .on_branch(survivor)
        .select()
        .expect("the survivor must be selectable before sibling cleanup")
        .conditional_clock(&world.clock)
        .expect("the surviving selection must bind its managed clock")
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(warm) = warm else {
        panic!("the surviving selection must establish its predecessor cursor")
    };
    assert_eq!(warm.retained_suppressed_wake_count(), 1);
    drop(warm);
    let selected_survivor = world
        .application
        .on_branch(survivor)
        .select()
        .expect("the survivor must remain explicitly selected through sibling cleanup");
    let branch = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .expect("the Signal fork must publish");
    let retained = world
        .application
        .on_branch(branch)
        .select()
        .expect("the new product must be selectable");
    for _generation in 0..2 {
        let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
        let selected = world.application.on_branch(branch).select().unwrap();
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
        .on_branch(branch)
        .close()
        .expect_err("the selected read must retain the branch-private World history")
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
        .on_branch(survivor)
        .select()
        .expect("the survivor replacement occurrence must remain selectable");
    let delivery = selected_successor
        .deliver_relational_change_to_conditional(&world.clock, 0, change)
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
        .conditional_clock(&world.clock)
        .expect("the replacement occurrence must retain its managed clock")
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(execution) =
        execution
    else {
        panic!("the survivor replacement cursor must execute")
    };
    assert_eq!(execution.committed_operation_count(), 1);
    assert_eq!(execution.authoritative_commit_count(), 1);
}

#[test]
fn reuse_only_descendant_reclaims_its_history_before_parent_component_cleanup() {
    let world = CourtroomWorld::publish("ready");
    let parent = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the parent Relational fork must publish");
    let child = world
        .application
        .branches()
        .fork(parent)
        .components(|components| {
            components
                .reuse_exact_relational_basis()
                .reuse_exact_signal_basis()
        })
        .create()
        .expect("the reuse-only child must publish");
    let committed = world
        .change_input_on_branch(child, "reuse-descendant-change")
        .require_committed()
        .expect("the child must install a branch-local World descendant");
    let WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure) = world
        .application
        .on_branch(child)
        .close()
        .expect_err("the caller-held publication receipt must retain its World history")
    else {
        panic!("the close must preserve retryable history cleanup")
    };
    assert!(matches!(
        failure.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    drop(committed);
    let child_close = failure
        .into_cleanup()
        .retry()
        .expect("internal evidence was retired, so releasing the caller receipt completes cleanup");
    assert_eq!(child_close.retired_component_count(), 0);
    let parent_close = world
        .application
        .on_branch(parent)
        .close()
        .expect("the parent must no longer be retained by the closed child");
    assert_eq!(parent_close.retired_component_count(), 1);
    assert!(world.application.branches().pending_cleanup().is_empty());
}
