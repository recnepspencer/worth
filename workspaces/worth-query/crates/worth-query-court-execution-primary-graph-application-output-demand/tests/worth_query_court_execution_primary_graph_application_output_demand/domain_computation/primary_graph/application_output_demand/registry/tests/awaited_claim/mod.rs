//! A dependent claim is an owner of the occurrence it consumed.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::published_retirement::admit;

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::published_retirement::publish;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

pub fn source_owner(
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner{
    use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
    let resources = WorthQueryInvalidationResources::new();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner::new(
        resources, 1,
    )
}

pub fn last_claim_closes(held_successor: bool) {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let unrelated = crate::domain_computation::primary_graph::application_output_demand::registry::tests::reservation_baseline::install(
            &mut (*registry.state_for_test()).lock().unwrap(),
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
            let mut state = (*registry.state_for_test()).lock().unwrap();
            let claims = vec![Arc::new(stale_key.clone())];

            (*(*state.records_mut_for_test()).get_mut(&dependent_key).unwrap().prerequisites_mut_for_test()) = claims;
            (*(*state
                .records_mut_for_test())
                .get_mut(&stale_key)
                .unwrap()
                .framework_required_count_mut_for_test()) += 1;
        }
        drop(stale);
        let newest = admit(&registry, &newest_key);
        publish(&registry, &newest, receipt);
        if held_successor {
            assert!(registry
                .hold_required_successor_for_test(Box::new(newest), |held| held
                    .downcast_ref::<WorthQueryOutputDemandInterest>()
                    .map(|interest| &(*interest.key_for_test())))
                .is_none());
        } else {
            drop(newest);
        }
        assert!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::awaited_by_stale_owner(
            &(*(*registry.state_for_test()).lock().unwrap().records_for_test()),
            &newest_key
        ));
        drop(dependent);
        // Release leaves costly cleanup for the next paying admission.
        // The lifetime proof funds the whole finite cleanup, once. Partial
        // admission and exact charges belong to the every-unit drain table.

        registry
            .drain_terminal_cleanup_admitted_for_test(&source_owner())
            .unwrap();

        {
            let state = (*registry.state_for_test()).lock().unwrap();
            assert!(
                !(*state.records_for_test()).contains_key_for_test(&newest_key),
                "the last claim's release retires the answer and any held successor"
            );
            assert!(
                !(*state.records_for_test()).contains_key_for_test(&stale_key),
                "the superseded row is retired too"
            );
            assert!((*state.records_for_test()).iter_for_test().all(|(key, record)| key == &unrelated ||
                !matches!(&(*record.state_for_test()), DemandState::Output(output) if matches!((*output.checkpoint_for_test()),
                    Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Published { .. })))));
        }

        assert!((*(*registry
            .state_for_test())
            .lock()
            .unwrap()
            .records_for_test())
            .contains_key_for_test(&unrelated));
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
