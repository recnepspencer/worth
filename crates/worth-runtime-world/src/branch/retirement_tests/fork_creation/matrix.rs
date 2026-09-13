//! The two mixed cells of the creation matrix. Exactly one owner is asked to
//! move, and the other must not be contacted at all: its component basis is the
//! one the source already names, and its lifecycle never advances.

use super::*;

use crate::identity::ProductBranchIdentity;

/// The created branch is the name-keyed identity of the intent, on a fresh
/// incarnation of its own.
fn assert_name_keyed_identity(
    owner: &TestOwner,
    child: &ProductBranchObservation,
    intent_name: &crate::branch::ProductBranchName,
) {
    assert_eq!(
        child.branch_identity(),
        &ProductBranchIdentity::issued(owner.owner_identity(), intent_name.clone()),
        "a created branch is the name-keyed identity its intent asked for"
    );
    assert_eq!(child.reference_generation().get(), 0);
}

/// The matrix cell where only the Relational owner is asked to move: the Signal
/// component of the child names the exact commit the source already names.
#[test]
fn relational_fork_with_signal_reuse_moves_exactly_one_owner() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let history_before = owner.state.history.len();
    let intent = fork_intent(
        "branch-relational-fork-only",
        relational_fork("relational-branch-fork-only"),
        SignalBranchCreationPlan::ReuseExact,
    );
    let name = intent.name().clone();
    let costs_before = owner.state.retention.cost_snapshot();
    let child = create_forked_branch(&owner, &source, intent);

    assert_new_branch_observation(&owner, &source, &child, history_before);
    assert_name_keyed_identity(&owner, &child, &name);
    assert_ne!(
        child.basis().relational_basis().identity(),
        source.basis().relational_basis().identity()
    );
    assert_eq!(
        child.basis().signal_basis().admission_identity(),
        source.basis().signal_basis().admission_identity()
    );
    let costs_after = owner.state.retention.cost_snapshot();
    assert_eq!(
        costs_after.signal_contacts(),
        costs_before.signal_contacts(),
        "an exactly reused Signal component is never contacted for a creation"
    );
    assert!(costs_after.relational_contacts() > costs_before.relational_contacts());
    assert_eq!(
        owner.state.custody.installed(),
        1,
        "only the owner that really created a component branch is in custody"
    );
    assert_eq!(
        owner.state.custody.installed_records()[0].component(),
        crate::branch::CustodyComponent::Relational
    );
}

/// The mirrored cell: only the Signal owner forks, and the Relational component
/// is reused exactly.
#[test]
fn signal_fork_with_relational_reuse_moves_exactly_one_owner() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let history_before = owner.state.history.len();
    let intent = fork_intent(
        "branch-signal-fork-only",
        RelationalBranchCreationPlan::ReuseExact,
        signal_fork("signal-branch-fork-only"),
    );
    let name = intent.name().clone();
    let costs_before = owner.state.retention.cost_snapshot();
    let child = create_forked_branch(&owner, &source, intent);

    assert_new_branch_observation(&owner, &source, &child, history_before);
    assert_name_keyed_identity(&owner, &child, &name);
    assert_eq!(
        child.basis().relational_basis().identity(),
        source.basis().relational_basis().identity()
    );
    assert_ne!(
        child.basis().signal_basis().admission_identity(),
        source.basis().signal_basis().admission_identity()
    );
    let costs_after = owner.state.retention.cost_snapshot();
    assert_eq!(
        costs_after.relational_contacts(),
        costs_before.relational_contacts(),
        "an exactly reused Relational component is never contacted for a creation"
    );
    assert!(costs_after.signal_contacts() > costs_before.signal_contacts());
    assert_eq!(
        owner.state.custody.installed(),
        1,
        "only the owner that really created a component branch is in custody"
    );
    assert_eq!(
        owner.state.custody.installed_records()[0].component(),
        crate::branch::CustodyComponent::Signal
    );
}
