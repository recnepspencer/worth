use primary_graph::{ProductUnpublishedNextAction, RuntimeWorldRecoveryDenial};
use std::{num::NonZeroUsize, sync::Arc};
use worth_query_host::facade::primary_graph;

use super::adapters::CompletingExternalTransport;
use super::courtroom_support::{assert_authoritative_value, observe};
use super::schema::{IntentEffectField, IntentLifecycleField};
use super::world::CourtroomWorld;

pub fn temporal_wake_settlement_repair_keeps_the_original_product_unpublished() {
    let mut world = CourtroomWorld::publish("ready");
    let transport = Arc::new(CompletingExternalTransport::default());
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let selected = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let selected_commit = selected.product().selected_commit().clone();
    drop(selected);
    let integration = world
        .application
        .granular_invalidation_installation()
        .retain_primary_graph_integration_handle();
    let commits_before =
        integration.with_runtime(|runtime| runtime.history().immutable_commit_count());
    integration
        .execute_mutation_with_index_refresh(|runtime| {
            runtime.fail_next_durable_append_for_test();
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();

    let partial = observe(&mut world);
    assert_eq!(partial.committed_operation_count(), 0);
    assert_eq!(partial.already_committed_operation_count(), 0);
    assert_eq!(partial.retained_due_wake_count(), 1);
    assert_eq!(partial.retained_failed_wake_count(), 1);
    assert_eq!(world.contacts.snapshot(), (1, 1, 1, 1));
    assert_eq!(
        transport.contact_count(),
        0,
        "owner-local outbox state cannot dispatch without a composite product commit"
    );
    let [provenance] = partial.execution_provenance() else {
        panic!("one partial wake must expose one lineage")
    };
    assert_eq!(
        provenance.terminal(),
        primary_graph::WorthQueryConditionalExecutionTerminal::ProductUnpublished
    );
    assert_eq!(
        integration.with_runtime(|runtime| runtime.history().immutable_commit_count()),
        commits_before + 1
    );
    let page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap();
    let [row] = page.rows() else {
        panic!("World must retain the temporal operation's owner effects")
    };
    let recovery = world
        .application
        .readmit_product_publication_recovery(row.handle())
        .unwrap();
    let failure = world
        .application
        .release_product_publication_recovery(recovery, 0)
        .expect_err("owner settlement remains required");
    assert!(matches!(
        &failure,
        primary_graph::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure)
            if failure.denial()
                == primary_graph::WorthQueryProductUnpublishedRecoveryReleaseDenial::World(
                    RuntimeWorldRecoveryDenial::SettlementRequired,
                )
    ));
    let recovery = failure
        .into_recovery()
        .expect("a World recovery denial preserves the exact recovery capability");
    let actions = recovery.continue_owner_settlement().unwrap();
    assert!(!actions
        .actions()
        .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
    assert!(!recovery.inspect().unwrap().relational_requires_settlement());

    let retained = observe(&mut world);
    assert_eq!(retained.retained_due_wake_count(), 1);
    assert_eq!(retained.committed_operation_count(), 0);
    assert_eq!(retained.already_committed_operation_count(), 0);
    assert_eq!(
        world.contacts.snapshot(),
        (1, 1, 1, 1),
        "owner settlement cannot re-enter application behavior"
    );
    let [provenance] = retained.execution_provenance() else {
        panic!("unpublished wake remains inspectable")
    };
    assert_eq!(
        provenance.terminal(),
        primary_graph::WorthQueryConditionalExecutionTerminal::ProductUnpublished
    );
    let current = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    assert_eq!(
        current.product().selected_commit(),
        &selected_commit,
        "settlement cannot fabricate a product occurrence"
    );
    assert_eq!(
        integration.with_runtime(|runtime| runtime.history().immutable_commit_count()),
        commits_before + 1
    );
    let cleanup = world
        .application
        .release_product_publication_recovery(recovery, 0)
        .unwrap();
    assert_eq!(cleanup.retired_component_count(), 0);
    let lifetime = world.application.conditional_runtime_lifecycle_probe();
    world.application.close_conditional_runtime().unwrap();
    super::courtroom_lifecycle::assert_conditional_resources_empty(
        world.application.inspect_conditional_runtime(),
    );
    let denial = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap()
        .conditional_clock(&world.clock)
        .err()
        .expect("close revokes the caller's retained clock handle");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalClockObservationDenialKind::ForeignRuntime
    );
    drop(world.clock);
    super::courtroom_lifecycle::assert_conditional_resources_empty(lifetime.live_inventory());
    let after_close = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    assert_eq!(after_close.product().selected_commit(), &selected_commit);
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(transport.contact_count(), 0);
}

pub fn temporal_wake_post_performed_index_repair_preserves_its_product_commit() {
    let mut world = CourtroomWorld::publish("ready");
    let selected = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let selected_commit = selected.product().selected_commit().clone();
    drop(selected);
    let integration = world
        .application
        .granular_invalidation_installation()
        .retain_primary_graph_integration_handle();
    let commits_before =
        integration.with_runtime(|runtime| runtime.history().immutable_commit_count());
    world.application.fail_next_index_publication_for_test();

    let committed = observe(&mut world);
    assert_eq!(committed.retained_due_wake_count(), 0);
    assert_eq!(committed.committed_operation_count(), 1);
    assert_eq!(committed.already_committed_operation_count(), 0);
    assert_eq!(world.contacts.snapshot(), (1, 1, 1, 1));
    let current = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    assert_ne!(current.product().selected_commit(), &selected_commit);
    let current_commit = current.product().selected_commit().clone();
    drop(current);
    assert_eq!(
        integration.with_runtime(|runtime| runtime.history().immutable_commit_count()),
        commits_before + 1
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
    assert_authoritative_value(
        &world,
        IntentLifecycleField::reference(),
        "completed".to_string(),
    );

    let settled = observe(&mut world);
    assert_eq!(settled.retained_due_wake_count(), 0);
    assert_eq!(
        world.contacts.snapshot(),
        (1, 1, 1, 1),
        "projection repair never repeats application behavior"
    );
    assert_eq!(
        world
            .application
            .on_branch(world.application.current_world())
            .select()
            .unwrap()
            .product()
            .selected_commit(),
        &current_commit
    );
    assert_eq!(
        integration.with_runtime(|runtime| runtime.history().immutable_commit_count()),
        commits_before + 1
    );
}
