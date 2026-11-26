//! Changed-slot retirement shares the catalog's real installed-entry authority.
use super::super::CompositeHistoryCatalog;
use super::fixtures::{history_contract, linear_history};
use std::num::NonZeroUsize;

#[path = "automatic_retirement/cost.rs"]
mod cost;

#[test]
fn a_released_gap_splices_behind_held_endpoints_without_losing_generation_span() {
    let (fixture, commits) = linear_history(5);
    let catalog = CompositeHistoryCatalog::new(
        commits[0].identity().owner_identity(),
        history_contract(5, u64::MAX),
    );
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let _first = catalog.protect_explicit_commit(&commits[0]).unwrap();
    let middle = catalog.protect_explicit_commit(&commits[2]).unwrap();
    let _last = catalog.protect_explicit_commit(&commits[4]).unwrap();
    drop(catalog.retire_changed_history());
    assert_eq!(catalog.len(), 3);
    assert!(catalog.lookup(commits[1].identity()).is_none());
    assert!(catalog.lookup(commits[3].identity()).is_none());
    drop(middle);
    let retired = catalog.retire_changed_history();
    assert_eq!(retired.len(), 1);
    drop(retired);
    let trace = catalog
        .trace_ancestry(commits[4].identity().clone(), NonZeroUsize::new(3).unwrap())
        .unwrap();
    assert_eq!(trace.visited_count(), 2);
    assert_eq!(trace.generations_to(1), Some(4));
    assert!(trace.crossed_retired_history());
    assert!(catalog.lookup(commits[0].identity()).is_some());
    assert!(catalog.lookup(commits[4].identity()).is_some());
}

#[test]
fn forming_and_sibling_dependencies_keep_their_actual_parents_until_release() {
    let (mut fixture, commits) = linear_history(2);
    let catalog = CompositeHistoryCatalog::new(
        commits[0].identity().owner_identity(),
        history_contract(8, u64::MAX),
    );
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let sibling = fixture.child_of(&commits[0]);
    catalog
        .append(sibling.clone(), fixture.history_pins(&sibling))
        .unwrap();
    let sibling_hold = catalog.protect_explicit_commit(&sibling).unwrap();
    let forming = fixture.child_of(&commits[1]);
    let reserved = catalog.reserve(&forming).unwrap();
    assert!(
        catalog.retire_changed_history().is_empty(),
        "a sole forming dependency is not an installed child"
    );
    assert!(
        catalog.lookup(commits[0].identity()).is_some(),
        "a real sibling keeps the fork point"
    );
    reserved
        .install(forming.clone(), fixture.history_pins(&forming))
        .unwrap();
    let retired = catalog.retire_changed_history();
    assert_eq!(
        retired.len(),
        1,
        "installing the child makes its unprotected parent eligible"
    );
    drop(retired);
    assert!(catalog.lookup(commits[1].identity()).is_none());
    assert!(catalog.lookup(commits[0].identity()).is_some());
    drop(sibling_hold);
    let (released, held) = catalog
        .release_retired_branch_history(sibling.identity())
        .unwrap();
    assert_eq!(held, None);
    assert_eq!(released.len(), 1);
    drop(released);
    let retired = catalog.retire_changed_history();
    assert_eq!(
        retired.len(),
        1,
        "branch removal wakes its now single-child parent"
    );
    drop(retired);
    assert_eq!(catalog.len(), 1);
    assert!(catalog.lookup(forming.identity()).is_some());
}

#[test]
fn manual_ancestry_scope_and_owner_wide_changed_scope_remain_distinct() {
    let (mut fixture, commits) = linear_history(2);
    let catalog = CompositeHistoryCatalog::new(
        commits[0].identity().owner_identity(),
        history_contract(8, u64::MAX),
    );
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let sibling = fixture.child_of(&commits[0]);
    let sibling_head = fixture.child_of(&sibling);
    for commit in [&sibling, &sibling_head] {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let _main = catalog.protect_explicit_commit(&commits[1]).unwrap();
    let _sibling = catalog.protect_explicit_commit(&sibling_head).unwrap();
    assert!(catalog
        .retire_unprotected_history(commits[1].identity())
        .unwrap()
        .is_empty());
    assert!(
        catalog.lookup(sibling.identity()).is_some(),
        "the manual walk cannot retire sibling history"
    );
    let retired = catalog.retire_changed_history();
    assert_eq!(retired.len(), 1);
    drop(retired);
    assert!(catalog.lookup(sibling.identity()).is_none());
    assert!(catalog.lookup(sibling_head.identity()).is_some());
    assert!(catalog.lookup(commits[1].identity()).is_some());
}

#[test]
fn external_history_ordinals_defer_interior_retirement_in_o1_until_released() {
    let (fixture, commits) = linear_history(4);
    let catalog = CompositeHistoryCatalog::new(
        commits[0].identity().owner_identity(),
        history_contract(4, u64::MAX),
    );
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let _head = catalog.protect_explicit_commit(&commits[3]).unwrap();
    let retained_ordinal = commits[1].identity().ordinal();
    catalog.retain_external_history_ordinals(&[retained_ordinal]);

    // Single durable pass with BOTH excluded (`commits[1]`) and non-excluded
    // (`commits[0]`, `commits[2]`) eligible commits skips `commits[1]`.
    let pending = catalog.begin_durable_retire_changed_history();
    assert_eq!(pending.remaining().len(), 2);
    assert!(!pending.remaining().contains(commits[1].identity()));
    // Abort once via ProvenNoEffect and re-begin cleanly.
    pending.release_remaining_proven_no_effect();
    let mut pending = catalog.begin_durable_retire_changed_history();
    assert_eq!(pending.remaining().len(), 2);
    let retired = pending.commit_next(2);
    assert_eq!(retired.len(), 2);
    drop(retired);
    drop(pending);

    assert!(catalog.lookup(commits[0].identity()).is_none());
    assert!(catalog.lookup(commits[1].identity()).is_some());
    assert!(catalog.lookup(commits[2].identity()).is_none());
    assert!(catalog.lookup(commits[3].identity()).is_some());

    // Spliced ancestry crosses the retired gap (`commits[2]`) and lands on `commits[1]`
    // with generation span 2, stopping cleanly because `commits[0]` is retired.
    let trace = catalog
        .trace_ancestry(commits[3].identity().clone(), NonZeroUsize::new(4).unwrap())
        .unwrap();
    assert_eq!(trace.visited_count(), 2);
    assert_eq!(trace.generations_to(1), Some(2));
    assert!(trace.crossed_retired_history());
    drop(trace);

    // Unchanged externally retained commits do NOT stay in the dirty frontier (O(1) cost).
    let lookups_before = catalog.counters().reachability_lookups();
    assert!(catalog.retire_changed_history().is_empty());
    assert!(catalog
        .begin_durable_retire_changed_history()
        .remaining()
        .is_empty());
    assert_eq!(
        catalog.counters().reachability_lookups(),
        lookups_before,
        "externally retained commits must not churn the dirty frontier"
    );

    // Releasing the external ordinal wakes the commit once so durable retirement reclaims it.
    catalog.release_external_history_ordinals(&[retained_ordinal]);
    let mut pending_released = catalog.begin_durable_retire_changed_history();
    assert_eq!(
        pending_released.remaining(),
        std::slice::from_ref(commits[1].identity())
    );
    let retired_after_release = pending_released.commit_next(1);
    assert_eq!(retired_after_release.len(), 1);
    drop(retired_after_release);
    drop(pending_released);
    assert_eq!(catalog.len(), 1);
    assert!(catalog.lookup(commits[1].identity()).is_none());
    assert!(catalog.lookup(commits[3].identity()).is_some());
}
