//! Recovery-handle registry ownership and terminal claim dispositions.

use worth_foundational::facade::RecoveryClaimDisposition;
use worth_query_execution::facade::domain_computation::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind,
};
use worth_query_installation::facade::PublishedAftermathPosture;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::recoverable_application_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

fn mint(
    world: &AuthorizationWorld,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
    world
        .application
        .with_host_advancement(|phase| world.application.mint_recovery_handle(&phase, receipt))
        .unwrap()
}

#[test]
#[expect(clippy::disallowed_methods, reason = "Client-race court")]
fn concurrent_receipt_claims_register_exactly_one_handle() {
    let (world, receipt) = recoverable_application_world(7, "claimed-by-one");
    let results = std::thread::scope(|scope| {
        let left = scope.spawn(|| mint(&world, &receipt));
        let right = scope.spawn(|| mint(&world, &receipt));
        [left.join().unwrap(), right.join().unwrap()]
    });
    let (handle, denial) = match results {
        [Ok(handle), Err(denial)] | [Err(denial), Ok(handle)] => (handle, denial),
        other => panic!("one client must hold the commit's recovery: {other:?}"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryRecoveryHandleDenialKind::RecoveryAlreadyMinted
    );
    let registry = handle.registry_arc_for_test();
    drop(handle);
    registry.assert_no_live_handles_for_test();
}

#[test]
fn recovery_registry_enumerates_terminates_and_covers_four_paths_without_leak() {
    for terminal in [
        RecoveryClaimDisposition::Consumed,
        RecoveryClaimDisposition::Expired,
        RecoveryClaimDisposition::Disposed,
        RecoveryClaimDisposition::ForceTerminated,
    ] {
        let (world, receipt) = recoverable_application_world(3, "terminal-recovery");
        let handle = mint(&world, &receipt).unwrap();
        let registry = handle.registry_arc_for_test();
        let slot = handle.registry_slot_for_test();
        assert!(registry.is_live_for_test(slot));
        assert!(registry.enumerate_live_for_test().contains(&slot));
        assert!(registry.mark_terminal_for_test(slot, terminal));
        assert!(!registry.is_live_for_test(slot));
        assert_eq!(registry.terminal_of_for_test(slot), Some(terminal));
        assert!(!registry.relinquish_for_test(slot));
        drop(handle);
        assert_eq!(
            mint(&world, &receipt).unwrap_err().kind(),
            WorthQueryRecoveryHandleDenialKind::RecoveryAlreadyMinted,
        );
        registry.assert_no_live_handles_for_test();
    }
}

#[test]
fn only_relinquishment_returns_a_commits_mint_claim() {
    let (world, receipt) = recoverable_application_world(3, "retryable-recovery");
    let handle = mint(&world, &receipt).unwrap();
    let registry = handle.registry_arc_for_test();
    let first = handle.registry_slot_for_test();
    assert!(registry.relinquish_for_test(first));
    drop(handle);

    let retry =
        mint(&world, &receipt).expect("a relinquished attempt leaves the commit recoverable");
    let second = retry.registry_slot_for_test();
    assert_ne!(first, second, "the retry gets a fresh slot");
    assert!(registry.is_live_for_test(second));
    drop(retry);
    registry.assert_no_live_handles_for_test();
}

#[test]
fn world_construction_installs_via_production_derivation() {
    let installed = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::notify_death();
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Reconcilable
    );
}
