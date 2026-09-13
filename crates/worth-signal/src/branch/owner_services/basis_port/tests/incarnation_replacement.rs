use crate::branch::{
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchBasisObservationDenial,
    SignalBranchBasisReadmissionDenial,
};

use super::world::{
    assert_retention_cleanup_with_identity_advance, basis_port_world, issue_reference,
};

#[test]
fn same_id_incarnation_replacement_denies_stale_authority_and_admits_fresh_authority() {
    let world = basis_port_world();
    let stale_reference = issue_reference(&world.port, &world.basis_b);
    let old_basis = world.basis_b.clone();
    let expected = world.basis_b.observation().clone();
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let admission = owner.admit().expect("canonical replacement setup admits");
    owner
        .replace_branch_incarnation_for_test(&admission, world.branch_b.id)
        .expect("the owner replaces the same id through its canonical registry");
    drop(admission);

    let retention_before = owner.retention_ledger_observation();
    let stale_cost_before = owner.cost_snapshot();
    assert!(matches!(
        world.port.observe_current(&stale_reference),
        Err(SignalBranchBasisObservationDenial::ManagedReferenceDenied {
            denial: ManagedSignalBranchReferenceAdmissionDenial::BranchIncarnationReplaced,
        })
    ));
    assert!(matches!(
        world
            .port
            .readmit_exact(&stale_reference, world.basis_b.descriptor()),
        Err(SignalBranchBasisReadmissionDenial::ManagedReferenceDenied {
            denial: ManagedSignalBranchReferenceAdmissionDenial::BranchIncarnationReplaced,
        })
    ));
    assert!(matches!(
        world.port.compare_current_exact(&old_basis),
        Err(SignalBranchBasisReadmissionDenial::LifecycleMismatch)
    ));
    let stale_cost_after = owner.cost_snapshot();
    assert_eq!(
        stale_cost_after.owner_upgrade_attempts(),
        stale_cost_before.owner_upgrade_attempts() + 3
    );
    assert_eq!(
        stale_cost_after.branch_registry_lookups(),
        stale_cost_before.branch_registry_lookups() + 3
    );
    assert_eq!(
        stale_cost_after.target_cell_contacts(),
        stale_cost_before.target_cell_contacts() + 1,
        "stale managed authority is rejected before reaching the replacement cell"
    );
    assert_eq!(
        stale_cost_after.retention_registry_contacts(),
        stale_cost_before.retention_registry_contacts(),
        "stale authority cannot reserve admitted retention against the replacement"
    );
    assert_eq!(owner.retention_ledger_observation(), retention_before);

    let fresh_reference = issue_reference(&world.port, &world.basis_b);
    assert_eq!(fresh_reference.branch_id(), stale_reference.branch_id());
    let observed = world
        .port
        .observe_current(&fresh_reference)
        .expect("fresh owner-issued authority reaches the replacement cell");
    let readmitted = world
        .port
        .readmit_exact(&fresh_reference, world.basis_b.descriptor())
        .expect("fresh authority readmits the exact preserved replacement state");
    assert_eq!(observed.observation(), &expected);
    assert_eq!(readmitted.observation(), &expected);

    let fresh_cost_after = owner.cost_snapshot();
    assert_eq!(
        fresh_cost_after.owner_upgrade_attempts(),
        stale_cost_after.owner_upgrade_attempts() + 3
    );
    assert_eq!(
        fresh_cost_after.branch_registry_lookups(),
        stale_cost_after.branch_registry_lookups() + 3
    );
    assert_eq!(
        fresh_cost_after.target_cell_contacts(),
        stale_cost_after.target_cell_contacts() + 2
    );
    assert_eq!(
        fresh_cost_after.retention_registry_contacts(),
        stale_cost_after.retention_registry_contacts() + 1,
        "the second fresh call reuses the canonical owner lease"
    );
    drop(observed);
    drop(readmitted);
    assert_retention_cleanup_with_identity_advance(
        &retention_before,
        &owner.retention_ledger_observation(),
        1,
    );
}
