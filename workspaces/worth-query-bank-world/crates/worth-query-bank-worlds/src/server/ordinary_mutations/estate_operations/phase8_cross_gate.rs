//! Accepted cross-gate recovery evidence through the real rail and aftermath.

#[path = "phase8_cross_gate/world.rs"]
pub mod world;

pub use crate::facade::server::support::request_scope;
pub use bank_domain::schema::{DisburseEstateOperation, NotifyDeathEstateOperation};
pub use bank_external_rail::test_control::FaultScript;
pub use bank_server::{
    BankCommitReceipt, BankEstateProgressionDenial, BankIdentityRuntime, BankRecoveryClaimStatus,
    BankRecoveryDenialKind, BankRecoveryDurability, BankRecoveryIdempotencyResolution,
    BankRecoveryInspection, BankRecoveryPosture, BankRecoverySupportTruth,
};
pub use worth_query_host::facade::domain::{
    PublishedAftermathPosture, WorthQueryInstalledAftermathContract,
};
pub use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectFailure;

pub use self::world::cross_gate_world;

pub fn assert_lost_response_commit(world: &world::CrossGateWorld, receipt: &BankCommitReceipt) {
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

pub fn assert_unresolved_recovery(
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
    assert_eq!(
        world
            .fixture
            .world
            .runtime
            .commit_recovery_claim_status(receipt),
        Ok(BankRecoveryClaimStatus::Consumed),
        "a resolved-but-unconfirmed rail effect must not report completed recovery"
    );
    assert_eq!(world.estate_account_revision(), revision_before);
}

pub fn assert_recovery_publication(
    first: &BankRecoveryInspection,
    second: &BankRecoveryInspection,
) {
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

pub fn installed_notify_death_aftermath(
    runtime: &BankIdentityRuntime,
) -> WorthQueryInstalledAftermathContract {
    runtime.installed_operation_aftermath(NotifyDeathEstateOperation::reference())
}

pub fn installed_disburse_aftermath(
    runtime: &BankIdentityRuntime,
) -> WorthQueryInstalledAftermathContract {
    runtime.installed_operation_aftermath(DisburseEstateOperation::reference())
}
