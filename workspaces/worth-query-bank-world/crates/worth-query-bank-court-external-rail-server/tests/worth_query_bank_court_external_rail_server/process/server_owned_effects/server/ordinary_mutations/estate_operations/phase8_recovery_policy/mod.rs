pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_recovery_policy::*;

#[test]
fn expired_grant_after_mint_denies_fresh_admission_not_foreign_principal() {
    let authorization_time = std::sync::Arc::new(
        worth_query_host::facade::runtime::time::ControlledTime::at_unix_secs(300),
    );
    let world = cross_gate_world_with_clock_and_grant_validity(
        "grant-expired-after-mint",
        Some(authorization_time.clone()),
        Some(400),
    );
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt_closed = world.commit_notification(84);
    let receipt = receipt_closed
        .live_receipt()
        .expect("fresh source retains its full native owner");
    let handle = world.open_recovery(receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();
    authorization_time.set_wall_unix_nanos((401) * 1_000_000_000);
    let denied = world
        .fixture
        .world
        .runtime
        .reconcile_commit_recovery(handle, &specialist, action, &scope)
        .expect_err("expired grant must fail fresh admission");
    // The recovery is still the principal's own, so the lapse is the fresh
    // authorization's, never a recovery-handle drift. A grant past its
    // validity window is no longer selected, so no capability authorization
    // covers the action.
    let BankEstateProgressionDenial::Authorization(denial) = denied else {
        panic!("expected a fresh authorization denial, got {denied:?}");
    };
    assert_eq!(
        denial.kind(),
        BankAuthorizationDenialKind::CapabilityAuthorizationMissing
    );
}
