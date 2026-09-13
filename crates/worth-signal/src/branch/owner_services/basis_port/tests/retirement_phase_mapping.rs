use worth_proof::TransitionOutcome;

use crate::branch::{
    SignalBranchBasisObservationDenial, SignalBranchBasisReadmissionDenial,
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetirementReason,
};

use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerCancellationSource;
use super::super::denial_mapping::{
    map_observation_retention_denial, map_readmission_retention_denial,
};
use super::world::{basis_port_world, issue_reference};

#[test]
fn active_retirement_reservation_maps_basis_denials_to_retirement_in_progress() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the reserved owner remains live");
    let admission = owner.admit().expect("retirement reservation admits");
    let reservation = owner
        .metadata
        .reserve_retirement(&admission, world.branch_b.id)
        .expect("canonical metadata records an active reservation");

    let observation_denial = owner
        .acquire_admitted_retention(&admission, world.branch_b.id)
        .expect_err("active retirement denies new admitted retention");
    assert!(matches!(
        map_observation_retention_denial(
            &owner,
            &admission,
            observation_denial,
            world.branch_b.id,
        ),
        SignalBranchBasisObservationDenial::RetirementInProgress { branch_id }
            if branch_id == world.branch_b.id
    ));
    let readmission_denial = owner
        .acquire_admitted_retention(&admission, world.branch_b.id)
        .expect_err("active retirement consistently denies admitted retention");
    assert!(matches!(
        map_readmission_retention_denial(
            &owner,
            &admission,
            readmission_denial,
            world.branch_b.id,
        ),
        SignalBranchBasisReadmissionDenial::RetirementInProgress { branch_id }
            if branch_id == world.branch_b.id
    ));

    drop(reservation);
    drop(admission);
    assert!(
        world.port.observe_current(&reference).is_ok(),
        "dropping the active reservation immediately restores healthy observation"
    );
}

#[test]
fn completed_retirement_receipt_maps_basis_denials_to_retired_branch() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let plan = match runtime.plan_signal_branch_retirement(
        target.clone(),
        basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the real owner issues terminal retirement authority: {other:?}"),
    };
    let (basis_port, _, lifecycle_port) = runtime
        .owner_port_slots()
        .expect("the terminal fixture seals into owner services");
    let owner = basis_port
        .upgrade_owner()
        .expect("the terminal owner remains live");
    let receipt =
        match lifecycle_port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()) {
            TransitionOutcome::Success(receipt) => receipt,
            other => panic!("the canonical retirement completes: {other:?}"),
        };
    assert_eq!(receipt.retired_branch(), &target);
    let admission = owner.admit().expect("terminal posture inspection admits");
    assert_eq!(
        owner
            .metadata
            .retirement_receipt(&admission, target.id)
            .expect("canonical receipt lookup is owner-admitted"),
        Some(receipt)
    );

    let observation_denial = owner
        .acquire_admitted_retention(&admission, target.id)
        .expect_err("terminal retirement denies new admitted retention");
    assert!(matches!(
        observation_denial,
        SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id }
            if branch_id == target.id
    ));
    let observation_denial = owner
        .acquire_admitted_retention(&admission, target.id)
        .expect_err("terminal retirement remains stable for observation mapping");
    assert!(matches!(
        map_observation_retention_denial(&owner, &admission, observation_denial, target.id),
        SignalBranchBasisObservationDenial::RetiredBranch { branch_id }
            if branch_id == target.id
    ));
    let readmission_denial = owner
        .acquire_admitted_retention(&admission, target.id)
        .expect_err("terminal retirement remains stable for readmission mapping");
    assert!(matches!(
        map_readmission_retention_denial(&owner, &admission, readmission_denial, target.id),
        SignalBranchBasisReadmissionDenial::RetiredBranch { branch_id }
            if branch_id == target.id
    ));
}
