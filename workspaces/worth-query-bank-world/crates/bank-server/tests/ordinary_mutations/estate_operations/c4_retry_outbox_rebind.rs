//! C4 final-source retry proof through Bank and its separate external rail.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bank_domain::estate::DeathNoticeStatus;
use bank_external_rail::test_control::FaultScript;
use bank_server::{BankEstateProgressionDenial, BankMutationCommitOutcome};

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
    let retry = world
        .fixture
        .world
        .runtime
        .notify_estate_death_with_key(
            &world.fixture.authenticate_specialist(),
            world.specialist_action(),
            &binding,
            &request_scope(),
        )
        .expect("freshly authorized equivalent retry reaches retained resolution");
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the poststate must resolve before materialization: {retry:?}");
    };
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

    let denied = world
        .fixture
        .world
        .runtime
        .notify_estate_death_with_key(
            &world.fixture.authenticate_specialist(),
            world.specialist_action(),
            &binding,
            &request_scope(),
        )
        .expect_err("expired current authority must precede retained resolution");
    assert!(matches!(
        denied,
        BankEstateProgressionDenial::ApplicationEntry(ref denial)
            if denial.kind()
                == worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::Authorization
    ));
    assert_eq!(world.transport.attempts().len(), 1);
    assert_eq!(
        world.notice_status(),
        DeathNoticeStatus::NotificationRequested
    );
    assert!(receipt.co_committed_dispatch_outbox());
}
