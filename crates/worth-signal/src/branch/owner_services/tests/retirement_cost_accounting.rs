use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::runtime_root::runtime_with_two_branches;

#[test]
fn retirement_planning_and_reservation_count_each_real_retention_read_once() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let (basis_port, _, _) = runtime.owner_port_slots().expect("retirement owner seals");
    let owner = basis_port
        .upgrade_owner()
        .expect("retirement owner remains live");
    let admission = owner.admit().expect("retirement accounting admits");
    let before_plan = owner.cost_snapshot().retention_registry_contacts();
    let plan = match owner.plan_retirement_exact(
        &admission,
        basis,
        SignalBranchRetirementReason::DependencyCancellation,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the exact retirement plan succeeds: {other:?}"),
    };
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        before_plan + 1,
        "planning performs exactly one canonical retention read"
    );
    let reservation = owner
        .reserve_retirement(&admission, target.id)
        .expect("the planned target reserves at the effect boundary");
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        before_plan + 2,
        "reservation performs its independent canonical retention recheck once"
    );
    drop(reservation);
    drop(plan);

    let (mut denied_runtime, _, denied_target, denied_basis) = runtime_with_two_branches();
    let (denied_port, _, _) = denied_runtime
        .owner_port_slots()
        .expect("denial owner seals");
    let denied_owner = denied_port
        .upgrade_owner()
        .expect("denial owner remains live");
    let denied_admission = denied_owner.admit().expect("denial accounting admits");
    let external = denied_owner
        .acquire_external_retention(&denied_admission, &denied_basis)
        .expect("real external custody opens");
    let before_denial = denied_owner.cost_snapshot().retention_registry_contacts();
    assert!(matches!(
        denied_owner.plan_retirement_exact(
            &denied_admission,
            denied_basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedComponentBasis {
            branch_id,
            active_leases: 1,
        }) if branch_id == denied_target.id
    ));
    assert_eq!(
        denied_owner.cost_snapshot().retention_registry_contacts(),
        before_denial + 1,
        "a retention denial does not duplicate the planning read"
    );
    drop(external);
}
