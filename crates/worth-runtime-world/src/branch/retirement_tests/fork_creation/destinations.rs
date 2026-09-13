//! Each owner fork of a creation reserves the exact destination its plan named
//! and then consumes that reservation. A destination another holder already
//! owns denies the creation, and the denial releases everything it charged.

use super::*;

const DUPLICATE_SIGNAL_TARGET: &str = "signal-branch-duplicate-destination";

#[test]
fn duplicate_signal_destination_denies_before_effect_and_retries_after_drop() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let held = owner
        .state
        .signal
        .mutation_port()
        .reserve_fork_exact(
            validate_signal_branch_name(DUPLICATE_SIGNAL_TARGET).expect("valid Signal name"),
            source.basis().signal_basis(),
        )
        .expect("the first owner-issued reservation holds the destination");
    let lifecycles_before = super::super::owner_lifecycles(&owner);
    let cancellation = RuntimeWorldCancellationSource::new();
    let denial = RuntimeWorldBranchService::create_product_branch(
        &owner,
        RuntimeWorldBranchCreationRequest::new(
            source.clone(),
            duplicate_intent(),
            &cancellation.token(),
        ),
    )
    .expect_err("a held destination denies the only owner this creation asks to move");

    assert_eq!(denial, RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
    assert_eq!(
        super::super::owner_lifecycles(&owner),
        lifecycles_before,
        "a denied sole fork leaves both component owners exactly where they were"
    );
    assert_eq!(
        owner.state.custody.installed(),
        0,
        "a fork that never happened is never in custody"
    );
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.recovery_record_count(), 0);

    drop(held);
    let history_before = owner.state.history.len();
    let child = create_forked_branch(&owner, &source, duplicate_intent());
    assert_new_branch_observation(&owner, &source, &child, history_before);
    assert_eq!(owner.state.custody.installed(), 1);
}

/// The same creation both times: only the release of the held destination can
/// explain the difference between the two outcomes.
fn duplicate_intent() -> ProductBranchCreationIntent {
    fork_intent(
        "branch-duplicate-destination",
        RelationalBranchCreationPlan::ReuseExact,
        signal_fork(DUPLICATE_SIGNAL_TARGET),
    )
}

#[test]
fn relational_fork_exact_consumes_the_owner_reservation_and_returns_target_basis() {
    let (fixture, owner, source) = setup_with_relational_source(3);
    let target = "relational-branch-consumed-reservation";
    assert!(
        fixture.reserve_relational_fork_target(target).is_ok(),
        "the destination is free before the creation asks for it"
    );
    let history_before = owner.state.history.len();
    let child = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-relational-consumed-reservation",
            relational_fork(target),
            SignalBranchCreationPlan::ReuseExact,
        ),
    );

    assert_new_branch_observation(&owner, &source, &child, history_before);
    assert_eq!(
        child.basis().relational_basis().descriptor().branch_id(),
        &BranchId(target.to_owned()),
        "the creation adopts the target basis the owner returned, not the source one"
    );
    assert_ne!(
        child.basis().relational_basis().descriptor().branch_id(),
        source.basis().relational_basis().descriptor().branch_id()
    );
    assert!(
        fixture.reserve_relational_fork_target(target).is_err(),
        "the reservation the creation consumed became a real branch"
    );
}

#[test]
fn signal_fork_exact_reserves_then_consumes_without_an_advance() {
    let (fixture, owner, source) = setup_with_relational_source(3);
    let target = "signal-branch-consumed-reservation";
    let signal_before = fixture.observe_signal_current_basis();
    let history_before = owner.state.history.len();
    let child = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-signal-consumed-reservation",
            RelationalBranchCreationPlan::ReuseExact,
            signal_fork(target),
        ),
    );

    assert_new_branch_observation(&owner, &source, &child, history_before);
    assert_ne!(
        child.basis().signal_basis().admission_identity(),
        source.basis().signal_basis().admission_identity(),
        "a Signal fork issues a new admitted basis for its destination"
    );
    assert_eq!(
        fixture.observe_signal_current_basis().admission_identity(),
        signal_before.admission_identity(),
        "forking a destination is not an advance of the branch it forked from"
    );
    assert!(
        owner
            .state
            .signal
            .mutation_port()
            .reserve_fork_exact(
                validate_signal_branch_name(target).expect("valid Signal name"),
                source.basis().signal_basis(),
            )
            .is_err(),
        "the reservation the creation consumed became a real branch"
    );
}
