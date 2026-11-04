//! Registry transfer evidence; this does not certify output currentness.

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

pub type Commit = worth_runtime_world::facade::CompositeCommitIdentity;

pub fn fixture() -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    Commit,
) {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    fixture_with_commit(commit)
}

pub fn fixture_with_commit(
    commit: Commit,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    Commit,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("replacement", 1, 1, 1);
    let next = key_with_identity("replacement", 2, 1, 1);
    let mut old_record = record(occurrence(), DemandState::Admitted, 1);
    for _ in 0..2 {
        (*old_record
            .performed_obligations_mut_for_test())
            .push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::PerformedOutputObligation::new_for_test(commit.clone(), (*old.source_for_test()).clone()));
    }
    (*old_record
        .performed_obligations_mut_for_test())
        .push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::PerformedOutputObligation::new_for_test(commit.clone(), key("unmatched", 1, 2).source_into_for_test()));
    let next_record = record(occurrence(), DemandState::Admitted, 1);
    let old_interest = interest(
        &registry,
        old.clone(),
        Arc::clone(&(*old_record.wake_for_test())),
    );
    let next_interest = interest(
        &registry,
        next.clone(),
        Arc::clone(&(*next_record.wake_for_test())),
    );
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        (*state.records_mut_for_test()).insert_for_test(old, old_record);
        (*state.records_mut_for_test()).insert_for_test(next, next_record);
    }
    (registry, old_interest, next_interest, commit)
}

#[test]
fn replacement_prepares_exact_commit_growth_and_transfers_matching_obligations() {
    let (registry, old, next, commit) = fixture();
    registry
        .finish_replaced_interest_for_test(&old, &next, "replacement")
        .unwrap();
    let state = (*registry.state_for_test()).lock().unwrap();
    let successor = &(*state.records_for_test())[&(*next.key_for_test())];
    assert_eq!((*successor.performed_obligations_for_test()).len(), 2);
    assert_eq!((*successor.source_commits_for_test()), [commit]);

    assert!((*state.required_keys_for_test()).contains(&(*next.key_for_test())));
    assert_eq!(
        (*(*state.records_for_test())[&(*old.key_for_test())].performed_obligations_for_test())
            .len(),
        0
    );
}
