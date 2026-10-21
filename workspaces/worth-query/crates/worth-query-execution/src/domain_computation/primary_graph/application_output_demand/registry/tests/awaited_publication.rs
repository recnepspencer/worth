//! A superseded interest owns the newest undelivered answer of its occurrence.
use super::super::{
    WorthQueryOutputCheckpoint, WorthQueryOutputDemandAdvanceAdmission as Admission,
};
use super::published_retirement::{admit, publish};
use super::*;

fn scenario(rejoin: bool, late_hold: bool) {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();

        let old_key = key("awaited", 1, 1);
        let newest_key = key("awaited", 2, 1);
        let stale = admit(&registry, &old_key);
        let refreshing = admit(&registry, &newest_key);
        // The registry transports this opaque installed readmission payload.
        // Native Query certification is outside this row-lifetime proof.
        let source = Arc::new(super::super::source_readmission::RequiredOutputReadmission {
            source: Arc::new(()), producer: std::sync::OnceLock::new(),
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile::standard().limits(),
            retained_program_basis: None,
        });
        registry
            .state
            .lock()
            .unwrap()
            .records
            .get_mut(&newest_key)
            .unwrap()
            .readmission_source = Some(Arc::clone(&source));
        publish(&registry, &refreshing, receipt);
        if late_hold {
            drop(stale);
            let key = newest_key.clone();
            let returned = registry.hold_required_successor(Box::new(refreshing), |successor| {
                &successor
                    .downcast_ref::<WorthQueryOutputDemandInterest>()
                    .unwrap()
                    .key
            });
            assert!(
                returned.is_some(),
                "no stale owner awaits this late successor; the registry must give it back"
            );
            drop(returned);
            assert!(!registry.state.lock().unwrap().records.contains_key(&key));

            return;
        }
        drop(refreshing);
        {
            let state = registry.state.lock().unwrap();
            assert!(
                state.records.contains_key(&newest_key),
                "the freshest Published answer still belongs to the stale owner"
            );
            assert!(super::super::refreshed_rejoin::awaited_by_stale_owner(
                &state.records,
                &newest_key
            ));
        }
        if rejoin {
            let (owner, joined_source) = registry
                .rejoin_refreshed(&stale)
                .unwrap()
                .expect("the stale owner rejoins its exact latest answer");
            assert_eq!(owner.key, newest_key);
            assert!(Arc::ptr_eq(&joined_source, &source));
            drop(joined_source);
            drop(stale);
            let Admission::AdvanceCheckpoint { claim, checkpoint } = registry.begin(&owner) else {
                panic!("the rejoined owner alone claims the authentic Published checkpoint");
            };
            let WorthQueryOutputCheckpoint::Published {
                receipt,
                delivery: _,
                ready_backing,
            } = checkpoint
            else {
                panic!("one undelivered publication is owed");
            };
            registry
                .finish_checkpoint(
                    &owner,
                    claim,
                    WorthQueryOutputCheckpoint::Delivered {
                        receipt,
                        delivery: None,
                        ready_backing,
                    },
                    None,
                )
                .unwrap();
            let Admission::AdvanceCheckpoint { claim, checkpoint } = registry.begin(&owner) else {
                panic!("the rejoined owner completes the delivered answer");
            };
            let WorthQueryOutputCheckpoint::Delivered {
                receipt,
                delivery: _,
                ready_backing,
            } = checkpoint
            else {
                panic!("the answer was delivered exactly once");
            };
            let ready = ready_backing.complete(super::super::WorthQueryCompletedOutputDemand {
                authority: super::super::WorthQueryAcceptedOutputAuthority::Committed(receipt),
                readiness: crate::domain_computation::primary_graph::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
            });
            registry
                .finish_checkpoint(
                    &owner,
                    claim,
                    WorthQueryOutputCheckpoint::Ready(ready),
                    None,
                )
                .unwrap();
            assert!(
                matches!(registry.begin(&owner), Admission::Ready(_)),
                "no second delivery"
            );
            drop(owner);
            assert!(
                registry.state.lock().unwrap().records[&newest_key].has_cached_ready(),
                "closing the owner keeps its delivered Ready cache"
            );

            registry.release_product_occurrence(occurrence());
        } else {
            drop(stale);
            assert!(
                !registry
                    .state
                    .lock()
                    .unwrap()
                    .records
                    .contains_key(&newest_key),
                "closing the last stale owner retires its now idle Published answer"
            );
        }
        drop(source);
    });
}
#[test]
fn a_stale_owner_rejoins_the_publication_after_the_refresher_closes() {
    scenario(true, false);
}
#[test]
fn closing_the_last_stale_owner_retires_its_undelivered_answer() {
    scenario(false, false);
}
#[test]
fn a_successor_held_after_its_last_awaiting_owner_closed_is_returned() {
    scenario(false, true);
}

#[test]
fn closing_the_last_stale_owner_releases_its_held_successor() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();

        let stale_key = key("held-awaiting", 1, 1);
        let newest_key = key("held-awaiting", 2, 1);
        let stale = admit(&registry, &stale_key);
        let successor = admit(&registry, &newest_key);
        publish(&registry, &successor, receipt);
        assert!(registry
            .hold_required_successor(Box::new(successor), |successor| &successor
                .downcast_ref::<WorthQueryOutputDemandInterest>()
                .unwrap()
                .key)
            .is_none());
        assert!(registry.state.lock().unwrap().records[&newest_key]
            .held_successor
            .is_some());
        drop(stale);
        assert!(
            !registry
                .state
                .lock()
                .unwrap()
                .records
                .contains_key(&newest_key),
            "the last awaiting owner's release also closes its held successor"
        );
    });
}
