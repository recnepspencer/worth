//! C4 final-source retry proof through Bank and its separate external rail.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

use bank_domain::estate::DeathNoticeStatus;
use bank_external_rail::test_control::FaultScript;

use super::{
    external_effect_dispatch::publication_assertions,
    phase8_cross_gate::world::{
        cross_gate_world, cross_gate_world_with_clock_and_grant_validity, idempotency, PATIENT,
    },
};
use crate::{authorization_time::AuthorizationTimeController, support::request_scope};

#[test]
fn ordinary_equivalent_retry_skips_materialization_and_the_external_rail() {
    let world = cross_gate_world("c4-response-loss-retry");
    let binding = idempotency(151);
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_with(binding.clone());
    assert_eq!(
        world.notice_status(),
        DeathNoticeStatus::NotificationRequested
    );
    let correlation = world.transport.attempts()[0].clone();
    let completed = world
        .transport
        .completed_notice(&correlation)
        .expect("the separate rail completed the response-lost notification");
    assert_eq!(completed.estate(), world.fixture.estate.get());
    assert_eq!(completed.notice(), world.fixture.notice.get());
    assert_eq!(completed.subject(), world.fixture.deceased.get());

    let dispatch_observed = Arc::new(AtomicBool::new(false));
    let armed = dispatch_observed.clone();
    world
        .transport
        .after_next_dispatch(move || armed.store(true, Ordering::Release));
    let retry = {
        let source_actor = &world.fixture.authenticate_specialist();
        let source_scope = &request_scope();
        let source_key = &binding;
        let source_intent =
            bank_domain::schema::NotifyEstateDeath::try_from(world.specialist_action())
                .expect("typed estate command preserves the exact variant");
        let source_request = world
            .fixture
            .world
            .runtime
            .request(source_actor, source_scope);
        let source_operation = world
            .fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            source_completed,
        ) = source_operation.advance(&source_request)
        else {
            panic!("native source completion required")
        };
        assert!(matches!(
            source_completed.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Recovered { .. }
        ));
        let closed_source = source_completed
            .close_with_evidence(&source_request)
            .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
        bank_server::BankCommitReceipt::from_closed_source(closed_source)
    };
    let recovered = retry;
    publication_assertions::assert_recovered_commit_axes(&receipt, &recovered);
    assert_eq!(world.transport.attempts().len(), 1);
    assert!(!dispatch_observed.load(Ordering::Acquire));
    assert_eq!(
        world.notice_status(),
        DeathNoticeStatus::NotificationRequested
    );

    world.transport.under(FaultScript::Succeed, PATIENT);
    let second = world.commit_second_notification(152);
    assert_eq!(second.emitted_effect_count(), 1);
    assert!(dispatch_observed.load(Ordering::Acquire));
    assert_eq!(world.transport.attempts().len(), 2);
    assert_eq!(
        world
            .transport
            .completed_notice(&correlation)
            .expect("retry cannot rewrite the rail's completed notice"),
        completed
    );
}

#[test]
fn equivalent_binding_does_not_bypass_fresh_authorization_currentness() {
    let time = AuthorizationTimeController::at_epoch_seconds(300);
    let world = cross_gate_world_with_clock_and_grant_validity(
        "c4-currentness",
        Some(time.clone()),
        Some(400),
    );
    let binding = idempotency(153);
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let receipt = world.commit_with(binding.clone());
    time.advance_to_epoch_seconds(401);

    let (_denied_source_custody, denied) = {
        let source_actor = &world.fixture.authenticate_specialist();
        let source_scope = &request_scope();
        let source_key = &binding;
        let source_intent =
            bank_domain::schema::NotifyEstateDeath::try_from(world.specialist_action())
                .expect("typed estate command preserves the exact variant");
        let source_request = world
            .fixture
            .world
            .runtime
            .request(source_actor, source_scope);
        let source_operation = world
            .fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused {
            operation: source_custody,
            denial:
                worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(cause),
        } = source_operation.advance(&source_request)
        else {
            panic!("fresh native authorization refusal")
        };
        (source_custody, cause)
    };
    assert!(matches!(
        denied,
        ref denial
            if matches!(
                denial.kind(),
                worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::Authorization(
                    worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::CapabilityAuthorizationMissing)
                )
            )
    ));
    assert_eq!(world.transport.attempts().len(), 1);
    assert_eq!(
        world.notice_status(),
        DeathNoticeStatus::NotificationRequested
    );
    assert!(receipt.co_committed_dispatch_outbox());
}
