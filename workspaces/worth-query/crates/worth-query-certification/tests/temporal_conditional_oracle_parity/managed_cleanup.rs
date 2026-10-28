//! Real native cleanup custody survives replacement of derived conditional watches.
//! Fresh-current selection exercises cross-binding retention and reinstallation;
//! this court does not claim delivery reconsideration of the original wake.
use super::*;
use worth_query_host::facade::primary_graph::{
    BridgeExecutionBasisFinalizationFailureKind as Bridge,
    WorthQueryConditionalExecutionCause as Cause, WorthQueryManagedRunCleanupFailureKind as Native,
    WorthQueryMutationCleanupFailureKind as Failure,
};
#[test]
fn conditional_reinstallation_retains_failed_cleanup_without_repeating_business() {
    let policy = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let mut world = CourtroomWorld::publish("ready");
    let guard = world
        .application
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let first = observe(&execution, &world);
    assert_eq!(first.committed_operation_count(), 0);
    let [evidence] = first.execution_provenance() else {
        panic!("one authentic due wake");
    };
    assert_eq!(evidence.terminal(), Terminal::Indeterminate);
    assert_eq!(
        evidence.cause(),
        Some(Cause::ManagedCleanupRequired(Failure::Native(
            Native::BridgeFinalization(Bridge::SignalRuntimeThreadAffinityViolation)
        )))
    );
    let original_run = evidence
        .managed_cleanup_run_identity()
        .expect("actual pending native run")
        .to_owned();
    let contacts = world.contacts.snapshot();
    assert_eq!((contacts.2, contacts.3), (1, 1));
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
    drop(guard);
    // Reconstitution must not clear the real terminal just because its business
    // write is now visible and the reconstructed intent is no longer due.
    assert_eq!(
        world.reinstall_conditional_runtime().unwrap_err().kind(),
        InstallationDenial::ManagedCleanupRetained(Failure::Native(Native::BridgeFinalization(
            Bridge::SignalRuntimeThreadAffinityViolation
        )))
    );
    assert_eq!(
        world
            .application
            .close_conditional_runtime()
            .unwrap_err()
            .kind(),
        InstallationDenial::ManagedCleanupRetained(Failure::Native(Native::BridgeFinalization(
            Bridge::SignalRuntimeThreadAffinityViolation
        )))
    );
    world.clock_control.push(2, 11);
    let released = observe(&execution, &world);
    assert_eq!(released.committed_operation_count(), 0);
    assert_eq!(released.retained_failed_wake_count(), 1);
    // A successful reconstitution is the valid twin of the refused replacement:
    // its preparation checks that the actual pending terminal has been released.
    world.reinstall_conditional_runtime().unwrap();
    world.clock_control.push(3, 12);
    let next = observe(&execution, &world);
    assert_eq!(next.committed_operation_count(), 0);
    assert_eq!(next.retained_failed_wake_count(), 1);
    let [retained] = next.execution_provenance() else {
        panic!("reinstallation discarded cleanup custody");
    };
    assert_eq!(retained.cause(), evidence.cause());
    assert_eq!(
        retained.managed_cleanup_run_identity(),
        Some(original_run.as_str())
    );
    assert_eq!(retained.terminal(), Terminal::Indeterminate);
    assert_eq!(
        (world.contacts.snapshot().2, world.contacts.snapshot().3),
        (1, 1)
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}

#[path = "managed_cleanup/lag.rs"]
mod lag;
