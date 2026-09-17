//! Gate 8.7 — safe-retry re-dispatch through the real bank-external-rail (§11 rows 16–18).
//!
//! Authority precedes every transport call. Exactly-once is observed through
//! the rail's admission ledger and independent physical consequence owner,
//! not by Query declining to ask (R8.70).

use bank_external_rail::test_control::FaultScript;
use bank_external_rail::LedgerStatus;
use bank_server::{
    BankEstateProgressionDenial, BankRecoveryClaimStatus, BankRecoveryDenialKind,
    BankRecoveryDurability,
};
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectPostureKind;

use super::phase8_cross_gate::world;
use crate::authorization_time::AuthorizationTimeController;
use crate::support::request_scope;

#[test]
fn faulted_dispatch_then_safe_retry_escapes_exactly_once() {
    let world = world::cross_gate_world("safe-retry-escape");
    world
        .transport
        .under(FaultScript::DisappearMidDispatch, world::PATIENT);
    let receipt = world.commit_notification(81);
    assert!(receipt.co_committed_dispatch_outbox());
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Unresolved)
    );
    assert_eq!(world.transport.admission_count(), 0);
    let correlation = world.transport.attempts()[0].clone();

    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();

    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let admission = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(handle, &specialist, action, &scope)
        .expect("safe-retry through production admission");
    assert!(admission.is_external_completion());
    assert!(admission.has_fresh_attempt());
    assert_eq!(
        world
            .fixture
            .world
            .runtime
            .commit_recovery_claim_status(&receipt),
        Ok(BankRecoveryClaimStatus::Completed)
    );
    assert_eq!(
        admission.durability(),
        BankRecoveryDurability::StoreCapabilityRequired
    );
    assert_eq!(
        world.transport.ledger_status(&correlation),
        LedgerStatus::Completed
    );
    assert_eq!(
        world.transport.admission_count(),
        1,
        "exactly one rail admission across faulted dispatch and safe-retry"
    );
    assert_request_affine_retry_completed_once(&world, &correlation);
}

#[test]
fn lost_response_safe_retry_reissues_same_request_and_completes_once() {
    // §11 row 16 under genuine indeterminacy: the rail admitted, Query holds
    // Unresolved, and retry must not produce a second admission (row 5 shape).
    let world = world::cross_gate_world("safe-retry-lost-response");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, world::PATIENT);
    let receipt = world.commit_notification(86);
    assert!(receipt.co_committed_dispatch_outbox());
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Unresolved)
    );
    assert_eq!(
        world.transport.admission_count(),
        1,
        "CommitThenLoseResponse admits at the rail before losing the response"
    );
    let correlation = world.transport.attempts()[0].clone();
    assert_eq!(
        world.transport.ledger_status(&correlation),
        LedgerStatus::Completed
    );

    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();

    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let admission = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(handle, &specialist, action, &scope)
        .expect("safe-retry under unresolved lost-response posture");
    assert!(admission.is_external_completion());
    assert!(admission.has_fresh_attempt());
    assert_eq!(
        world.transport.ledger_status(&correlation),
        LedgerStatus::Completed
    );
    assert_eq!(
        world.transport.admission_count(),
        1,
        "indeterminate lost-response retry must not admit a second rail attempt"
    );
    assert_request_affine_retry_completed_once(&world, &correlation);

    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let _unrelated = world.commit_second_notification(87);
    assert_eq!(world.transport.production_dispatches().len(), 3);
    assert_eq!(world.transport.completed_effect_count(), 2);
}

fn assert_request_affine_retry_completed_once(
    world: &world::CrossGateWorld,
    correlation: &bank_external_rail::RailCorrelation,
) {
    let dispatches = world.transport.production_dispatches();
    assert_eq!(
        dispatches.len(),
        2,
        "initial and retry must both cross the production Bank adapter"
    );
    assert_eq!(dispatches[0].correlation, dispatches[1].correlation);
    assert_eq!(dispatches[0].payload, dispatches[1].payload);
    assert_eq!(&dispatches[0].correlation, correlation);
    assert_eq!(
        world.transport.completed_effect_count(),
        1,
        "two request-affine dispatches must produce one physical consequence"
    );
    let completed = world
        .transport
        .completed_notice(correlation)
        .expect("the independent consequence owner retains the completed notice");
    assert_eq!(
        (completed.estate(), completed.notice(), completed.subject()),
        (
            world.fixture.estate.get(),
            world.fixture.notice.get(),
            world.fixture.deceased.get(),
        )
    );
}

#[test]
fn safe_retry_of_already_completed_effect_repeats_dispatch_but_not_physical_consequence() {
    let world = world::cross_gate_world("safe-retry-completed");
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let receipt = world.commit_notification(82);
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Completed)
    );
    assert_eq!(world.transport.admission_count(), 1);
    assert_eq!(world.transport.production_dispatches().len(), 1);
    assert_eq!(world.transport.completed_effect_count(), 1);
    let correlation = world.transport.attempts()[0].clone();

    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();

    // Would emit differently if the rail re-ran the fault path instead of
    // idempotently replaying the completed ledger record.
    world
        .transport
        .under(FaultScript::DuplicateAcknowledgement, world::PATIENT);
    let admission = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(handle, &specialist, action, &scope)
        .expect("safe-retry of completed effect");
    assert!(admission.is_external_completion());
    assert_eq!(
        world.transport.ledger_status(&correlation),
        LedgerStatus::Completed
    );
    assert_eq!(
        world.transport.admission_count(),
        1,
        "already-completed re-dispatch must not admit a second rail attempt"
    );
    assert_eq!(
        world.transport.production_dispatches().len(),
        2,
        "safe retry still crosses the production adapter"
    );
    assert_eq!(
        world.transport.completed_effect_count(),
        1,
        "the completed correlation must not repeat its physical consequence"
    );
}

#[test]
fn foreign_principal_safe_retry_denies_before_transport() {
    let world = world::cross_gate_world("safe-retry-foreign");
    world
        .transport
        .under(FaultScript::DisappearMidDispatch, world::PATIENT);
    let receipt = world.commit_notification(83);
    let handle = world.open_recovery(&receipt);
    let action = world.specialist_action();
    let scope = request_scope();
    let attempts_before = world.transport.attempts().len();
    let admissions_before = world.transport.admission_count();

    // Deceased never holds a notify-death grant. Fresh admission fails at
    // CapabilityGrantMissing before effect authority or transport (R8.69).
    // Binding-axis ForeignPrincipal is a later gate and is not reached here.
    let deceased = world.fixture.authenticate_deceased();
    let denied = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(handle, &deceased, action, &scope)
        .expect_err("foreign principal must deny");
    let (denied, retained) = denied.into_parts();
    let retained = retained.expect("denial must return the exact recovery handle");
    match denied {
        BankEstateProgressionDenial::Authorization(d) => {
            assert_eq!(
                d.kind(),
                bank_server::BankAuthorizationDenialKind::CapabilityGrantMissing,
                "deceased safe-retry must miss a live capability grant, got {:?}",
                d.kind()
            );
        }
        other => panic!("expected CapabilityGrantMissing authorization denial, got {other:?}"),
    }
    assert_eq!(world.transport.attempts().len(), attempts_before);
    assert_eq!(world.transport.admission_count(), admissions_before);
    let specialist = world.fixture.authenticate_specialist();
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(retained, &specialist, action, &scope)
        .expect("the authorized retry must consume the returned handle");
    assert_eq!(world.transport.attempts().len(), attempts_before + 1);
}

#[test]
fn unresolved_redispatch_retains_the_handle_for_a_later_completed_attempt() {
    let world = world::cross_gate_world("safe-retry-unresolved");
    world
        .transport
        .under(FaultScript::DisappearMidDispatch, world::PATIENT);
    let receipt = world.commit_notification(87);
    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();
    let denied = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(handle, &specialist, action, &scope)
        .expect_err("unresolved rail attempt cannot settle recovery");
    let (denial, retained) = denied.into_parts();
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::Recovery(ref denied)
            if denied.kind() == BankRecoveryDenialKind::UnresolvedExternalPosture
    ));
    let handle = retained.expect("unresolved transport must preserve the same live handle");
    assert_eq!(world.transport.completed_effect_count(), 0);
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let completed = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(handle, &specialist, action, &scope)
        .expect("the next completed attempt consumes the retained handle");
    assert!(completed.is_external_completion());
    assert_eq!(world.transport.completed_effect_count(), 1);
}

#[test]
fn expired_handle_safe_retry_denies_before_transport() {
    let authorization_time = AuthorizationTimeController::at_epoch_seconds(2_000);
    let world = world::cross_gate_world_with_authorization_time(
        "safe-retry-expired",
        Some(authorization_time.clone()),
    );
    world
        .transport
        .under(FaultScript::DisappearMidDispatch, world::PATIENT);
    let receipt = world.commit_notification(84);
    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();
    let attempts_before = world.transport.attempts().len();
    let admissions_before = world.transport.admission_count();

    authorization_time.advance_to_epoch_seconds(5_601);
    let denied = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(handle, &specialist, action, &scope)
        .expect_err("expired handle must deny");
    let (denied, retained) = denied.into_parts();
    assert!(
        retained.is_some(),
        "expiry denial must preserve owner custody"
    );
    match denied {
        BankEstateProgressionDenial::Recovery(d) => {
            assert_eq!(d.kind(), BankRecoveryDenialKind::Expired);
        }
        other => panic!("expected Expired, got {other:?}"),
    }
    assert_eq!(world.transport.attempts().len(), attempts_before);
    assert_eq!(world.transport.admission_count(), admissions_before);
}

#[test]
fn undeclared_external_effect_leaves_retry_path_nothing_to_find() {
    // R8.4 positive twin: transport is live; an undeclared effect writes no outbox.
    use std::sync::Arc;

    use bank_domain::model::Money;
    use bank_domain::proposals::BankIdempotencyKey;
    use bank_domain::schema::SendMoney;
    use bank_server::{mutations, BankMutationControls};
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

    use super::external_effect_dispatch::rail_transport::{spawn_rail, BankEstateRailTransport};
    use crate::fixture::{ordinary_read_world, principal_id, OWNER, RECIPIENT};

    let rail = spawn_rail();
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    let fixture = ordinary_read_world("safe-retry-r84", 0);
    fixture
        .world
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("rail installs");
    assert!(fixture.world.runtime.has_external_effect_transport());

    let owner = fixture.authenticate(OWNER);
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            from: fixture.personal_account,
            recipient: principal_id(RECIPIENT),
            amount: Money::from_minor(100).unwrap(),
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new("safe-retry-r84-send").unwrap(),
        ))
        .execute();
    let Ok(WorthQueryApplicationMutationOutcome::Committed { receipt, .. }) = &outcome else {
        panic!("lawful transfer must commit: {outcome:?}");
    };
    assert!(receipt.dispatch_outbox().is_none());
    assert!(transport.attempts().is_empty());
    assert_eq!(transport.admission_count(), 0);
}
