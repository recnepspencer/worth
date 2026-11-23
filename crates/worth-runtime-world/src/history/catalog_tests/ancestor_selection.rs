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
        .select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target.identity(),
            commits[0].identity(),
            None,
        )
        .expect("the source before the fork is an actual ancestor");
    assert_eq!(selected.source().identity(), commits[0].identity());
    assert_eq!(selected.target(), target.identity());
    assert_eq!(selected.parent_links_followed(), 3);
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target.identity(),
            sibling.identity(),
            None,
        ),
        Err(Denial::NotAncestor)
    ));
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target.identity(),
            commits[0].identity(),
            NonZeroUsize::new(2),
        ),
        Err(Denial::SelectionLimitReached)
    ));
    let missing = fixture.child_of(&target);
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target.identity(),
            missing.identity(),
            None,
        ),
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
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
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
        .select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            source,
            source,
            NonZeroUsize::new(1),
        )
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
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            commits[0].identity(),
            source,
            None,
        ),
        Err(Denial::SourceUnavailable)
    ));
}

#[test]
fn retiring_interior_commit_fences_ancestry_exact_position_and_publication_reads() {
    let (mut fixture, catalog, p) = crate::branch::reference_test_fixture::installed_root();
    let p_guard = catalog.protect_explicit_commit(p.as_ref()).expect("protect P");
    let q = crate::branch::reference_test_fixture::install_ordinary(&mut fixture, &catalog, p.as_ref());
    let r = crate::branch::reference_test_fixture::install_ordinary(&mut fixture, &catalog, q.as_ref());
    let r_guard = catalog.protect_explicit_commit(r.as_ref()).expect("protect R");
    let policy = worth_foundational::ExecutionRequestPolicy::for_workers(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        NonZeroUsize::MIN,
    );
    let serial = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let mut pending = catalog
        .begin_durable_retire_unprotected_history(r.identity())
        .expect("reserve interior Q");
    assert_eq!(pending.remaining(), &[q.identity().clone()]);
    assert!(matches!(
        catalog.protect_explicit_commit(q.as_ref()),
        Err(crate::history::CompositeHistoryCatalogDenial::UnknownProtectionTarget(_))
    ));
    assert!(matches!(
        catalog.retain_exact_history_position(execution, q.identity()),
        Err(Denial::SourceUnavailable)
    ));
    let retained_r = catalog
        .retain_exact_history_position(execution, r.identity())
        .expect("R itself remains retainable");
    assert!(matches!(
        catalog.next_exact_history_position(execution, &retained_r),
        Err(Denial::RetiredHistoryGap)
    ));
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            r.identity(),
            q.identity(),
            None,
        ),
        Err(Denial::SourceUnavailable)
    ));
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Exact,
            &execution,
            r.identity(),
            p.identity(),
            None,
        ),
        Err(Denial::RetiredHistoryGap)
    ));
    assert!(matches!(
        catalog.protect_performed_publication(q.identity()),
        Err(crate::history::CompositeHistoryCatalogDenial::UnknownProtectionTarget(_))
    ));
    let traversal = catalog
        .trace_ancestry(r.identity().clone(), NonZeroUsize::new(10).unwrap())
        .expect("trace ancestry from R while Q is retiring");
    assert_eq!(traversal.visited_count(), 1);
    assert!(traversal.crossed_retired_history());
    assert_eq!(traversal.next_parent(), Some(q.identity()));
    let retired = pending.commit_next(1);
    assert_eq!(retired.len(), 1);
    assert!(pending.finish_all().is_none());
    drop((retired, traversal, retained_r));
    assert!(catalog.lookup(q.identity()).is_none());
    let after = catalog
        .trace_ancestry(r.identity().clone(), NonZeroUsize::new(10).unwrap())
        .expect("trace spliced ancestry after commit");
    assert_eq!(after.visited_count(), 2);
    assert!(after.crossed_retired_history());
    assert_eq!(after.generations_to(0), Some(0));
    assert_eq!(after.generations_to(1), Some(2));
    drop((after, r_guard, p_guard));
}

#[path = "ancestor_selection/splicing.rs"]
mod splicing;
