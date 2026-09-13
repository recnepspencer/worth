#[path = "courtroom/clock_affinity.rs"]
mod clock_affinity;
pub use clock_affinity::{
    duplicate_reordered_and_foreign_clocks_fail_closed,
    provider_replacement_requires_fresh_runtime_publication,
};

use super::courtroom_support::{assert_authoritative_value, observe, wake_evidence};
use super::schema::{IntentEffectField, IntentLifecycleField};
use super::world::CourtroomWorld;

pub fn future_temporal_operation_waits_until_due() {
    let world = CourtroomWorld::publish("ready");
    world.clock_control.push(1, 4);
    let future = observe(&world);
    assert_eq!(
        future.committed_operation_count(),
        0,
        "{}",
        wake_evidence(&future)
    );
    assert_eq!(world.contacts.snapshot(), (0, 0, 0, 0));

    world.clock_control.push(2, 5);
    let due = observe(&world);
    assert_eq!(
        due.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&due)
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}

pub fn unrelated_rows_do_not_expand_conditional_observation_work() {
    let world = CourtroomWorld::publish_with_unrelated_rows("ready", 2_048);
    let before = world.application.inspect_conditional_runtime();
    world.clock_control.push(1, 4);

    let receipt = observe(&world);

    assert_eq!(receipt.due_wake_count(), 0);
    assert_eq!(receipt.authoritative_commit_count(), 0);
    assert!(!receipt.authoritative_work_remaining());
    assert_eq!(world.contacts.snapshot(), (0, 0, 0, 0));
    let after = world.application.inspect_conditional_runtime();
    assert_eq!(before.managed_clock_count(), 0);
    assert_eq!(before.reconstructed_intent_count(), 0);
    assert_eq!(after.managed_clock_count(), 1);
    assert_eq!(after.reconstructed_intent_count(), 1);
    assert_eq!(
        after.installed_binding_count(),
        before.installed_binding_count()
    );
    assert_eq!(after.provider_count(), before.provider_count());
    assert_eq!(after.lease_count(), before.lease_count());
    assert_eq!(after.signal_graph_count(), before.signal_graph_count());
    assert_eq!(
        after.installation_canonical_work(),
        before.installation_canonical_work()
    );
}

pub fn host_installs_and_executes_due_operation() {
    let world = CourtroomWorld::publish("ready");
    let receipt = observe(&world);
    assert_eq!(
        receipt.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&receipt)
    );
    assert_eq!(receipt.retained_due_wake_count(), 0);
    let [provenance] = receipt.execution_provenance() else {
        panic!("one committed operation must expose one typed lineage")
    };
    assert_eq!(provenance.intent_identity(), "intent-1");
    assert_eq!(provenance.intent_revision(), 1);
    assert_eq!(
        provenance.signal_decision(),
        Some(super::primary_graph::WorthQueryConditionalSignalDecision::Eligible)
    );
    assert!(provenance.application_attempt_ordinal().is_some());
    assert_eq!(
        provenance.terminal(),
        super::primary_graph::WorthQueryConditionalExecutionTerminal::Committed
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
}

pub fn temporal_identity_work_is_cold_or_fresh_admission_only() {
    let world = CourtroomWorld::publish("ready");
    let binding = world.clock.binding_canonical_work();
    assert_eq!(binding.basis_preparations(), 1);
    assert_eq!(binding.digest_derivations(), 1);
    assert_eq!(binding.canonical_entries(), 8);
    assert_eq!(binding.digest_text_materializations(), 1);

    let installation = world
        .application
        .inspect_conditional_runtime()
        .installation_canonical_work();
    assert_eq!(installation.basis_preparations(), 2);
    assert_eq!(installation.digest_derivations(), 2);
    assert_eq!(installation.canonical_entries(), 14);
    assert_eq!(installation.digest_text_materializations(), 2);

    let receipt = observe(&world);
    let [provenance] = receipt.execution_provenance() else {
        panic!("one committed operation must expose one typed lineage")
    };
    let phases = provenance.canonical_work();
    assert_eq!(phases.admission().basis_preparations(), 2);
    assert_eq!(phases.admission().digest_derivations(), 2);
    assert_eq!(phases.admission().canonical_entries(), 5);
    assert_eq!(phases.admission().digest_text_materializations(), 0);
    for work in [
        phases.installation(),
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.live_delivery(),
        phases.retry_resolution(),
        phases.recovery_inspection(),
        phases.publication(),
    ] {
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 0);
        assert_eq!(work.digest_text_materializations(), 0);
    }
}

pub fn cancellation_after_publication_retires_stale_wake() {
    let mut world = CourtroomWorld::publish("ready");
    world.amend_intent(2, "cancelled", "ready");
    let receipt = observe(&world);
    assert_eq!(
        receipt.committed_operation_count(),
        0,
        "{}",
        wake_evidence(&receipt)
    );
    assert_eq!(
        receipt.failed_operation_count(),
        0,
        "{}",
        wake_evidence(&receipt)
    );
    assert_eq!(
        receipt.retained_due_wake_count(),
        0,
        "{}",
        wake_evidence(&receipt)
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "pending".to_string(),
    );
    assert_authoritative_value(
        &world,
        IntentLifecycleField::reference(),
        "cancelled".to_string(),
    );
    assert_eq!(world.contacts.snapshot(), (0, 0, 0, 0));
}

pub fn active_successor_revision_replaces_predecessor_wake() {
    let mut world = CourtroomWorld::publish("ready");
    world.supersede_intent(2, 8, "active", "successor-payload", "ready");
    world.clock_control.push(1, 10);
    let receipt = observe(&world);
    assert_eq!(
        receipt.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&receipt)
    );
    assert_eq!(
        receipt.retained_due_wake_count(),
        0,
        "{}",
        wake_evidence(&receipt)
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "successor-payload".to_string(),
    );
    assert_authoritative_value(&world, super::schema::IntentRevisionField::reference(), 3);
    assert_eq!(world.contacts.snapshot(), (1, 1, 1, 1));
}

pub fn suppressed_wake_is_reconsidered_after_truth_change() {
    let mut world = CourtroomWorld::publish("blocked");
    let suppressed = observe(&world);
    assert_eq!(
        suppressed.committed_operation_count(),
        0,
        "{}",
        wake_evidence(&suppressed)
    );
    assert_eq!(
        suppressed.retained_suppressed_wake_count(),
        1,
        "{}",
        wake_evidence(&suppressed)
    );
    let [provenance] = suppressed.execution_provenance() else {
        panic!("the suppressed wake must retain its typed lineage")
    };
    assert_eq!(
        provenance.signal_decision(),
        Some(super::primary_graph::WorthQueryConditionalSignalDecision::Suppressed)
    );
    assert_eq!(
        provenance.terminal(),
        super::primary_graph::WorthQueryConditionalExecutionTerminal::SuppressedRetained
    );
    world.amend_intent(1, "active", "ready");
    let mut reconsidered = observe(&world);
    assert_eq!(
        reconsidered.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&reconsidered)
    );
    assert_eq!(
        reconsidered.retained_due_wake_count(),
        0,
        "{}",
        wake_evidence(&reconsidered)
    );
    let batch = reconsidered.take_granular_invalidation_batch();
    let installation = world.application.granular_invalidation_installation();
    assert!(installation.admits_batch(&batch));
    let foreign = CourtroomWorld::publish("blocked")
        .application
        .granular_invalidation_installation();
    assert!(!foreign.admits_batch(&batch));
    assert_eq!(batch.observation().direct_truth_delivery_count(), 1);
    assert_eq!(batch.observation().signal_performed_delivery_count(), 1);
    let [granular] = batch.into_bridge_deliveries().try_into().unwrap_or_else(
        |deliveries: Vec<_>| {
            panic!(
                "one exact authoritative dependency should surface one granular Bridge delivery, found {}",
                deliveries.len()
            )
        },
    );
    assert_eq!(granular.truth().change_set().changes().len(), 1);
    granular
        .performed_signal()
        .expect("the recomputed Signal node must carry its bounded performed receipt");
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}

pub fn precondition_panic_isolated_and_retry_succeeds() {
    let world = CourtroomWorld::publish("ready");
    world.preconditions_panic.set(true);
    let failed = observe(&world);
    assert_eq!(
        failed.committed_operation_count(),
        0,
        "{}",
        wake_evidence(&failed)
    );
    assert_eq!(
        failed.failed_operation_count(),
        1,
        "{}",
        wake_evidence(&failed)
    );
    world.preconditions_panic.set(false);
    let retried = observe(&world);
    assert_eq!(
        retried.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&retried)
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}

pub fn predicate_panic_does_not_corrupt_runtime_owners() {
    let world = CourtroomWorld::publish("ready");
    world.predicate_panic.set(true);
    let failed = observe(&world);
    assert_eq!(
        failed.committed_operation_count(),
        0,
        "{}",
        wake_evidence(&failed)
    );
    assert_eq!(
        failed.retained_failed_wake_count(),
        1,
        "{}",
        wake_evidence(&failed)
    );
    let [provenance] = failed.execution_provenance() else {
        panic!("the failed wake must retain its typed lineage")
    };
    assert_eq!(provenance.signal_decision(), None);
    assert_eq!(
        provenance.terminal(),
        super::primary_graph::WorthQueryConditionalExecutionTerminal::Failed
    );
    world.predicate_panic.set(false);
    let next = observe(&world);
    assert_eq!(
        next.committed_operation_count(),
        0,
        "{}",
        wake_evidence(&next)
    );
    assert_eq!(
        next.retained_failed_wake_count(),
        1,
        "{}",
        wake_evidence(&next)
    );
}
