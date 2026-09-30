//! Fresh admission policy distinct from binding-axis drift (Gate 8.3 turn 3).
//!
//! `ForeignPrincipal` is proved per-axis in `worth-query-execution`
//! `binding_axis_tests` — the default notify-death fixture only authorizes one
//! specialist principal on recovery re-admission.

use bank_external_rail::test_control::FaultScript;
use bank_server::{BankAuthorizationDenialKind, BankEstateProgressionDenial};

use super::phase8_cross_gate::world::{cross_gate_world_with_clock_and_grant_validity, PATIENT};
use crate::authorization_time::AuthorizationTimeController;
use crate::support::request_scope;

#[test]
fn expired_grant_after_mint_denies_fresh_admission_not_foreign_principal() {
    let authorization_time = AuthorizationTimeController::at_epoch_seconds(300);
    let world = cross_gate_world_with_clock_and_grant_validity(
        "grant-expired-after-mint",
        Some(authorization_time.clone()),
        Some(400),
    );
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_notification(84);
    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();
    authorization_time.advance_to_epoch_seconds(401);
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
