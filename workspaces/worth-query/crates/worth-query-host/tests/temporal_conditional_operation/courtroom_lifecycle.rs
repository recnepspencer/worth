use worth_query_host::facade::primary_graph;

use super::courtroom_support::{assert_authoritative_value, observe, wake_evidence};
use super::schema::IntentEffectField;
use super::world::CourtroomWorld;

pub fn reinstallation_reconstructs_active_authoritative_work() {
    let mut world = CourtroomWorld::publish("ready");
    let before = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let receipt = world.reinstall_conditional_runtime().unwrap();
    let lower = receipt.lower_runtime_reconstitution();
    let after = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_eq!(before.branch_identity(), after.branch_identity());
    assert_eq!(before.selected_commit(), after.selected_commit());
    assert_eq!(
        before.relational_basis_descriptor(),
        after.relational_basis_descriptor()
    );
    assert_eq!(lower.signal().service_readmission_count(), 1);
    assert_eq!(lower.readmitted_lowering_count(), 1);
    assert!(lower
        .correspondence()
        .exact_semantic_dependency_index_parity());
    assert!(lower.correspondence().exact_mapping_index_parity());
    assert!(lower.correspondence().exact_index_parity());
    assert_eq!(receipt.reconstructed_binding_count(), 1);
    assert_eq!(receipt.reconstructed_intent_count(), 1);
    let observed = observe(&mut world);
    assert_eq!(
        observed.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&observed)
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}

pub fn reconstruction_work_ignores_unrelated_rows() {
    let mut baseline = CourtroomWorld::publish("ready");
    let baseline = baseline.reinstall_conditional_runtime().unwrap();
    let mut expanded = CourtroomWorld::publish_with_unrelated_rows("ready", 2_048);
    let expanded = expanded.reinstall_conditional_runtime().unwrap();
    assert!(baseline.total_work_units() > 0);
    assert_eq!(
        baseline.examined_candidate_count(),
        expanded.examined_candidate_count()
    );
    assert_eq!(
        baseline.projected_record_count(),
        expanded.projected_record_count()
    );
    assert_eq!(
        baseline.projected_field_count(),
        expanded.projected_field_count()
    );
    assert_eq!(baseline.total_work_units(), expanded.total_work_units());
}

pub fn reinstallation_restores_no_terminal_work() {
    let mut cancelled = CourtroomWorld::publish("ready");
    cancelled.amend_intent(2, "cancelled", "ready");
    cancelled.reinstall_conditional_runtime().unwrap();
    let receipt = observe(&mut cancelled);
    assert_eq!(receipt.committed_operation_count(), 0);
    assert_eq!(cancelled.contacts.snapshot(), (0, 0, 0, 0));

    let mut completed = CourtroomWorld::publish("ready");
    let _ = observe(&mut completed);
    let contacts = completed.contacts.snapshot();
    completed.reinstall_conditional_runtime().unwrap();
    let receipt = observe(&mut completed);
    assert_eq!(receipt.committed_operation_count(), 1);
    assert_eq!(completed.contacts.snapshot(), contacts);
}

pub fn reinstallation_after_eligibility_retries_freshly() {
    let mut world = CourtroomWorld::publish("ready");
    world.preconditions_panic.set(true);
    let failed = observe(&mut world);
    assert_eq!(
        failed.failed_operation_count(),
        1,
        "{}",
        wake_evidence(&failed)
    );
    world.preconditions_panic.set(false);
    world.reinstall_conditional_runtime().unwrap();
    let retried = observe(&mut world);
    assert_eq!(
        retried.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&retried)
    );
}

pub fn reinstallation_after_commit_cannot_duplicate_effect() {
    let mut world = CourtroomWorld::publish("ready");
    assert_eq!(observe(&mut world).committed_operation_count(), 1);
    let contacts = world.contacts.snapshot();
    world.reinstall_conditional_runtime().unwrap();
    let second = observe(&mut world);
    assert_eq!(second.committed_operation_count(), 1);
    assert_eq!(second.already_committed_operation_count(), 0);
    assert_eq!(world.contacts.snapshot(), contacts);
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}

pub fn reinstallation_revokes_captured_granular_batches() {
    let mut world = CourtroomWorld::publish("blocked");
    let before = world.application.granular_invalidation_installation();
    world.amend_intent(1, "active", "ready");
    let mut observed = observe(&mut world);
    let batch = observed.take_granular_invalidation_batch();
    assert!(before.admits_batch(&batch));

    world.reinstall_conditional_runtime().unwrap();
    let current = world.application.granular_invalidation_installation();
    assert!(!before.admits_batch(&batch));
    assert!(!current.admits_batch(&batch));
}

pub fn closing_runtime_releases_inventory_and_revokes_handles() {
    let mut world = CourtroomWorld::publish("blocked");
    let _ = observe(&mut world);
    let before = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let installed = world.application.inspect_conditional_runtime();
    assert_eq!(installed.installed_binding_count(), 1);
    assert_eq!(installed.managed_clock_count(), 1);
    assert_eq!(installed.reconstructed_intent_count(), 1);
    assert_eq!(installed.retained_wake_count(), 1);
    assert_eq!(
        world.application.close_conditional_runtime().unwrap(),
        installed
    );
    let empty = world.application.inspect_conditional_runtime();
    assert_conditional_resources_empty(empty);
    assert_eq!(empty.signal_graph_count(), installed.signal_graph_count());
    let after = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_eq!(before.branch_identity(), after.branch_identity());
    assert_eq!(before.selected_commit(), after.selected_commit());
    assert_eq!(
        before.relational_basis_descriptor(),
        after.relational_basis_descriptor()
    );
    let denial = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .unwrap()
        .conditional_clock(&world.clock)
        .err()
        .expect("closed clock must be rejected");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalClockObservationDenialKind::ForeignRuntime
    );
}

pub fn dropping_runtime_releases_exact_inventory() {
    let mut world = CourtroomWorld::publish("ready");
    world.preconditions_panic.set(true);
    let failed = observe(&mut world);
    assert_eq!(failed.failed_operation_count(), 1);
    let probe = world.application.conditional_runtime_lifecycle_probe();
    let live = probe.live_inventory();
    assert_eq!(live.installed_binding_count(), 1);
    assert_eq!(live.provider_count(), 1);
    assert_eq!(live.managed_clock_count(), 1);
    assert_eq!(live.retained_wake_count(), 1);
    assert_eq!(live.reconstructed_intent_count(), 1);
    assert_eq!(live.retained_attempt_count(), 1);
    assert_eq!(live.lease_count(), 1);
    assert_eq!(live.signal_graph_count(), 1);
    drop(world);
    assert_empty(probe.live_inventory());
}

fn assert_empty(empty: primary_graph::WorthQueryConditionalRuntimeInspection) {
    assert!(
        empty.is_empty(),
        "live conditional inventory after drop: {empty:?}"
    );
    assert_eq!(empty.signal_graph_count(), 0);
    assert_conditional_resources_empty(empty);
}

pub(super) fn assert_conditional_resources_empty(
    empty: primary_graph::WorthQueryConditionalRuntimeInspection,
) {
    assert_eq!(empty.installed_binding_count(), 0);
    assert_eq!(empty.provider_count(), 0);
    assert_eq!(empty.managed_clock_count(), 0);
    assert_eq!(empty.retained_wake_count(), 0);
    assert_eq!(empty.reconstructed_intent_count(), 0);
    assert_eq!(empty.scheduler_task_count(), 0);
    assert_eq!(empty.scheduler_queue_count(), 0);
    assert_eq!(empty.retained_attempt_count(), 0);
    assert_eq!(empty.lease_count(), 0);
}
