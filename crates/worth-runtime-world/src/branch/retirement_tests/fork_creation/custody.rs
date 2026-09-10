//! Owner-created component branches are the only creations Runtime World is in
//! custody of, and retirement is where that custody is discharged: as typed
//! work for the component owner, never as a deletion Runtime World performs.

use super::*;

use crate::branch::{CustodyComponent, ProductBranchRetirementReport};

const RELATIONAL_TARGET: &str = "relational-branch-custody";
const SIGNAL_TARGET: &str = "signal-branch-custody";

fn retire(owner: &TestOwner, child: &ProductBranchObservation) -> ProductBranchRetirementReport {
    RuntimeWorldBranchService::retire_product_branch(owner, child)
        .expect("a live product branch retires")
}

#[test]
fn owner_created_component_branches_carry_custody_and_retirement_emits_typed_work() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let forked = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-custody-forked",
            relational_fork(RELATIONAL_TARGET),
            signal_fork(SIGNAL_TARGET),
        ),
    );

    let records = owner.state.custody.installed_records();
    assert_eq!(records.len(), 2, "each owner fork is one custody record");
    for record in &records {
        assert_eq!(record.product_branch(), forked.branch_identity());
        assert_eq!(record.incarnation(), forked.lifecycle_incarnation());
    }
    assert_eq!(records[0].component(), CustodyComponent::Relational);
    assert_eq!(records[0].target().name(), RELATIONAL_TARGET);
    assert_eq!(records[1].component(), CustodyComponent::Signal);
    assert_eq!(records[1].target().name(), SIGNAL_TARGET);

    let report = retire(&owner, &forked);
    assert_eq!(
        report.released_product_reference(),
        forked.branch_identity()
    );
    assert_eq!(report.owner_retirement_work().len(), 2);
    assert_eq!(
        report.retirement_boundary(),
        Some(source.selected_commit()),
        "World issues the exact source boundary independently of component custody"
    );
    assert_eq!(
        report.owner_retirement_work()[0].target_name(),
        RELATIONAL_TARGET
    );
    assert_eq!(
        report.owner_retirement_work()[1].target_name(),
        SIGNAL_TARGET
    );
    assert_eq!(
        owner.state.custody.installed(),
        0,
        "reported work discharges the custody charge it was derived from"
    );
    drop(forked);
}

/// The mirrored cell of the same matrix: an exact reuse creates no component
/// branch, so it never enters custody and owes its owners nothing at
/// retirement.
#[test]
fn exactly_reused_component_branches_never_enter_custody_and_retirement_emits_no_work() {
    let (_fixture, owner, root) = super::super::setup(3);
    let reused = super::super::create_reused_branch(
        &owner,
        &root,
        super::super::reuse_intent("branch-custody-reused"),
    );
    assert_eq!(owner.state.custody.installed(), 0);
    let report = retire(&owner, &reused);
    assert_eq!(
        report.released_product_reference(),
        reused.branch_identity()
    );
    assert!(
        report.owner_retirement_work().is_empty(),
        "an exact reuse never puts a component branch in Runtime World custody"
    );
    assert_eq!(
        report.retirement_boundary(),
        Some(root.selected_commit()),
        "reuse still carries the exact history boundary needed by retirement"
    );
}

#[test]
fn custody_capacity_denies_creation_before_any_owner_effect() {
    let (fixture, owner, source) = setup_with_custody_budget(3, 1);
    let lifecycles_before = super::super::owner_lifecycles(&owner);
    let costs_before = owner.state.retention.cost_snapshot();
    let cancellation = RuntimeWorldCancellationSource::new();
    let denial = RuntimeWorldBranchService::create_product_branch(
        &owner,
        RuntimeWorldBranchCreationRequest::new(
            source.clone(),
            fork_intent(
                "branch-custody-starved",
                relational_fork("relational-branch-custody-starved"),
                signal_fork("signal-branch-custody-starved"),
            ),
            &cancellation.token(),
        ),
    )
    .expect_err("a creation that cannot charge both custody slots must deny");

    assert_eq!(
        denial,
        RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted
    );
    assert_eq!(
        super::super::owner_lifecycles(&owner),
        lifecycles_before,
        "the custody charge is taken before the first owner fork"
    );
    assert_eq!(owner.state.retention.cost_snapshot(), costs_before);
    assert_eq!(owner.state.custody.installed(), 0);
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert!(
        fixture
            .reserve_relational_fork_target("relational-branch-custody-starved")
            .is_ok(),
        "the denied destination was never created"
    );

    // The denied attempt released the slot it did charge, so a creation that
    // fits the ceiling still runs.
    let child = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-custody-fits",
            relational_fork("relational-branch-custody-fits"),
            SignalBranchCreationPlan::ReuseExact,
        ),
    );
    assert_eq!(owner.state.custody.installed(), 1);
    assert_eq!(
        owner.state.custody.installed_records()[0].component(),
        CustodyComponent::Relational
    );
    drop(child);
}

/// INT-003. A close that leaves an owner-created component branch installed in
/// custody would drop a real branch on the floor: the world stops, and nobody
/// is named as owing its retirement. Close therefore drains custody and hands
/// the same typed work back in its terminal report, and it counts the product
/// references it released while doing so.
#[test]
fn close_reports_every_undrained_custody_record_as_outstanding_retirement_work() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let forked = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-custody-unretired",
            relational_fork("relational-branch-unretired"),
            signal_fork("signal-branch-unretired"),
        ),
    );
    assert_eq!(
        owner.state.custody.installed(),
        2,
        "the branch is never retired, so both fork records are still installed"
    );
    let branches_before = owner.state.branches.branch_count();
    drop(forked);

    let report = owner
        .close()
        .expect("undrained custody is a report row, never a close denial");

    assert_eq!(report.outstanding_owner_retirement_work().len(), 2);
    assert_eq!(
        report.outstanding_owner_retirement_work()[0].target_name(),
        "relational-branch-unretired"
    );
    assert_eq!(
        report.outstanding_owner_retirement_work()[1].target_name(),
        "signal-branch-unretired"
    );
    assert_eq!(
        report.retired_owner_created_custody(),
        2,
        "the custody charge close released is the work it reported"
    );
    assert_eq!(
        owner.state.custody.installed(),
        0,
        "close drains custody once; a record left installed would be unnamed"
    );
    assert_eq!(
        report.released_product_head_pins(),
        branches_before - 1,
        "close released every non-root product reference the registry held"
    );
    assert_eq!(
        owner.state.branches.branch_count(),
        1,
        "the root reference is the world's own, not a created branch"
    );
}
