//! Exact inherited ancestry, bounded refusal and selected-occurrence custody.
use std::num::NonZeroUsize;

use super::fixtures::{history_contract, linear_history};
use crate::history::{
    CompositeAncestorSelectionDenial as Denial, CompositeHistoryCatalog,
    CompositeHistoryCatalogDenial, CompositeHistoryReclamationRequest,
};

#[test]
fn selection_follows_fork_parent_links_and_distinguishes_incomplete_from_sibling() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (mut fixture, commits) = linear_history(3);
    let fork = fixture.child_of(&commits[1]);
    let target = fixture.child_of(&fork);
    let sibling = fixture.child_of(&commits[1]);
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(6, u64::MAX));
    for commit in commits.iter().chain([&fork, &target, &sibling]) {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let selected = catalog
        .select_retained_ancestor(&execution, target.identity(), commits[0].identity(), None)
        .expect("the source before the fork is an actual ancestor");
    assert_eq!(selected.source().identity(), commits[0].identity());
    assert_eq!(selected.target(), target.identity());
    assert_eq!(selected.parent_links_followed(), 3);
    assert!(matches!(
        catalog.select_retained_ancestor(&execution, target.identity(), sibling.identity(), None,),
        Err(Denial::NotAncestor)
    ));
    assert!(matches!(
        catalog.select_retained_ancestor(
            &execution,
            target.identity(),
            commits[0].identity(),
            NonZeroUsize::new(2),
        ),
        Err(Denial::SelectionLimitReached)
    ));
    let missing = fixture.child_of(&target);
    assert!(matches!(
        catalog.select_retained_ancestor(&execution, target.identity(), missing.identity(), None,),
        Err(Denial::SourceUnavailable)
    ));

    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    assert!(matches!(
        catalog.select_retained_ancestor(
            &worth_execution::ExecutionRequest::serial(&stopped),
            target.identity(),
            commits[0].identity(),
            None,
        ),
        Err(Denial::Catalog(CompositeHistoryCatalogDenial::Cancelled))
    ));
}

#[test]
fn selected_source_is_protected_and_retired_source_cannot_be_reselected() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (fixture, commits) = linear_history(2);
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(2, u64::MAX));
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let source = commits[1].identity();
    let selected = catalog
        .select_retained_ancestor(&execution, source, source, NonZeroUsize::new(1))
        .unwrap();
    assert_eq!(selected.parent_links_followed(), 0);
    let reclaim = || CompositeHistoryReclamationRequest::new(owner, vec![source.clone()], 1);
    assert_eq!(
        catalog
            .reclaim_batch(reclaim())
            .unwrap()
            .skipped_protected(),
        1
    );
    drop(selected);
    assert_eq!(
        catalog
            .reclaim_batch(reclaim())
            .unwrap()
            .reclaimed_commits(),
        std::slice::from_ref(source)
    );
    assert!(matches!(
        catalog.select_retained_ancestor(&execution, commits[0].identity(), source, None,),
        Err(Denial::SourceUnavailable)
    ));
}
