pub(crate) use worth_query_bank_worlds::facade::server::estate_operations::phase8_recovery_mechanism::*;

#[test]
fn notify_death_compensation_consumes_its_exact_handle_contract() {
    let compensate_world = cross_gate_world("mechanism-compensate-admit");
    compensate_world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = compensate_world.commit_notification(82);
    let handle = compensate_world.open_recovery(&receipt);
    let specialist = compensate_world.fixture.authenticate_specialist();
    let action = compensate_world.specialist_action();
    let admitted = compensate_world
        .fixture
        .world
        .runtime
        .compensate_commit_recovery(handle, &specialist, action, &request_scope())
        .expect("NotifyDeath carries a compensation mechanism");
    assert_eq!(admitted.installed_operation(), "NotifyDeathEstateOperation");
}

#[test]
fn notify_death_reconciliation_consumes_its_exact_handle_contract() {
    let reconcile_world = cross_gate_world("mechanism-reconcile-admit");
    reconcile_world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = reconcile_world.commit_notification(81);
    let handle = reconcile_world.open_recovery(&receipt);
    let specialist = reconcile_world.fixture.authenticate_specialist();
    let action = reconcile_world.specialist_action();
    let admitted = reconcile_world
        .fixture
        .world
        .runtime
        .reconcile_commit_recovery(handle, &specialist, action, &request_scope())
        .expect("NotifyDeath carries external-owner reconciliation authority");
    assert_eq!(admitted.installed_operation(), "NotifyDeathEstateOperation");
}
