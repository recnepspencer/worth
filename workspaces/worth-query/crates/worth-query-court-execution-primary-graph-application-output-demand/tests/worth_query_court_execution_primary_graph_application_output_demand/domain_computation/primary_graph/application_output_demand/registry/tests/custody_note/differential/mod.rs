//! Distinct claim-release paths drain their own held custody after the last claim.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::custody_note::*;

#[test]
fn custody_claim_release_cancel_and_publication_leave_no_unreachable_note() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let ready = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
    let scope = receipt.principal_scope().scope();
    let (_lineage, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    // These are three different owner transitions, not lifecycle stage variants.
    for disposition in ["release", "cancel", "publish"] {
        let (registry, owner, dependent, _) = fixture(&receipt, &ready, 0, scope);
        let commit = receipt.committed_product_publication().composite_commit();
        let neighbor = key("unrelated-live-row", 1, 3);
        {
            let mut state = registry.state_for_test().lock().unwrap();
            state.records_mut_for_test().insert_for_test(
                neighbor.clone(),
                record(occurrence(), DemandState::Admitted, 1),
            );
        }
        drop(owner);
        registry
            .state_for_test()
            .lock()
            .unwrap()
            .drain_next_foreign_custody_for_test();
        assert!(
            registry
                .state_for_test()
                .lock()
                .unwrap()
                .source_custody
                .contains_key(commit),
            "{disposition}: the stale dependency still owns its exact source"
        );
        if disposition == "release" {
            let mut state = registry.state_for_test().lock().unwrap();
            *state
                .records_mut_for_test()
                .get_mut(dependent.key_for_test())
                .unwrap()
                .state_mut_for_test() =
                DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    "finished dependent",
                ));
            state.release_record_prerequisites_for_test(dependent.key_for_test());
        } else {
            worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_output_demand::registry::prerequisite_claims::custody_note_steps::unhold(
                &dependent,disposition=="cancel",&identity);
        }
        drop(dependent);
        registry.drain_closed_cleanup_admitted().unwrap();
        let mut state = registry.state_for_test().lock().unwrap();
        let queued = state.custody_cleanup_for_test().len;
        for _ in 0..queued {
            state.drain_next_foreign_custody_for_test();
        }
        assert!(
            !state.source_custody.contains_key(commit),
            "{disposition}: last claim releases custody"
        );
        assert!(
            !state.pending_custody_prune_for_test(),
            "{disposition}: no unreachable cleanup note remains"
        );
        assert!(matches!(
            state.records_for_test()[&neighbor].state_for_test(),
            DemandState::Admitted
        ));
        assert_eq!(*state.records_for_test()[&neighbor].interests_for_test(), 1);
    }
}
