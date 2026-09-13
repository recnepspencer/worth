//! Exact reuse installs the commit the source observation names, and only
//! while the branch still carries it.
//!
//! The reference cell is the only authority for a product head. A reuse that
//! resolved its commit through a basis index derived from the cell lagged
//! every movement by one re-index and, in that window, denied a creation from
//! a head the owner itself had just published. Resolving from the observation
//! the cell issued removes that window: a source that is current takes its
//! exact commit, and a source already displaced is refused as a stale source
//! head before any capacity is charged. A movement that lands after this
//! admission is refused at the installation itself, under the source cell's
//! guard; that window is pinned in `source_guarded_install`.

use super::fork_creation::{seed_relational_source, setup_with_relational_source};
use super::{create_reused_branch, owner_lifecycles, reuse_intent};
use crate::branch::RuntimeWorldBranchAdmissionDenial;
use crate::lifecycle::{RuntimeWorldBranchCreationRequest, RuntimeWorldBranchService};
use crate::publication::RuntimeWorldCancellationSource;

#[test]
fn exact_reuse_after_a_publication_selects_the_published_commit() {
    let (_fixture, owner, published) = setup_with_relational_source(3);
    let root_commit = published.selected_commit().clone();

    let child = create_reused_branch(&owner, &published, reuse_intent("published-head-child"));

    assert_eq!(
        child.selected_commit(),
        &root_commit,
        "exact reuse selects the commit the published head names"
    );
    assert_eq!(
        child.basis(),
        published.basis(),
        "the child carries the published composite basis unchanged"
    );
    assert_ne!(child.branch_identity(), published.branch_identity());
    assert_eq!(owner.state.branches.branch_count(), 2);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(
        owner.state.history.len(),
        2,
        "reuse publishes nothing: the seeded and bootstrap commits are all there is"
    );
}

/// The mirrored case: the source observation names a head the branch has
/// since moved past. Nothing resolves it to some other occurrence carrying the
/// same basis and nothing calls it an unavailable owner; the creation is
/// refused as a stale source head and charges nothing.
#[test]
fn exact_reuse_from_a_displaced_source_head_denies_as_stale_before_any_charge() {
    let (mut fixture, owner, first) = setup_with_relational_source(3);
    let held_source = first.clone();
    seed_relational_source(&owner, &mut fixture, first);
    let lifecycles_before = owner_lifecycles(&owner);
    let costs_before = owner.state.retention.cost_snapshot();
    let pins_before = owner.state.retention.unique_pin_count();
    let cancellation = RuntimeWorldCancellationSource::new();

    let denial = RuntimeWorldBranchService::create_product_branch(
        &owner,
        RuntimeWorldBranchCreationRequest::new(
            held_source.clone(),
            reuse_intent("displaced-head-child"),
            &cancellation.token(),
        ),
    )
    .expect_err("a displaced source head denies an exact reuse");

    assert_eq!(denial, RuntimeWorldBranchAdmissionDenial::StaleSourceHead);
    assert_eq!(owner_lifecycles(&owner), lifecycles_before);
    assert_eq!(owner.state.retention.cost_snapshot(), costs_before);
    assert_eq!(owner.state.retention.unique_pin_count(), pins_before);
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    // The caller's own observation is the only custody in play; it is held
    // across the assertions so that its release is not mistaken for a charge.
    drop(held_source);
}

#[test]
fn retirement_accepts_an_older_head_from_the_same_installed_occurrence() {
    let (mut fixture, owner, first) = setup_with_relational_source(3);
    seed_relational_source(&owner, &mut fixture, first.clone());
    let current = super::RuntimeWorldObservationService::observe_product_branch(
        &owner,
        first.branch_identity(),
    )
    .unwrap();
    assert_ne!(first.selected_commit(), current.selected_commit());
    assert_eq!(
        first.lifecycle_incarnation(),
        current.lifecycle_incarnation()
    );
    let report = owner
        .retire_product_branch(&first)
        .expect("the live occurrence accepts an older observation");
    assert_eq!(report.retired_head(), current.selected_commit());
    assert_ne!(report.retired_head(), first.selected_commit());
    assert!(report.owner_retirement_work().is_empty());
    assert!(matches!(
        owner.retire_product_branch(&current),
        Err(super::RuntimeWorldBranchRetirementDenial::AlreadyRetired)
    ));
    assert_eq!(owner.state.branches.branch_count(), 0);
}
