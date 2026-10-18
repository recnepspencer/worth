use std::sync::Arc;
use std::time::Duration;

use bank_domain::estate::DeathNoticeStatus;
use bank_external_rail::test_control::FaultScript;
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

    let outcome = {
        let source_actor = &fixture.authenticate_specialist();
        let source_scope = &request_scope();
        let source_key = &idempotency(97);
        let source_intent = bank_domain::schema::NotifyEstateDeath::try_from(
            fixture.action(fixture.notice, fixture.deceased),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            source_completed,
        ) = source_operation.advance(&source_request)
        else {
            panic!("native source completion required")
        };
        assert!(matches!(
            source_completed.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        let closed_source = source_completed
            .close_with_evidence(&source_request)
            .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
        bank_server::BankCommitReceipt::from_closed_source(closed_source)
    };
    let receipt = outcome;

    assert_eq!(transport.production_dispatches().len(), 1);
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .and_then(|posture| posture.failure()),
        Some(WorthQueryPublishedExternalEffectFailure::LostResponse)
    );
}
