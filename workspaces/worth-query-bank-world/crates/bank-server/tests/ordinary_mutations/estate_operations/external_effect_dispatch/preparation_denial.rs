use std::sync::Arc;
use std::time::Duration;

use bank_domain::estate::DeathNoticeStatus;
use bank_external_rail::test_control::FaultScript;
use bank_server::BankMutationCommitOutcome;
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectFailure;

use super::{idempotency, request_scope, spawn_rail, BankEstateRailTransport};
use crate::authorization_time::AuthorizationTimeController;
use crate::estate_operations::notify_death::fixture::notification_world_with_authorization_time;

#[test]
fn late_runtime_time_failure_preserves_the_observed_lost_response() {
    let rail = spawn_rail();
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    transport.under(FaultScript::CommitThenLoseResponse, Duration::from_secs(5));
    let time = AuthorizationTimeController::at_epoch_seconds(7_000);
    let fixture = notification_world_with_authorization_time(
        "external-dispatch-time-denial",
        DeathNoticeStatus::Reported,
        Some(time.clone()),
    );
    fixture
        .world
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("rail transport installs");
    transport.after_next_dispatch(move || time.make_unavailable());

    let outcome = fixture
        .world
        .runtime
        .notify_estate_death_with_key(
            &fixture.authenticate_specialist(),
            fixture.action(fixture.notice, fixture.deceased),
            &idempotency(97),
            &request_scope(),
        )
        .expect("lawful notification commits before dispatch classification");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("notification must remain committed: {outcome:?}")
    };

    assert_eq!(transport.production_dispatches().len(), 1);
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .and_then(|posture| posture.failure()),
        Some(WorthQueryPublishedExternalEffectFailure::LostResponse)
    );
}
