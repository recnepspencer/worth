use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::super::super::lifecycle_state::MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS;
use super::super::super::tests::runtime_root::runtime_with_two_branches;

#[test]
fn planning_distinguishes_current_canonical_live_child_and_merge_participant() {
    let (mut current_runtime, current, _, target_basis) = runtime_with_two_branches();
    let current_basis = current_runtime
        .observe_signal_branch_basis(current.clone())
        .expect("the current branch admits");
    drop(target_basis);
    let (_, _, current_port) = current_runtime
        .owner_port_slots()
        .expect("the current fixture seals");
    assert!(matches!(
        current_port.plan_retirement_exact(current_basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CurrentBranch { branch_id })
            if branch_id == current.id
    ));

    let (mut canonical_runtime, canonical, selected, selected_basis) = runtime_with_two_branches();
    let canonical_basis = canonical_runtime
        .observe_signal_branch_basis(canonical.clone())
        .expect("the canonical branch admits");
    canonical_runtime
        .switch_branch(selected)
        .expect("the child becomes selected");
    drop(selected_basis);
    let (_, _, canonical_port) = canonical_runtime
        .owner_port_slots()
        .expect("the canonical fixture seals");
    assert!(matches!(
        canonical_port
            .plan_retirement_exact(canonical_basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBranch { branch_id })
            if branch_id == canonical.id
    ));

    let (mut child_runtime, _, parent, parent_basis) = runtime_with_two_branches();
    let (child, child_basis) = child_runtime
        .fork_signal_branch("lifecycle-port-live-child", &parent_basis)
        .expect("the real parent forks a live child")
        .into_parts();
    drop(child_basis);
    let (_, _, child_port) = child_runtime
        .owner_port_slots()
        .expect("the child fixture seals");
    assert!(matches!(
        child_port.plan_retirement_exact(parent_basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::LiveChildren {
            branch_id,
            child_branch_ids,
        }) if branch_id == parent.id && child_branch_ids == vec![child.id]
    ));

    let (mut merge_runtime, selected, target, merge_basis) = runtime_with_two_branches();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        merge_runtime.inject_merge_participation_unwind_for_owner_contract(selected.id, target.id);
    }));
    assert!(unwind.is_err(), "the controlled merge boundary unwinds");
    let (_, _, merge_port) = merge_runtime
        .owner_port_slots()
        .expect("interrupted merge metadata seals");
    assert!(matches!(
        merge_port.plan_retirement_exact(merge_basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::MergeParticipant { branch_id })
            if branch_id == target.id
    ));
}

#[test]
fn planning_rejects_stale_and_foreign_complete_basis_without_weakening_affinity() {
    let (mut stale_runtime, _, _, stale_basis) = runtime_with_two_branches();
    let (snapshot, refreshed_basis) = stale_runtime
        .capture_signal_branch_snapshot(&stale_basis)
        .expect("a real capture advances the observation")
        .into_parts();
    drop(snapshot);
    drop(refreshed_basis);
    let (_, _, stale_port) = stale_runtime
        .owner_port_slots()
        .expect("the stale fixture seals");
    assert!(matches!(
        stale_port.plan_retirement_exact(stale_basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));

    let (mut runtime_a, _, _, basis_a) = runtime_with_two_branches();
    let (_, _, port_a) = runtime_a.owner_port_slots().expect("owner A seals");
    let owner_a = port_a.upgrade_owner().expect("owner A remains live");
    let (mut runtime_b, _, _, basis_b) = runtime_with_two_branches();
    let (_, _, _port_b) = runtime_b.owner_port_slots().expect("owner B seals");
    drop(basis_a);
    let before = owner_a.cost_snapshot();
    assert!(matches!(
        port_a.plan_retirement_exact(basis_b, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));
    assert_eq!(
        owner_a.cost_snapshot().branch_registry_lookups(),
        before.branch_registry_lookups(),
        "foreign authority denies before registry contact"
    );
    assert_eq!(
        owner_a.cost_snapshot().retention_registry_contacts(),
        before.retention_registry_contacts(),
        "foreign authority denies before retention contact"
    );
    assert_eq!(
        owner_a.cost_snapshot().target_cell_contacts(),
        before.target_cell_contacts(),
        "foreign authority denies before target contact"
    );
}

#[test]
fn planning_preserves_component_admitted_and_shared_holder_denials() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let (_, _, port) = runtime.owner_port_slots().expect("external fixture seals");
    let owner = port.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("external retention admits");
    let external = owner
        .acquire_external_retention(&admission, &basis)
        .expect("the exact component basis retains");
    assert!(matches!(
        port.plan_retirement_exact(basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedComponentBasis {
            branch_id,
            active_leases: 1,
        }) if branch_id == target.id
    ));
    drop(external);
    drop(admission);

    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let (_, _, port) = runtime.owner_port_slots().expect("admitted fixture seals");
    let owner = port.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("admitted retention admits");
    let extra = owner
        .acquire_admitted_retention(&admission, target.id)
        .expect("one additional admitted lease issues");
    assert!(matches!(
        port.plan_retirement_exact(basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedAdmittedBasis {
            branch_id,
            active_leases: 2,
        }) if branch_id == target.id
    ));
    drop(extra);
    drop(admission);

    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let shared = basis.clone();
    let (_, _, port) = runtime.owner_port_slots().expect("holder fixture seals");
    assert!(matches!(
        port.plan_retirement_exact(basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
            branch_id,
            shared_holders: 2,
        }) if branch_id == target.id
    ));
    drop(shared);
}

#[test]
fn planning_maps_internal_admission_reentry_and_capacity_without_target_contact() {
    let (mut reentry_runtime, _, target, reentry_basis) = runtime_with_two_branches();
    let (_, _, reentry_port) = reentry_runtime
        .owner_port_slots()
        .expect("the reentry fixture seals");
    let reentry_owner = reentry_port
        .upgrade_owner()
        .expect("the owner remains live");
    let admission = reentry_owner.admit().expect("setup admits");
    let cell = reentry_owner
        .lookup_cell(&admission, target.id)
        .expect("the target cell is canonical");
    let target_before = cell.cost_snapshot();
    let metadata_hold = admission
        .hold_owner_metadata()
        .expect("the executing thread holds metadata");
    assert!(matches!(
        reentry_port.plan_retirement_exact(reentry_basis, SignalBranchRetirementReason::Rejected),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::OwnerReentry)
    ));
    assert_eq!(cell.cost_snapshot(), target_before);
    drop(metadata_hold);
    drop(admission);

    let (mut capacity_runtime, _, target, capacity_basis) = runtime_with_two_branches();
    let (_, _, capacity_port) = capacity_runtime
        .owner_port_slots()
        .expect("the capacity fixture seals");
    let capacity_owner = capacity_port
        .upgrade_owner()
        .expect("the capacity owner remains live");
    let admissions = (0..MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS)
        .map(|_| {
            capacity_owner
                .admit()
                .expect("capacity admits to its bound")
        })
        .collect::<Vec<_>>();
    let setup = admissions.first().expect("at least one admission exists");
    let cell = capacity_owner
        .lookup_cell(setup, target.id)
        .expect("the capacity target is canonical");
    let target_before = cell.cost_snapshot();
    assert!(matches!(
        capacity_port.plan_retirement_exact(
            capacity_basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        }) if maximum_in_flight_operations == MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS
    ));
    assert_eq!(cell.cost_snapshot(), target_before);
    drop(admissions);
}
