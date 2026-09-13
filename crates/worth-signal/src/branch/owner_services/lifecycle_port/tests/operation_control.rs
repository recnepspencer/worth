use std::panic::{catch_unwind, AssertUnwindSafe};

use worth_proof::TransitionOutcome;

use crate::branch::SignalBranchRetirementReason;

use super::super::super::operation_control::SignalOwnerOperationBoundary;
use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerCancellationSource;

#[test]
fn lifecycle_planning_reaches_admission_and_exact_basis_boundaries() {
    let (mut admission_runtime, _, target, admission_basis) = runtime_with_two_branches();
    let (_, _, admission_port) = admission_runtime
        .owner_port_slots()
        .expect("the admission fixture seals");
    let admission_owner = admission_port
        .upgrade_owner()
        .expect("the admission owner remains live");
    let setup = admission_owner.admit().expect("setup admits");
    let admission_cell = admission_owner
        .lookup_cell(&setup, target.id)
        .expect("the admission target is canonical");
    drop(setup);
    let admission_before = admission_cell.cost_snapshot();
    admission_owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::OwnerLifecycleAdmission);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = admission_port
            .plan_retirement_exact(admission_basis, SignalBranchRetirementReason::Rejected);
    }))
    .is_err());
    assert_eq!(admission_cell.cost_snapshot(), admission_before);
    assert!(admission_owner.admit().is_ok(), "the one-shot fault clears");

    let (mut preflight_runtime, _, target, preflight_basis) = runtime_with_two_branches();
    let (_, _, preflight_port) = preflight_runtime
        .owner_port_slots()
        .expect("the preflight fixture seals");
    let preflight_owner = preflight_port
        .upgrade_owner()
        .expect("the preflight owner remains live");
    let setup = preflight_owner.admit().expect("setup admits");
    let preflight_cell = preflight_owner
        .lookup_cell(&setup, target.id)
        .expect("the preflight target is canonical");
    drop(setup);
    let preflight_before = preflight_cell.cost_snapshot();
    preflight_owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::ExactBasisPreflight);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = preflight_port
            .plan_retirement_exact(preflight_basis, SignalBranchRetirementReason::Rejected);
    }))
    .is_err());
    let preflight_after = preflight_cell.cost_snapshot();
    assert_eq!(preflight_after.contacts(), preflight_before.contacts() + 1);
    assert_eq!(preflight_after.movements(), preflight_before.movements());
    assert!(preflight_owner.admit().is_ok(), "the planning hold unwinds");
}

#[test]
fn lifecycle_planning_reaches_registry_and_target_cell_boundaries() {
    for (boundary, expected_registry_lookups) in [
        (SignalOwnerOperationBoundary::BranchRegistryLookup, 0),
        (SignalOwnerOperationBoundary::TargetCellAdmission, 1),
    ] {
        let (mut runtime, _, target, basis) = runtime_with_two_branches();
        let (_, _, port) = runtime.owner_port_slots().expect("the fixture seals");
        let owner = port.upgrade_owner().expect("the owner remains live");
        let setup = owner.admit().expect("setup admits");
        let cell = owner
            .lookup_cell(&setup, target.id)
            .expect("the target cell is canonical");
        drop(setup);
        let owner_before = owner.cost_snapshot();
        let cell_before = cell.cost_snapshot();
        owner.operation_control().inject_panic_once(boundary);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _ = port.plan_retirement_exact(basis, SignalBranchRetirementReason::Rejected);
        }))
        .is_err());
        assert_eq!(cell.cost_snapshot(), cell_before);
        assert_eq!(
            owner.cost_snapshot().branch_registry_lookups(),
            owner_before.branch_registry_lookups() + expected_registry_lookups,
            "{boundary:?} exposes its exact side of registry contact"
        );
        assert!(owner.admit().is_ok(), "the one-shot boundary fault clears");
    }
}

#[test]
fn lifecycle_retirement_reaches_both_sides_of_canonical_movement() {
    let (mut before_runtime, _, before_target, before_basis) = runtime_with_two_branches();
    let before_plan = match before_runtime.plan_signal_branch_retirement(
        before_target.clone(),
        before_basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the pre-movement fixture plans: {other:?}"),
    };
    let (_, _, before_port) = before_runtime
        .owner_port_slots()
        .expect("the pre-movement fixture seals");
    let before_owner = before_port
        .upgrade_owner()
        .expect("the pre-movement owner remains live");
    let setup = before_owner.admit().expect("setup admits");
    let before_cell = before_owner
        .lookup_cell(&setup, before_target.id)
        .expect("the pre-movement target is canonical");
    drop(setup);
    let cost_before = before_cell.cost_snapshot();
    before_owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ =
            before_port.retire_exact(before_plan, &SignalOwnerCancellationSource::new().token());
    }))
    .is_err());
    let cost_after = before_cell.cost_snapshot();
    assert_eq!(cost_after.contacts(), cost_before.contacts() + 1);
    assert_eq!(cost_after.movements(), cost_before.movements());
    let inspection = before_owner.admit().expect("unwind releases admission");
    assert!(before_owner
        .lookup_cell(&inspection, before_target.id)
        .is_ok());

    let (mut after_runtime, _, after_target, after_basis) = runtime_with_two_branches();
    let after_plan = match after_runtime.plan_signal_branch_retirement(
        after_target.clone(),
        after_basis,
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the post-movement fixture plans: {other:?}"),
    };
    let (_, _, after_port) = after_runtime
        .owner_port_slots()
        .expect("the post-movement fixture seals");
    let after_owner = after_port
        .upgrade_owner()
        .expect("the post-movement owner remains live");
    let setup = after_owner.admit().expect("setup admits");
    let after_cell = after_owner
        .lookup_cell(&setup, after_target.id)
        .expect("the post-movement target is canonical");
    drop(setup);
    let cost_before = after_cell.cost_snapshot();
    after_owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::AfterCanonicalMovement);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = after_port.retire_exact(after_plan, &SignalOwnerCancellationSource::new().token());
    }))
    .is_err());
    let cost_after = after_cell.cost_snapshot();
    assert_eq!(cost_after.contacts(), cost_before.contacts() + 1);
    assert_eq!(cost_after.movements(), cost_before.movements() + 1);
    let inspection = after_owner
        .admit()
        .expect("receipt recovery releases admission");
    assert_eq!(
        after_owner
            .metadata
            .retirement_receipt(&inspection, after_target.id)
            .expect("the performed receipt remains owner-admitted")
            .expect("post-movement unwind recovers the receipt")
            .retired_branch(),
        &after_target
    );
}
