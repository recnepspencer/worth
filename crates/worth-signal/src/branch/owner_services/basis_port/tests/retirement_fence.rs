use crate::branch::{
    SignalBranchBasisObservationDenial, SignalBranchBasisReadmissionDenial,
    SignalBranchRetentionReleaseOutcome, SignalBranchRetirementDenial,
};

use super::world::{basis_port_world, issue_reference};

#[test]
fn ready_reuse_allows_retirement_with_one_canonical_lease_and_external_retention_blocks_it() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let observed = world
        .port
        .observe_current(&reference)
        .expect("Ready reuse follows checked-cell observation without retention reacquisition");
    let admission = owner.admit().expect("retirement planning admits");
    let retirement = owner
        .reserve_retirement(&admission, world.branch_b.id)
        .expect("one canonical admitted lease is the retirement allowance");
    drop(retirement);
    drop(observed);

    let external = world
        .port
        .retain_exact(&world.basis_b)
        .expect("external retention acquires through the same metadata fence");
    assert!(matches!(
        owner.reserve_retirement(&admission, world.branch_b.id),
        Err(SignalBranchRetirementDenial::RetainedComponentBasis {
            branch_id,
            active_leases: 1,
        }) if branch_id == world.branch_b.id
    ));
    assert!(matches!(
        world.port.release_exact(external),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    assert!(
        world.port.observe_current(&reference).is_ok(),
        "retirement denial leaves the real port immediately healthy"
    );
}

#[test]
fn installed_retirement_fence_denies_observation_and_readmission_after_each_checked_cell_contact() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let admission = owner.admit().expect("retirement reservation admits");
    let retirement = owner
        .metadata
        .reserve_retirement(&admission, world.branch_b.id)
        .expect("the canonical metadata reservation installs first");
    let cell = owner
        .lookup_cell(&admission, world.branch_b.id)
        .expect("the reservation has not yet removed the live cell");
    let cell_before = cell.cost_snapshot();
    let retention_before = owner.retention_ledger_observation();

    assert!(matches!(
        world.port.observe_current(&reference),
        Err(SignalBranchBasisObservationDenial::RetirementInProgress { branch_id })
            if branch_id == world.branch_b.id
    ));
    assert!(matches!(
        world
            .port
            .readmit_exact(&reference, world.basis_b.descriptor()),
        Err(SignalBranchBasisReadmissionDenial::RetirementInProgress { branch_id })
            if branch_id == world.branch_b.id
    ));
    let cell_after = cell.cost_snapshot();
    assert_eq!(
        cell_after.contacts(),
        cell_before.contacts() + 2,
        "the retirement fence denies after one checked-cell contact per call"
    );
    assert_eq!(cell_after.waits(), cell_before.waits());
    assert_eq!(cell_after.movements(), cell_before.movements());
    assert_eq!(
        owner.retention_ledger_observation(),
        retention_before,
        "a denied reservation leaks no admitted obligation"
    );

    drop(retirement);
    drop(admission);
    assert!(
        world
            .port
            .readmit_exact(&reference, world.basis_b.descriptor())
            .is_ok(),
        "releasing the unperformed fence admits a healthy twin"
    );
}
