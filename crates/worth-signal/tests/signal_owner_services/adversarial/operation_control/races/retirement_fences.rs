use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::super::super::world::AdversarialWorld;
use super::{
    advance, capture, prove_advance_is_effectful, prove_capture_and_restore_are_effectful,
    prove_retirement_is_effectful, retire, retirement_plan,
};

#[test]
fn same_branch_advance_retire_fences_retention_then_retires_uncontended() {
    prove_advance_is_effectful();
    prove_retirement_is_effectful();
    let AdversarialWorld {
        runtime,
        basis,
        mutation,
        lifecycle,
        root_basis,
        child_basis,
    } = AdversarialWorld::new();
    let branch_id = child_basis.branch_id();
    let reference = basis
        .issue_managed_branch_reference(&child_basis)
        .expect("the advancing contender starts from a managed reference");
    let contender_basis = basis
        .observe_current(&reference)
        .expect("the advancing contender receives current managed-reference custody");
    assert_eq!(contender_basis.branch_id(), branch_id);
    assert!(matches!(
        lifecycle.plan_retirement_exact(child_basis, SignalBranchRetirementReason::Superseded),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedAdmittedBasis {
            branch_id: denied_branch,
            active_leases: 2,
        }) if denied_branch == branch_id
    ));
    drop(contender_basis);

    let retry_basis = basis
        .observe_current(&reference)
        .expect("the sole retirement basis can be reacquired after the contender releases");
    let shared_holder = retry_basis.clone();
    assert!(matches!(
        lifecycle.plan_retirement_exact(retry_basis, SignalBranchRetirementReason::Superseded),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
            branch_id: denied_branch,
            shared_holders: 2,
        }) if denied_branch == branch_id
    ));
    drop(shared_holder);

    let plan = retirement_plan(
        &lifecycle,
        basis
            .observe_current(&reference)
            .expect("the final retirement basis is owner-issued and current"),
    )
    .expect("retirement is admissible after the advancing contender and shared holder release");
    retire(&lifecycle, plan).expect("uncontended retirement succeeds");
    advance(&mutation, &root_basis)
        .expect("the unaffected root remains healthy after child retirement");
    runtime
        .as_ref()
        .expect("the owner root remains live after the retirement fence")
        .owner_operation_control()
        .expect("the owner remains callable after the healthy follow-up");
}

#[test]
fn same_branch_restore_retire_fences_retention_then_retires_uncontended() {
    prove_capture_and_restore_are_effectful();
    prove_retirement_is_effectful();
    let AdversarialWorld {
        runtime,
        basis,
        mutation,
        lifecycle,
        root_basis,
        child_basis,
    } = AdversarialWorld::new();
    let branch_id = child_basis.branch_id();
    let reference = basis
        .issue_managed_branch_reference(&child_basis)
        .expect("the restoring contender starts from a managed reference");
    let (snapshot, current) = capture(&mutation, &child_basis)
        .expect("the restoring contender has a real snapshot and current basis");
    let contender_basis = basis
        .observe_current(&reference)
        .expect("the restoring contender receives current managed-reference custody");
    assert_eq!(contender_basis.branch_id(), branch_id);
    assert!(matches!(
        lifecycle.plan_retirement_exact(current, SignalBranchRetirementReason::Superseded),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedAdmittedBasis {
            branch_id: denied_branch,
            active_leases: 4,
        }) if denied_branch == branch_id
    ));
    drop(contender_basis);
    drop(snapshot);
    drop(child_basis);

    let plan = retirement_plan(
        &lifecycle,
        basis
            .observe_current(&reference)
            .expect("the final retirement basis is owner-issued and current"),
    )
    .expect("retirement is admissible after the restoring contender releases");
    retire(&lifecycle, plan).expect("uncontended retirement succeeds");
    advance(&mutation, &root_basis)
        .expect("the unaffected root remains healthy after child retirement");
    runtime
        .as_ref()
        .expect("the owner root remains live after the retirement fence")
        .owner_operation_control()
        .expect("the owner remains callable after the healthy follow-up");
}
