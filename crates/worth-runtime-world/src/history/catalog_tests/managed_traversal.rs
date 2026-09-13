use super::fixtures::{history_contract, linear_history};
use crate::history::{CompositeHistoryCatalog, CompositeHistoryReclamationRequest};
use std::num::NonZeroUsize;
#[test]
fn bounded_traversal_protects_chain_and_reuses_exact_history_leases() {
    let (fixture, commits) = linear_history(3);
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(3, u64::MAX));
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
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
