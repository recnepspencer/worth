use super::courtroom_support::{assert_authoritative_value, observe};
use super::schema::{IntentEffectField, IntentLifecycleField};
use super::world::CourtroomWorld;

pub fn temporal_wake_post_commit_snapshot_recovery_preserves_its_product_commit(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let world = CourtroomWorld::publish("ready");
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
    let selected_commit = selected.product().selected_commit().clone();
    drop(selected);
    let integration = world
        .application
        .granular_invalidation_installation()
        .retain_primary_graph_integration_handle();
    let commits_before = integration.with_runtime(request_execution, |runtime| {
        runtime.history().immutable_commit_count()
    });
    world.application.fail_next_post_commit_snapshot_for_test();

    let committed = observe(request_execution, &world);
    assert_eq!(committed.retained_due_wake_count(), 0);
    assert_eq!(committed.committed_operation_count(), 1);
    assert_eq!(committed.already_committed_operation_count(), 0);
    assert_eq!(world.contacts.snapshot(), (1, 1, 1, 1));
    let current = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_ne!(current.product().selected_commit(), &selected_commit);
    let current_commit = current.product().selected_commit().clone();
    drop(current);
    assert_eq!(
        integration.with_runtime(request_execution, |runtime| runtime
            .history()
            .immutable_commit_count()),
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

    let settled = observe(request_execution, &world);
    assert_eq!(settled.retained_due_wake_count(), 0);
    assert_eq!(
        world.contacts.snapshot(),
        (1, 1, 1, 1),
        "post-commit recovery never repeats application behavior"
    );
    assert_eq!(
        &world
            .application
            .with_host_advancement(|phase| world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone())
            .expect("installed host call opens"),
        &current_commit
    );
    assert_eq!(
        integration.with_runtime(request_execution, |runtime| runtime
            .history()
            .immutable_commit_count()),
        commits_before + 1
    );
}
