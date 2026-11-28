//! Installed correction mechanism and authority refusals.

use super::fixture::{outboxless_handle, world};
use worth_query_execution::facade::domain_computation::{
    compensate_recovery_handle, reconcile_recovery_handle, WorthQueryRecoveryHandleDenialKind,
};

#[test]
fn recorded_inverse_handle_denies_compensation_from_its_own_mechanism() {
    let world = world();
    let (handle, admission) = outboxless_handle(&world);
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .unwrap();
    let denied = world
        .application
        .with_host_advancement(|phase| compensate_recovery_handle(&phase, handle, &authority))
        .unwrap()
        .expect_err("recorded inverse cannot be used for compensation");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::CompensationNotAdmitted
    );
}

#[test]
fn runtime_alone_handle_denies_reconciliation_from_its_own_authority() {
    let world = world();
    let (handle, admission) = outboxless_handle(&world);
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .unwrap();
    let denied = world
        .application
        .with_host_advancement(|phase| reconcile_recovery_handle(&phase, handle, &authority))
        .unwrap()
        .expect_err("runtime-alone authority cannot be used for reconciliation");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::ReconciliationNotAdmitted
    );
}
