//! Canonical occurrence searches keep their producer/source boundary.

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::awaited_by_stale_owner;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::newest_of_occurrence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::occurrence_rows;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support;
pub fn check(search: usize) {
    let mut contacts = Vec::new();
    let receipts = receipt_batch();
    for (neighbors, entries) in [(1, 1), (64, 4)] {
        let mut state = DemandRegistryState::default();
        let selected = key("own", 1, 1);
        for generation in 1..=3 {
            (*state.records_mut_for_test()).insert_for_test(
                key("own", generation, 1),
                record(occurrence(), DemandState::Admitted, 0),
            );
        }
        for generation in 1..=neighbors {
            (*state.records_mut_for_test()).insert_for_test(
                key("larger-other", generation, 2),
                record(occurrence(), DemandState::Admitted, 0),
            );
        }
        add_custody(&mut state, &receipts[..entries], neighbors);
        let before = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited();
        match search {
            0 => assert_eq!(occurrence_rows(&(*state.records_for_test()), &selected).count(), 3),
            1 => assert_eq!(
                newest_of_occurrence(&(*state.records_for_test()), &selected),
                Some(key("own", 3, 1))
            ),
            2 => assert!(!awaited_by_stale_owner(&(*state.records_for_test()), &selected)),
            _ => panic!("unknown occurrence search"),
        }
        contacts.push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited() - before);
    }
    assert_eq!(contacts, vec![3, 3], "search {search}");
}
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
pub fn receipt_batch() -> Vec<WorthQueryApplicationCommitReceipt> {
    (0..4)
        .map(|_| crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application())
        .collect()
}
pub fn add_custody(
    state: &mut DemandRegistryState,
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
) {
    let (scope, _) = crate::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence();
    add_custody_in_scope(state, receipts, rows, scope);
}
pub fn add_custody_in_scope(
    state: &mut DemandRegistryState,
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
    scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
) {
    add_custody_with_completion(state, receipts, rows, scope, true);
}
pub fn add_live_custody(
    state: &mut DemandRegistryState,
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
) {
    let (scope, _) = crate::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence();
    add_custody_with_completion(state, receipts, rows, scope, false);
}
pub fn add_custody_with_completion(
    state: &mut DemandRegistryState,
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
    scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
    completed: bool,
) {
    for (i, receipt) in receipts.iter().enumerate() {
        let commit = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        let selected = key(
            &format!("custody-unrelated-{i}"),
            1,
            u8::try_from(i).unwrap() + 100,
        );
        for generation in 1..=rows {
            let mut row = record(
                receipt.product_branch().occurrence_for_test(),
                DemandState::Admitted,
                1,
            );
            (*row.source_commits_mut_for_test()).push(commit.clone());
            (*row.source_scope_mut_for_test()) = Some(scope);
            (*state.records_mut_for_test()).insert_for_test(
                key(
                    &(*selected.producer_for_test()),
                    generation,
                    u8::try_from(i).unwrap() + 100,
                ),
                row,
            );
        }
        state.source_custody.insert(
            commit.clone(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody {
                occurrence: receipt.product_branch().occurrence_for_test(),
                root_kind: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind::Discovered(
                    worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId::of_for_test::<()>(),
                ),
                source: None,
                discovery: None,
                bound_sources: Some(vec![worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource {
                    scope,
                    identity: (*selected.source_for_test()).clone(),
                }]),
                consumed_sources: vec![selected.source_into_for_test()],
                retired_sources: Vec::new(),
                retired: None,
                token_count: 0,
                cleanup: Default::default(),
                completed,
            },
        );
        if completed {
            state.note_custody_for_test(&commit);
        }
    }
}
