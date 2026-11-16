//! Observation-pressure release follows declared commit order, with native custody.

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryRetainedOutputDemandSource;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn unheld_performed_sources_release_the_oldest_in_commit_order() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let mut sources = Vec::new();
    let mut current = "open".to_owned();
    for index in 0..3 {
        let replacement = format!("release-source-{index}");
        let mut receipt =
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::commit_on_world_with_output_demand_observation(
                &world,
                170 + index,
                &current,
                &replacement,
            );
        let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation_for_test()
            .unwrap();
        sources.push(WorthQueryRetainedOutputDemandSource::new_for_test(
            receipt,
            change,
            observation,
            None,
        ));
        current = replacement;
    }
    let oldest = (*sources[0].change_for_test()).product_commit().clone();
    let occurrence = (*sources[0].receipt_for_test())
        .product_branch()
        .occurrence_for_test();
    assert!(sources
        .windows(2)
        .all(|pair| (*pair[0].change_for_test()).product_commit()
            < (*pair[1].change_for_test()).product_commit()));
    let mut state = DemandRegistryState::default();
    for index in [2, 0, 1] {
        let source = sources[index].clone();
        state
            .source_custody
            .insert((*source.change_for_test()).product_commit().clone(), {
                // Bound in the original literal order, passed in the door's.
                let (
                    occurrence,
                    root_kind,
                    source,
                    discovery,
                    bound_sources,
                    consumed_sources,
                    retired_sources,
                    retired,
                    token_count,
                    cleanup,
                    completed,
                ) = (
                    occurrence,
                    PreparedOutputRootKind::Required(TypeId::of_for_test::<()>()),
                    Some(source),
                    None,
                    None,
                    Vec::new(),
                    Vec::new(),
                    None,
                    0,
                    Default::default(),
                    false,
                );
                SourceCustody::new_for_test(
                    occurrence,
                    root_kind,
                    source,
                    discovery,
                    consumed_sources,
                    retired_sources,
                    bound_sources,
                    retired,
                    token_count,
                    cleanup,
                    completed,
                )
            });
    }
    for slot in 1..=3 {
        let mut row = record(occurrence, DemandState::Admitted, 0);
        (*row.source_commits_mut_for_test()).push(oldest.clone());
        (*state.records_mut_for_test()).insert_for_test(key("release-proof", 1, slot), row);
    }

    let released = state.release_unheld_performed_source_for_test().unwrap();
    assert!(released.is_some());
    assert!(!state.source_custody.contains_key(&oldest));
    assert_eq!(state.source_custody.len(), 2);
    assert!((*state.records_for_test())
        .values_for_test()
        .all(|row| !(*row.source_commits_for_test()).contains(&oldest)));
}
