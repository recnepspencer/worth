pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_recovery_counters::*;

#[test]
fn mint_and_repeat_inspection_leave_recovery_work_at_zero() {
    let world = cross_gate_world("r8-33-counters");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(89);
    let aftermath = installed_notify_death_aftermath(&world.fixture.world.runtime);
    let handle = world.open_recovery(&receipt);
    assert_eq!(handle.installed_operation(), aftermath.operation_slot());
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();
    let first = world
        .fixture
        .world
        .runtime
        .inspect_commit_recovery(&handle, &specialist, action, &scope)
        .expect("inspect");
    let second = world
        .fixture
        .world
        .runtime
        .inspect_commit_recovery(&handle, &specialist, action, &scope)
        .expect("inspect again");
    for work in [first.canonical_work(), second.canonical_work()] {
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 0);
        assert_eq!(work.digest_text_materializations(), 0);
    }
}
