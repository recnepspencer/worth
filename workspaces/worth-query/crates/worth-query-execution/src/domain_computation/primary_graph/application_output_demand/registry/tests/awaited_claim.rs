//! A dependent claim is an owner of the occurrence it consumed.
use super::published_retirement::{admit, balances, publish};
use super::*;

fn source_owner(
) -> crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    let resources = WorthQueryInvalidationResources::install(
        WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
    )
    .unwrap();
    crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner::new(
        resources, 1,
    )
}

fn last_claim_closes(held_successor: bool) {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let (unrelated, baseline) = super::reservation_baseline::install(
            &mut registry.state.lock().unwrap(),
            receipt.clone(),
        );
        let stale_key = key("claim-awaited", 1, 1);
        let newest_key = key("claim-awaited", 2, 1);
        let dependent_key = key("claim-owner", 1, 2);
        let stale = admit(&registry, &stale_key);
        let dependent = admit(&registry, &dependent_key);
        publish(&registry, &dependent, receipt.clone());
        // This local lifetime fixture declares one funded consumed edge.
        // The registry's release of its own claim, rather than native
        // certification of the dependency, is the boundary under test.
        {
            let mut state = registry.state.lock().unwrap();
            let claims = vec![Arc::new(stale_key.clone())];
            state.required_reserved_bytes +=
                claims.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            state.records.get_mut(&dependent_key).unwrap().prerequisites = claims;
            state
                .records
                .get_mut(&stale_key)
                .unwrap()
                .framework_required_count += 1;
        }
        drop(stale);
        let newest = admit(&registry, &newest_key);
        publish(&registry, &newest, receipt);
        if held_successor {
            assert!(registry
                .hold_required_successor(Box::new(newest), |held| &held
                    .downcast_ref::<WorthQueryOutputDemandInterest>()
                    .unwrap()
                    .key)
                .is_none());
        } else {
            drop(newest);
        }
        assert!(super::super::refreshed_rejoin::awaited_by_stale_owner(
            &registry.state.lock().unwrap().records,
            &newest_key
        ));
        drop(dependent);
        // Release leaves costly cleanup for the next paying admission.
        loop {
            let before = registry.state.lock().unwrap().records.len();
            match registry.drain_terminal_cleanup_admitted(&source_owner(), &mut record_admission())
            {
                Ok(()) => break,
                Err(stop) => {
                    assert_eq!(
                        stop.kind(),
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                    );
                    assert!(
                        registry.state.lock().unwrap().records.len() < before,
                        "a fresh paying admission must retire at least one queued row"
                    );
                }
            }
        }
        {
            let state = registry.state.lock().unwrap();
            assert!(
                !state.records.contains_key(&newest_key),
                "the last claim's release retires the answer and any held successor"
            );
            assert!(
                !state.records.contains_key(&stale_key),
                "the superseded row is retired too"
            );
            assert!(state.records.iter().all(|(key, record)| key == &unrelated ||
                !matches!(&record.state, DemandState::Output(output) if matches!(output.checkpoint,
                    Some(super::super::WorthQueryOutputCheckpoint::Published { .. })))));
        }
        assert_eq!(
            balances(&registry),
            baseline,
            "all three reservations return by value"
        );
        assert!(registry
            .state
            .lock()
            .unwrap()
            .records
            .contains_key(&unrelated));
    });
}

#[test]
fn the_last_awaiting_claim_retires_the_undelivered_answer() {
    last_claim_closes(false);
}
#[test]
fn the_last_awaiting_claim_releases_the_held_successor() {
    last_claim_closes(true);
}
