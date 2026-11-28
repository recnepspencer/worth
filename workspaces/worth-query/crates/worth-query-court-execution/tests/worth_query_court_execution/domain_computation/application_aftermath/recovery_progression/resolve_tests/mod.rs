//! Recovery resolution accepts only an admitted read for its own key.

use super::fixture::{outboxless_handle, world};
use worth_query_execution::facade::domain_computation::{
    WorthQueryApplicationIdempotencyResolution, WorthQueryRecoveryHandleDenialKind,
    resolve_recovery_handle,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;

#[test]
fn foreign_admitted_idempotency_read_denies_distinctly() {
    let world = world();
    let (handle, admission) = outboxless_handle(&world);
    let registry = handle.registry_arc_for_test();
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .unwrap();

    let denied = world
        .application
        .with_host_advancement(|phase| {
            let foreign_read = world
                .application
                .resolve_admitted_application_idempotency(
                    &phase,
                    &admission,
                    idempotency(0xA1, 0xA2),
                )
                .unwrap();
            assert!(matches!(
                foreign_read.resolution(),
                WorthQueryApplicationIdempotencyResolution::Unseen
            ));
            resolve_recovery_handle(&phase, handle, &authority, foreign_read)
        })
        .unwrap()
        .expect_err("a read of a different key cannot resolve this handle");

    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::ForeignIdempotencyRead
    );
    registry.assert_no_live_handles_for_test();
}

#[test]
fn matching_admitted_idempotency_read_resolves() {
    let world = world();
    let (handle, admission) = outboxless_handle(&world);
    let registry = handle.registry_arc_for_test();
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .unwrap();

    let resolution = world
        .application
        .with_host_advancement(|phase| {
            let read = world
                .application
                .resolve_admitted_application_idempotency(
                    &phase,
                    &admission,
                    handle.binding().idempotency(),
                )
                .unwrap();
            resolve_recovery_handle(&phase, handle, &authority, read)
        })
        .unwrap()
        .expect("a read of the matching commit resolves the handle");

    assert!(
        matches!(
            resolution,
            WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(_)
        ),
        "the matching key resolves the actual commit: {resolution:?}",
    );
    registry.assert_no_live_handles_for_test();
}
