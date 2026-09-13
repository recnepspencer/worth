//! Recovery-handle registry, linearity, and wire-boundary evidence (Gate 8.3).

use std::sync::Arc;

use crate::domain_computation::application_aftermath::aftermath_schema_fixture as fixture;
use crate::domain_computation::managed_run::{
    WorthQueryRecoveryHandleRegistry, WorthQueryRecoveryMintClaim,
    WorthQueryRecoveryResourceTerminal,
};
use worth_query_installation::facade::PublishedAftermathPosture;

#[test]
fn concurrent_receipt_claims_register_exactly_one_handle() {
    let registry = Arc::new(WorthQueryRecoveryHandleRegistry::new());
    let claim = real_claim(7);
    let left_registry = Arc::clone(&registry);
    let left_claim = claim.clone();
    let left = std::thread::spawn(move || left_registry.register_once(left_claim));
    let right_registry = Arc::clone(&registry);
    let right = std::thread::spawn(move || right_registry.register_once(claim));

    let results = [
        left.join().expect("left claim"),
        right.join().expect("right claim"),
    ];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
    let slot = results.into_iter().find_map(Result::ok).expect("one slot");
    assert!(registry.mark_terminal(slot, WorthQueryRecoveryResourceTerminal::Disposed));
    registry.assert_no_live_handles();
}

#[test]
fn recovery_registry_enumerates_terminates_and_covers_four_paths_without_leak() {
    let registry = WorthQueryRecoveryHandleRegistry::new();
    let slot = registry.register_axis_probe();
    assert!(registry.is_live(slot));
    assert!(registry.enumerate_live().contains(&slot));
    assert!(registry.mark_terminal(slot, WorthQueryRecoveryResourceTerminal::ForceTerminated));
    assert!(!registry.is_live(slot));
    assert_eq!(
        registry.terminal_of(slot),
        Some(WorthQueryRecoveryResourceTerminal::ForceTerminated)
    );
    registry.assert_no_live_handles();

    let registry = WorthQueryRecoveryHandleRegistry::new();
    let terminals = [
        WorthQueryRecoveryResourceTerminal::Consumed,
        WorthQueryRecoveryResourceTerminal::Expired,
        WorthQueryRecoveryResourceTerminal::Disposed,
        WorthQueryRecoveryResourceTerminal::ForceTerminated,
    ];
    for terminal in terminals {
        let slot = registry.register_axis_probe();
        assert!(registry.mark_terminal(slot, terminal));
        assert!(!registry.is_live(slot));
        assert_eq!(registry.terminal_of(slot), Some(terminal));
    }
    registry.assert_no_live_handles();
}

// Q8.21-L11 boundary, stated where it is cheapest to state: which fates keep a
// commit's mint claim and which give it back. The Bank integration twins prove
// the guarantee end to end; this proves the exact line, including the two cases
// no transition can reach — `Drop` on an abandoned handle (which records
// `Disposed`, so abandoning recovery spends it) and relinquishing a slot that
// already reached a terminal (which must not resurrect a spent commit).
#[test]
fn only_relinquishment_returns_a_commits_mint_claim() {
    for retaining in [
        WorthQueryRecoveryResourceTerminal::Consumed,
        WorthQueryRecoveryResourceTerminal::Expired,
        WorthQueryRecoveryResourceTerminal::Disposed,
        WorthQueryRecoveryResourceTerminal::ForceTerminated,
    ] {
        let registry = WorthQueryRecoveryHandleRegistry::new();
        let claim = real_claim(3);
        let slot = registry.register_once(claim.clone()).expect("first claim");
        assert!(registry.mark_terminal(slot, retaining));
        registry
            .register_once(claim)
            .expect_err("an exercised recovery stays spent forever");
        // Relinquishing after a real terminal must not give the claim back.
        assert!(!registry.relinquish(slot));
        registry.assert_no_live_handles();
    }

    let registry = WorthQueryRecoveryHandleRegistry::new();
    let claim = real_claim(3);
    let first = registry.register_once(claim.clone()).expect("first claim");
    assert!(registry.relinquish(first));
    let second = registry
        .register_once(claim)
        .expect("a relinquished attempt leaves the commit recoverable");
    assert_ne!(
        first, second,
        "the retry gets a fresh slot, not the old one"
    );
    assert!(registry.is_live(second));
    assert!(registry.mark_terminal(second, WorthQueryRecoveryResourceTerminal::Consumed));
    registry.assert_no_live_handles();
}

#[test]
fn phase8_world_construction_installs_via_production_derivation() {
    let installed = fixture::notify_death();
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Reconcilable
    );
}

fn real_claim(provider_runtime_instance_id: u64) -> WorthQueryRecoveryMintClaim {
    let receipt = crate::domain_computation::primary_graph::committed_recoverable_application();
    WorthQueryRecoveryMintClaim::new(
        provider_runtime_instance_id,
        receipt
            .committed_product_publication()
            .composite_commit()
            .clone(),
    )
}
