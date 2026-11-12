//! Recovery-handle registry, linearity, and wire-boundary evidence (Gate 8.3).

use std::sync::Arc;

use worth_query_execution::court_doors_for_test::domain_computation::managed_run::recovery_registry::WorthQueryRecoveryHandleRegistry;
use worth_query_execution::court_doors_for_test::domain_computation::managed_run::recovery_registry::WorthQueryRecoveryMintClaim;
use worth_foundational::facade::RecoveryClaimDisposition;
use worth_query_installation::facade::PublishedAftermathPosture;

#[test]
fn concurrent_receipt_claims_register_exactly_one_handle() {
    let registry = Arc::new(WorthQueryRecoveryHandleRegistry::new_for_test());
    let claim = real_claim(7);
    let left_registry = Arc::clone(&registry);
    let left_claim = claim.clone();
    let results = std::thread::scope(|scope| {
        let left = scope.spawn(move || left_registry.register_once_for_test(left_claim));
        let right_registry = Arc::clone(&registry);
        let right = scope.spawn(move || right_registry.register_once_for_test(claim));

        [
            left.join().expect("left claim"),
            right.join().expect("right claim"),
        ]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
    let slot = results.into_iter().find_map(Result::ok).expect("one slot");
    assert!(registry.mark_terminal_for_test(slot, RecoveryClaimDisposition::Disposed));
    registry.assert_no_live_handles_for_test();
}

#[test]
fn recovery_registry_enumerates_terminates_and_covers_four_paths_without_leak() {
    let registry = WorthQueryRecoveryHandleRegistry::new_for_test();
    let slot = registry.register_axis_probe_for_test();
    assert!(registry.is_live_for_test(slot));
    assert!(registry.enumerate_live_for_test().contains(&slot));
    assert!(registry.mark_terminal_for_test(slot, RecoveryClaimDisposition::ForceTerminated));
    assert!(!registry.is_live_for_test(slot));
    assert_eq!(
        registry.terminal_of_for_test(slot),
        Some(RecoveryClaimDisposition::ForceTerminated)
    );
    registry.assert_no_live_handles_for_test();

    let registry = WorthQueryRecoveryHandleRegistry::new_for_test();
    let terminals = [
        RecoveryClaimDisposition::Consumed,
        RecoveryClaimDisposition::Expired,
        RecoveryClaimDisposition::Disposed,
        RecoveryClaimDisposition::ForceTerminated,
    ];
    for terminal in terminals {
        let slot = registry.register_axis_probe_for_test();
        assert!(registry.mark_terminal_for_test(slot, terminal));
        assert!(!registry.is_live_for_test(slot));
        assert_eq!(registry.terminal_of_for_test(slot), Some(terminal));
    }
    registry.assert_no_live_handles_for_test();
}

// Q8.21-L11 boundary, stated where it is cheapest to state: which fates keep a
// commit's mint claim and which give it back. The Bank integration twins prove
// the guarantee end to end; this proves the exact line, including the two cases
// no transition can reach â€” `Drop` on an abandoned handle (which records
// `Disposed`, so abandoning recovery spends it) and relinquishing a slot that
// already reached a terminal (which must not resurrect a spent commit).
#[test]
fn only_relinquishment_returns_a_commits_mint_claim() {
    for retaining in [
        RecoveryClaimDisposition::Consumed,
        RecoveryClaimDisposition::Expired,
        RecoveryClaimDisposition::Disposed,
        RecoveryClaimDisposition::ForceTerminated,
    ] {
        let registry = WorthQueryRecoveryHandleRegistry::new_for_test();
        let claim = real_claim(3);
        let slot = registry
            .register_once_for_test(claim.clone())
            .expect("first claim");
        assert!(registry.mark_terminal_for_test(slot, retaining));
        registry
            .register_once_for_test(claim)
            .expect_err("an exercised recovery stays spent forever");
        // Relinquishing after a real terminal must not give the claim back.
        assert!(!registry.relinquish_for_test(slot));
        registry.assert_no_live_handles_for_test();
    }

    let registry = WorthQueryRecoveryHandleRegistry::new_for_test();
    let claim = real_claim(3);
    let first = registry
        .register_once_for_test(claim.clone())
        .expect("first claim");
    assert!(registry.relinquish_for_test(first));
    let second = registry
        .register_once_for_test(claim)
        .expect("a relinquished attempt leaves the commit recoverable");
    assert_ne!(
        first, second,
        "the retry gets a fresh slot, not the old one"
    );
    assert!(registry.is_live_for_test(second));
    assert!(registry.mark_terminal_for_test(second, RecoveryClaimDisposition::Consumed));
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

pub fn real_claim(provider_runtime_instance_id: u64) -> WorthQueryRecoveryMintClaim {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    WorthQueryRecoveryMintClaim::new_for_test(
        provider_runtime_instance_id,
        receipt
            .committed_product_publication()
            .composite_commit()
            .clone(),
    )
}
