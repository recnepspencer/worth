//! Accepted cross-gate recovery evidence through the real rail and aftermath.

#[path = "phase8_cross_gate/world.rs"]
pub(crate) mod world;

use crate::support::request_scope;
use bank_domain::schema::{DisburseEstateOperation, NotifyDeathEstateOperation};
use bank_external_rail::test_control::FaultScript;
use bank_server::{
    BankCommitReceipt, BankEstateProgressionDenial, BankIdentityRuntime, BankRecoveryDenialKind,
    BankRecoveryDurability, BankRecoveryIdempotencyResolution, BankRecoveryInspection,
    BankRecoveryPosture, BankRecoverySupportTruth,
};
use worth_query_host::facade::domain::{
    PublishedAftermathPosture, WorthQueryInstalledAftermathContract,
};
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectFailure;

use self::world::cross_gate_world;

#[test]
fn lost_response_recovery_through_real_rail_and_aftermath() {
    let world = cross_gate_world("lost-response-recovery");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, world::PATIENT);
    let revision_before = world.estate_account_revision();
    let receipt = world.commit_with(world::idempotency(71));
    assert_lost_response_commit(&world, &receipt);
    assert_unresolved_recovery(&world, &receipt, revision_before);
}

fn assert_lost_response_commit(world: &world::CrossGateWorld, receipt: &BankCommitReceipt) {
    assert!(receipt.co_committed_dispatch_outbox());
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .and_then(|posture| posture.failure()),
        Some(WorthQueryPublishedExternalEffectFailure::LostResponse)
    );
    let aftermath = installed_notify_death_aftermath(&world.fixture.world.runtime);
    assert_eq!(
        aftermath.published_posture(),
        PublishedAftermathPosture::Reconcilable
    );
}

fn assert_unresolved_recovery(
    world: &world::CrossGateWorld,
    receipt: &BankCommitReceipt,
    revision_before: u64,
) {
    let handle = world.open_recovery(receipt);
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
        .expect("inspect-again");
    assert_recovery_publication(&first, &second);
    let denied = world
        .fixture
        .world
        .runtime
        .resolve_commit_recovery(handle, &specialist, action, &scope)
        .expect_err("unresolved stays unresolved");
    match denied {
        BankEstateProgressionDenial::Recovery(d) => {
            assert_eq!(d.kind(), BankRecoveryDenialKind::UnresolvedExternalPosture)
        }
        other => panic!("expected unresolved posture denial, got {other:?}"),
    }
    assert_eq!(world.estate_account_revision(), revision_before);
}

fn assert_recovery_publication(first: &BankRecoveryInspection, second: &BankRecoveryInspection) {
    assert_eq!(first.recovery_inspection_work().basis_preparations(), 0);
    assert_eq!(first.recovery_inspection_work().digest_derivations(), 0);
    assert_eq!(
        first
            .recovery_inspection_work()
            .digest_text_materializations(),
        0
    );
    assert_eq!(second.recovery_inspection_work().basis_preparations(), 0);
    assert_eq!(
        first.durability(),
        BankRecoveryDurability::StoreCapabilityRequired
    );
    assert_eq!(
        first.support_truth(),
        BankRecoverySupportTruth::DegradedRecoveryReport
    );
    assert_eq!(first.posture(), BankRecoveryPosture::Reconcilable);
    assert_eq!(first, second);
}

#[test]
fn unrelated_aftermath_lookup_cannot_substitute_the_handle_contract() {
    let world = cross_gate_world("aftermath-slot-substitution");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, world::PATIENT);
    let receipt = world.commit_notification(85);
    let aftermath = installed_notify_death_aftermath(&world.fixture.world.runtime);
    let substituted = installed_disburse_aftermath(&world.fixture.world.runtime);
    assert_ne!(substituted.identity(), aftermath.identity());
    let specialist = world.fixture.authenticate_specialist();
    let admitted = world
        .fixture
        .world
        .runtime
        .reconcile_commit_recovery(
            world.open_recovery(&receipt),
            &specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect("handle-carried NotifyDeath aftermath admits reconcile");
    assert_eq!(admitted.installed_operation(), aftermath.operation_slot());
}

#[test]
fn already_completed_resolve_returns_inherited_taxonomy() {
    let world = cross_gate_world("already-completed");
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let receipt = world.commit_with(world::idempotency(76));
    let specialist = world.fixture.authenticate_specialist();
    let resolution = world.fixture.world.runtime.resolve_commit_recovery(
        world.open_recovery(&receipt),
        &specialist,
        world.specialist_action(),
        &request_scope(),
    );
    match resolution {
        Ok(BankRecoveryIdempotencyResolution::AlreadyCommitted) => {}
        Err(BankEstateProgressionDenial::Recovery(d))
            if d.kind() == BankRecoveryDenialKind::UnresolvedExternalPosture => {}
        other => panic!("expected AlreadyCommitted or unresolved posture, got {other:?}"),
    }
}

#[test]
fn inspect_requires_disclosure_proof_not_boolean() {
    let world = cross_gate_world("inspect-disclosure");
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let receipt = world.commit_notification(74);
    let specialist = world.fixture.authenticate_specialist();
    let view = world
        .fixture
        .world
        .runtime
        .inspect_commit_recovery(
            &world.open_recovery(&receipt),
            &specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect("disclosure-backed inspect");
    assert_eq!(view.recovery_inspection_work().basis_preparations(), 0);
}

pub(crate) fn installed_notify_death_aftermath(
    runtime: &BankIdentityRuntime,
) -> WorthQueryInstalledAftermathContract {
    runtime.installed_operation_aftermath(NotifyDeathEstateOperation::reference())
}

pub(crate) fn installed_disburse_aftermath(
    runtime: &BankIdentityRuntime,
) -> WorthQueryInstalledAftermathContract {
    runtime.installed_operation_aftermath(DisburseEstateOperation::reference())
}
