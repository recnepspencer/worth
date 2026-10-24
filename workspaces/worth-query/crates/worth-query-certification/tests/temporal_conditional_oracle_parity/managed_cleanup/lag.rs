//! A history longer than any former retained window cannot replace a failed
//! terminal's engine or wake. The subscription retains every position
//! (ee07da2b0e), so replacement is only the explicit reinstallation path.
use super::*;
use crate::schema::UnrelatedValueField;
use crate::world::request_scope;
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalClockObservationOutcome as Outcome, WorthQueryPrincipalResolutionMode,
};
#[test]
fn lag_reconstruction_preserves_same_native_cleanup_before_and_after_fresh_release() {
    let policy = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let mut world = CourtroomWorld::publish_with_unrelated_rows("ready", 1);
    let guard = world
        .application
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let first = observe(&execution, &world);
    let [first] = first.execution_provenance() else {
        panic!("first retained terminal");
    };
    let run = first
        .managed_cleanup_run_identity()
        .expect("actual native terminal")
        .to_owned();
    assert_eq!(
        first.cause(),
        Some(Cause::ManagedCleanupRequired(Failure::Native(
            Native::BridgeFinalization(Bridge::SignalRuntimeThreadAffinityViolation)
        )))
    );
    drop(guard);
    let branch = world.application.current_world();
    let unrelated = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    UnrelatedValueField::reference(),
                    0,
                    &request_scope(),
                    WorthQueryPrincipalResolutionMode::Certification,
                )
                .unwrap()
                .relational_record_identity_parts()
        })
        .unwrap();
    // More genuine ordinary commits than the 128-position window this court
    // once exhausted. No window remains, so none of them is lost.
    for value in 1..=130 {
        world
            .application
            .publish_native_field_write_for_test(
                branch,
                unrelated,
                UnrelatedValueField::reference(),
                value,
                &request_scope(),
            )
            .unwrap();
    }
    // Replacing the engine after that history is still refused while the
    // actual native terminal is pending.
    assert_eq!(
        world.reinstall_conditional_runtime().unwrap_err().kind(),
        InstallationDenial::ManagedCleanupRetained(Failure::Native(Native::BridgeFinalization(
            Bridge::SignalRuntimeThreadAffinityViolation
        )))
    );
    let mut port = world.conditional_clock();
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .retained_wake_count(),
        1
    );
    assert_eq!(
        (world.contacts.snapshot().2, world.contacts.snapshot().3),
        (1, 1)
    );
    world.clock_control.push(2, 11);
    let Outcome::Accepted(released) = port.observe(&execution) else {
        panic!("original Signal owner releases and heals lag");
    };
    assert_eq!(released.committed_operation_count(), 0);
    assert_eq!(released.retained_failed_wake_count(), 1);
    let [retained] = released.execution_provenance() else {
        panic!("lag replacement lost original evidence");
    };
    assert_eq!(retained.managed_cleanup_run_identity(), Some(run.as_str()));
    assert_eq!(retained.terminal(), Terminal::Indeterminate);
    assert_eq!(
        (world.contacts.snapshot().2, world.contacts.snapshot().3),
        (1, 1)
    );
    drop(port);
    world.reinstall_conditional_runtime().unwrap();
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .retained_wake_count(),
        1
    );
    assert_authoritative_value(&world, IntentEffectField::reference(), "payload".to_owned());
}
