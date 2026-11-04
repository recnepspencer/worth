//! Drop notes completed custody; the next admitted drain owns its sweep.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::lifecycle::completed_custody::contacts;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

pub fn fixture(
    token: bool,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandKey,
    Vec<worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt>,
) {
    let receipts = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
    let registry = WorthQueryOutputDemandRegistry::default();
    let selected = key("custody-unrelated-0", 1, 100);
    let ready = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
    let mut state = (*registry.state_for_test()).lock().unwrap();
    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_live_custody(&mut state, &receipts, 1);
    for mut row in state.records.values_mut() {
        row.interests = 0;
        row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
            WorthQueryOutputDemandDenialKind::NoEffect,
            "completed",
        ));
    }
    for (index, receipt) in receipts.iter().enumerate() {
        let commit = receipt.committed_product_publication().composite_commit();
        state.source_custody.get_mut(commit).unwrap().token_count =
            usize::from(token || index != 0);
    }
    if !token {
        let mut row = state.records.get_mut(&selected).unwrap();
        row.state = ready;
        row.interests = 1;
    }
    assert!(!state.pending_custody_prune());
    state
        .source_custody
        .get_mut(
            receipts[0]
                .committed_product_publication()
                .composite_commit(),
        )
        .unwrap()
        .completed = true;
    drop(state);
    registry.complete_prepared_source_for_test(&receipts[0]);
    (registry, selected, receipts)
}

#[test]
fn registry_interest_drop_leaves_custody_note_without_sweeping() {
    let (registry, selected, receipts) = fixture(false);
    let wake = Arc::clone(&(*registry.state_for_test()).lock().unwrap().records[&selected].wake);
    let owner = interest(&registry, selected, wake);
    let before = contacts();
    drop(owner);
    assert_eq!(
        contacts(),
        before,
        "Drop does not inspect any custody entry"
    );
    let commit = receipts[0]
        .committed_product_publication()
        .composite_commit();
    let mut state = (*registry.state_for_test()).lock().unwrap();
    assert!(state.pending_custody_prune());
    assert!(state.source_custody[commit].cleanup.queued);
    state.drain_completed_custody();
    assert!(!state.source_custody.contains_key(commit));
    assert!(!state.pending_custody_prune());
}

#[test]
fn registry_prepared_token_drop_leaves_custody_note_without_sweeping() {
    let (registry, _, receipts) = fixture(true);
    let commit = receipts[0]
        .committed_product_publication()
        .composite_commit();
    let before = contacts();
    registry.release_prepared_token_for_test(commit);
    assert_eq!(contacts(), before, "token Drop does not inspect custody");
    let mut state = (*registry.state_for_test()).lock().unwrap();
    assert_eq!(state.source_custody[commit].token_count, 0);
    assert!(state.source_custody[commit].cleanup.queued);
    state.drain_completed_custody();
    assert!(!state.source_custody.contains_key(commit));
    assert!(!state.pending_custody_prune());
}
