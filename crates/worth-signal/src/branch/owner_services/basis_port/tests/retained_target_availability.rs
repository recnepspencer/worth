use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::branch::{SignalBranchBasisReadmissionDenial, SignalBranchRetainedReadmissionDenial};

use super::world::{advance_exact, basis_port_world, issue_reference};

#[test]
fn retained_exact_readmission_denies_quarantined_target_without_minting_basis() {
    let world = basis_port_world();
    let descriptor = world.basis_b.descriptor().clone();
    let reference = issue_reference(&world.port, &world.basis_b);
    let lease = world
        .port
        .retain_exact(&world.basis_b)
        .expect("the live exact target opens one external obligation");
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let mutation_admission = owner.admit().expect("the separate mutation admits");
    let cell = owner
        .lookup_cell(&mutation_admission, world.branch_b.id)
        .expect("the retained target is installed before the mutation");
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = cell.advance_exact::<(), (), _>(
            &mutation_admission,
            &world.basis_b,
            &mut (),
            &super::super::super::SignalOwnerCancellationSource::new().token(),
            |_| panic!("inject retained-target mutation panic"),
        );
    }));
    assert!(panic.is_err(), "the admitted mutation reaches its fault");
    drop(mutation_admission);

    let retention_before = owner.retention_ledger_observation();
    let readmission = world.port.readmit_retained_exact(&descriptor, &lease);
    let retention_after = owner.retention_ledger_observation();

    assert_eq!(
        retention_after, retention_before,
        "unavailable exact admission cannot mint an admitted pin or consume capacity"
    );
    assert!(matches!(
        readmission,
        Err(SignalBranchRetainedReadmissionDenial::UnavailableExactTarget(_))
    ));
    assert!(matches!(
        world.port.readmit_exact(&reference, &descriptor),
        Err(SignalBranchBasisReadmissionDenial::QuarantinedBranch { branch_id })
            if branch_id == world.branch_b.id
    ));
}

#[test]
fn retained_exact_readmission_accepts_live_historical_target_after_movement() {
    let world = basis_port_world();
    let historical = world.basis_b.descriptor().clone();
    let lease = world
        .port
        .retain_exact(&world.basis_b)
        .expect("the exact historical target opens one external obligation");
    let moved = advance_exact(&world.port, &world.basis_b);

    let readmitted = world
        .port
        .readmit_retained_exact(&historical, &lease)
        .expect("movement does not revoke exact historical availability");

    assert_ne!(moved, *historical.observation());
    assert_eq!(readmitted.observation(), historical.observation());
}
