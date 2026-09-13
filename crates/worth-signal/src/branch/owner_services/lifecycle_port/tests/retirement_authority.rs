use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerCancellationSource;

#[test]
fn foreign_equal_local_id_retirement_plan_denies_before_receiving_owner_contact() {
    let (mut issuer_runtime, _, issuer_target, issuer_basis) = runtime_with_two_branches();
    let (_, _, issuer_port) = issuer_runtime
        .owner_port_slots()
        .expect("the issuing runtime seals");
    let foreign_plan = match issuer_port
        .plan_retirement_exact(issuer_basis, SignalBranchRetirementReason::Rejected)
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the issuing owner produces a real retirement plan: {other:?}"),
    };

    let (mut receiver_runtime, _, receiver_target, receiver_basis) = runtime_with_two_branches();
    let (_, _, receiver_port) = receiver_runtime
        .owner_port_slots()
        .expect("the receiving runtime seals");
    let receiver_owner = receiver_port
        .upgrade_owner()
        .expect("the receiving owner remains live");
    assert_eq!(
        issuer_target.id, receiver_target.id,
        "the adversarial worlds intentionally reuse an equal-looking local branch id"
    );
    assert_ne!(
        issuer_port.diagnostic_owner_runtime_instance_id(),
        receiver_port.diagnostic_owner_runtime_instance_id(),
        "the equal local ids remain owned by distinct runtimes"
    );
    assert_eq!(
        receiver_owner.admitted_retention_count(receiver_target.id),
        1,
        "the receiver has the sole local admitted basis that permits honest retirement"
    );

    let inspection = receiver_owner.admit().expect("receiver inspection admits");
    let receiver_cell = receiver_owner
        .lookup_cell(&inspection, receiver_target.id)
        .expect("the equal local target is healthy before foreign execution");
    let contract_before = receiver_owner
        .metadata
        .retirement_contract_observation(&inspection, receiver_target.id)
        .expect("the receiving retirement contract is observable");
    drop(inspection);
    let cell_before = receiver_cell.cost_snapshot();
    let retention_before = receiver_owner.retention_ledger_observation();
    let cost_before = receiver_owner.cost_snapshot();
    let live_before = receiver_owner.live_count();

    assert!(matches!(
        receiver_port.retire_exact(foreign_plan, &SignalOwnerCancellationSource::new().token(),),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));

    let cost_after = receiver_owner.cost_snapshot();
    assert_eq!(receiver_cell.cost_snapshot(), cell_before);
    assert_eq!(
        receiver_owner.retention_ledger_observation(),
        retention_before
    );
    assert_eq!(receiver_owner.live_count(), live_before);
    assert_eq!(
        cost_after.branch_registry_lookups(),
        cost_before.branch_registry_lookups()
    );
    assert_eq!(
        cost_after.branch_registry_reservations(),
        cost_before.branch_registry_reservations()
    );
    assert_eq!(
        cost_after.target_cell_contacts(),
        cost_before.target_cell_contacts()
    );
    assert_eq!(
        cost_after.retention_registry_contacts(),
        cost_before.retention_registry_contacts()
    );
    assert_eq!(
        cost_after.canonical_movements(),
        cost_before.canonical_movements()
    );
    let follow_up = receiver_owner
        .admit()
        .expect("foreign denial releases receiving admission");
    assert_eq!(
        receiver_owner
            .metadata
            .retirement_contract_observation(&follow_up, receiver_target.id)
            .expect("foreign denial leaves no metadata reservation"),
        contract_before
    );
    drop(follow_up);

    assert!(matches!(
        receiver_port.plan_retirement_exact(
            receiver_basis,
            SignalBranchRetirementReason::ProjectionRebuild,
        ),
        TransitionOutcome::Success(_)
    ));
}

#[test]
fn same_owner_stale_retirement_plan_reaches_optimistic_exact_comparison() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let (_, _, port) = runtime
        .owner_port_slots()
        .expect("the same-owner fixture seals");
    let owner = port.upgrade_owner().expect("the same owner remains live");
    let plan = match port.plan_retirement_exact(basis, SignalBranchRetirementReason::Superseded) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the current same-owner plan issues: {other:?}"),
    };
    let movement_basis = plan.admitted_basis().clone();
    let movement_admission = owner.admit().expect("the intervening movement admits");
    let cell = owner
        .lookup_cell(&movement_admission, target.id)
        .expect("the same-owner target remains installed");
    cell.advance_exact::<(), (), _>(
        &movement_admission,
        &movement_basis,
        &mut (),
        &SignalOwnerCancellationSource::new().token(),
        |_| Ok(()),
    )
    .expect("the intervening canonical movement succeeds");
    drop(movement_admission);
    drop(movement_basis);
    let cell_before = cell.cost_snapshot();
    let cost_before = owner.cost_snapshot();

    assert!(matches!(
        port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));

    let cost_after = owner.cost_snapshot();
    assert_eq!(
        cost_after.target_cell_contacts(),
        cost_before.target_cell_contacts() + 1,
        "same-owner staleness reaches the target's exact execution comparison"
    );
    assert_eq!(
        cost_after.retention_registry_contacts(),
        cost_before.retention_registry_contacts() + 1,
        "same-owner staleness reaches the ordinary retirement reservation"
    );
    assert_eq!(
        cost_after.canonical_movements(),
        cost_before.canonical_movements(),
        "the stale retirement itself performs no movement"
    );
    let cell_after = cell.cost_snapshot();
    assert_eq!(cell_after.contacts(), cell_before.contacts() + 1);
    assert_eq!(cell_after.movements(), cell_before.movements());
    let follow_up = owner.admit().expect("stale denial releases reservation");
    assert_eq!(
        cell.observe_exact(&follow_up)
            .expect("the moved same-owner target remains immediately healthy")
            .generation()
            .get(),
        1
    );
}
