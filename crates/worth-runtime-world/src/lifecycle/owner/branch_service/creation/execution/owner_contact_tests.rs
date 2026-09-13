//! A creation's cost counters must name the owners it actually asked for work.
//!
//! The two-by-two creation cell is the only place where the same reserved
//! attempt can contact both owners, one owner, or neither, so the contact
//! counters are proved at the execution seam that decides it: the settled
//! attempt still carries the counters the fork legs recorded on, and the plan
//! is the only thing that separates the fork case from the reuse case.

use crate::branch::{RelationalBranchCreationPlan, SignalBranchCreationPlan};
use crate::lifecycle::owner::branch_service::tests::fork_creation::{
    fork_intent, relational_fork, setup_with_relational_source, signal_fork,
};
use crate::lifecycle::RuntimeWorldPreparationService;
use crate::publication::{ReservedBranchCreationAttempt, RuntimeWorldCancellationSource};

use super::{execute_creation, BranchCreationExecution, CreationDestination};

/// Drive one creation to its owner-execution terminal and hand back the
/// settled attempt, which is the only holder of the counters the fork legs
/// recorded on.
fn settled_creation_counters(
    relational: RelationalBranchCreationPlan,
    signal: SignalBranchCreationPlan,
    name: &str,
) -> (u64, u64) {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let cancellation = RuntimeWorldCancellationSource::new();
    let intent = fork_intent(name, relational, signal);
    let mut reservation = owner
        .state
        .branches
        .reserve_branch(owner.owner_identity(), intent.name().clone())
        .expect("the destination name is reserved before effects");
    let (branch, incarnation) = owner
        .issue_branch_identities(intent.name().clone())
        .expect("the destination identities are issued before any owner effect");
    let attempt = RuntimeWorldPreparationService::prepare_creation(
        &owner,
        source,
        intent,
        &cancellation.token(),
        None,
    )
    .expect("a creation from the seeded head reserves its bounded resources");
    assert_eq!(
        contacts(&attempt),
        (0, 0),
        "a reserved attempt has contacted no owner yet"
    );
    let destination = CreationDestination {
        witness: reservation
            .bind_creation_destination(branch.clone(), incarnation)
            .expect("the reserved destination binds its exact witness"),
        branch,
        incarnation,
    };
    match execute_creation(&owner, attempt, &destination, &cancellation.token()) {
        BranchCreationExecution::Settled { attempt, .. } => contacts(&attempt),
        BranchCreationExecution::NoEffect(denial) => {
            panic!("an admitted creation must reach its owner terminal: {denial:?}")
        }
        BranchCreationExecution::ProductUnpublished(_) => {
            panic!("no sibling denied this creation, so nothing may be retained")
        }
    }
}

fn contacts(attempt: &ReservedBranchCreationAttempt) -> (u64, u64) {
    (
        attempt.counters().relational_owner_contacts(),
        attempt.counters().signal_owner_contacts(),
    )
}

#[test]
fn a_two_owner_fork_creation_records_one_contact_per_owner() {
    assert_eq!(
        settled_creation_counters(
            relational_fork("relational-creation-contact"),
            signal_fork("signal-creation-contact"),
            "creation-contacts-fork",
        ),
        (1, 1),
        "each forking owner is asked for work exactly once"
    );
}

#[test]
fn an_exact_reuse_creation_records_no_owner_contact() {
    assert_eq!(
        settled_creation_counters(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
            "creation-contacts-reuse",
        ),
        (0, 0),
        "reuse selects the commit the source already names, so no owner is asked"
    );
}

/// One forking owner and one reusing owner is the case a single shared counter
/// would report wrongly: the Relational leg moved and the Signal leg did not.
#[test]
fn a_single_owner_fork_records_only_that_owner() {
    assert_eq!(
        settled_creation_counters(
            relational_fork("relational-only-creation-contact"),
            SignalBranchCreationPlan::ReuseExact,
            "creation-contacts-relational-only",
        ),
        (1, 0),
        "only the owner the plan asked to fork is contacted"
    );
}
