use super::fixtures::{history_contract, linear_history};
use crate::history::{CompositeHistoryCatalog, CompositeHistoryReclamationRequest};
use std::num::NonZeroUsize;
#[test]
fn bounded_traversal_protects_chain_and_reuses_exact_history_leases() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (fixture, commits) = linear_history(3);
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(3, u64::MAX));
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    // Absence traverses the genuine retained chain, without a numeric sentinel.
    let full = catalog
        .trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution,
                maximum: None,
            },
            commits[2].identity().clone(),
            None,
        )
        .unwrap();
    assert_eq!(full.visited_count(), 3);
    assert!(full.is_complete());
    assert_eq!(
        full.commits().last().unwrap().identity(),
        commits[0].identity()
    );
    drop(full);
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        None,
    );
    let held = fixture.retention.active_component_obligation_count();
    assert!(matches!(
        catalog.trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution: worth_execution::ExecutionRequest::serial(&stopped),
                maximum: None
            },
            commits[2].identity().clone(),
            None
        ),
        Err(crate::history::CompositeHistoryCatalogDenial::Cancelled)
    ));
    assert_eq!(fixture.retention.active_component_obligation_count(), held);
    let full = catalog
        .trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution,
                maximum: None,
            },
            commits[2].identity().clone(),
            None,
        )
        .unwrap();
    assert_eq!(
        full.visited_count(),
        3,
        "fresh original request retains the same native history"
    );
    drop(full);
    let before = fixture.retention.cost_snapshot();
    let traversal = catalog
        .trace_ancestry(commits[2].identity().clone(), NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(traversal.visited_count(), 1);
    assert_eq!(traversal.next_parent(), Some(commits[1].identity()));
    assert_eq!(
        traversal.commits().next().unwrap().identity(),
        commits[2].identity()
    );
    let request =
        || CompositeHistoryReclamationRequest::new(owner, vec![commits[2].identity().clone()], 1);
    assert_eq!(
        catalog
            .reclaim_batch(request())
            .unwrap()
            .skipped_protected(),
        1
    );
    assert_eq!(fixture.retention.unique_pin_count(), 2);
    assert_eq!(fixture.retention.active_component_obligation_count(), 6);
    assert_eq!(
        fixture
            .retention
            .cost_snapshot()
            .owner_acquisition_contacts(),
        before.owner_acquisition_contacts()
    );
    drop(traversal);
    assert_eq!(
        catalog
            .reclaim_batch(request())
            .unwrap()
            .reclaimed_commits()
            .len(),
        1
    );
    assert_eq!(fixture.retention.active_component_obligation_count(), 4);
    let traversal = catalog
        .trace_ancestry(commits[1].identity().clone(), NonZeroUsize::new(2).unwrap())
        .unwrap();
    drop(catalog);
    assert_eq!(traversal.visited_count(), 2);
    assert_eq!(fixture.retention.active_component_obligation_count(), 4);
    drop(traversal);
    assert_eq!(fixture.retention.active_component_obligation_count(), 0);
}
