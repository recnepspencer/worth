//! A selected publication joins only the binding that owns its delivery.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture;

#[test]
fn selected_delivery_keeps_both_producer_and_applicability_identity() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt.committed_product_publication().composite_commit();
    let (scope, _) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence();
    for other_producer in [true, false] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let own = key("z-own-binding", 1, 1);
        let requested = key("z-own-binding", 0, 1);
        let mut foreign = own.clone();
        if other_producer {
            (*foreign.producer_mut_for_test()) = "a-other-binding".into();
        } else {
            (*foreign.applicability_mut_for_test()) = WorthQueryProducerApplicability::new(
                "registry-fixture",
                WorthQueryProducerLifecyclePosture::Initial,
            );
        }
        assert!(foreign < own);
        assert!(requested < foreign);
        {
            let mut state = (*registry.state_for_test()).lock().unwrap();
            for binding in [&foreign, &own] {
                let DemandState::Output(mut output) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready() else {
                    panic!("Ready fixture")
                };
                (*output.published_commit_mut_for_test()) = Some(commit.clone());
                // Delivery can outlive an interrupted advancement. Semantic
                // joining must not hide the selected-publication fallback.
                output.finish_stopped_for_test(WorthQueryOutputDemandDenial::new_for_test(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "",
                ));
                let mut row = record(occurrence(), DemandState::Output(output), 1);
                (*row.source_scope_mut_for_test()) = Some(scope);
                (*state.records_mut_for_test()).insert_for_test(binding.clone(), row);
            }
        }
        let joined = registry
            .admit_for_test(
                requested,
                Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::admission::selected_output::SelectedOutputAdmission::new(commit, None)),
                scope,
                occurrence(),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
                None,
                None,
            )
            .map(|(interest, _pause)| interest)
            .unwrap();
        assert_eq!(
            (*joined.key_for_test()),
            own,
            "other producer: {other_producer}"
        );
        let state = (*registry.state_for_test()).lock().unwrap();
        assert_eq!((*(*state.records_for_test())[&own].interests_for_test()), 2);
        assert_eq!(
            (*(*state.records_for_test())[&foreign].interests_for_test()),
            1
        );
        assert_eq!((*state.records_for_test()).len_for_test(), 2);
        drop(state);
        drop(joined);
    }
}
