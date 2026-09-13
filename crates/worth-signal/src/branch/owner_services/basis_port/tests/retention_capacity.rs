use crate::branch::{SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionReleaseOutcome};

use super::world::{basis_port_world, issue_reference};

#[test]
fn retention_capacity_preserves_canonical_reuse_without_lease_amplification() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let mut held = Vec::new();
    let maximum_active_leases = loop {
        match world.port.retain_exact(&world.basis_b) {
            Ok(lease) => held.push(lease),
            Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases,
            }) => break maximum_active_leases,
            Err(other) => panic!("only bounded retention capacity should deny: {other:?}"),
        }
    };
    assert_eq!(
        owner.retention_ledger_observation().used_capacity,
        maximum_active_leases
    );
    let full = owner.retention_ledger_observation();
    let denial_cost_before = owner.cost_snapshot();

    let observed = world
        .port
        .observe_current(&reference)
        .expect("the existing exact basis reuses its owner admission at capacity");
    assert_eq!(
        observed.admission_identity(),
        world.basis_b.admission_identity()
    );
    drop(observed);
    let readmitted = world
        .port
        .readmit_exact(&reference, world.basis_b.descriptor())
        .expect("exact readmission reuses the live canonical basis at capacity");
    assert_eq!(
        readmitted.admission_identity(),
        world.basis_b.admission_identity()
    );
    drop(readmitted);
    let retained = world
        .port
        .readmit_retained_exact(world.basis_b.descriptor(), &held[0])
        .expect("retained readmission reuses the live canonical basis at capacity");
    assert_eq!(
        retained.admission_identity(),
        world.basis_b.admission_identity()
    );
    drop(retained);
    assert!(matches!(
        world.port.retain_exact(&world.basis_b),
        Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
            maximum_active_leases: observed,
        }) if observed == maximum_active_leases
    ));
    let denial_cost_after = owner.cost_snapshot();
    assert_eq!(
        denial_cost_after.retention_registry_contacts(),
        denial_cost_before.retention_registry_contacts() + 1,
        "only the new external obligation contacts the retention registry"
    );
    assert_eq!(
        owner.retention_ledger_observation(),
        full,
        "every denied route returns reservations and preserves lease identity"
    );

    let released = held
        .pop()
        .expect("capacity includes at least one external lease");
    assert!(matches!(
        world.port.release_exact(released),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let observed = world
        .port
        .observe_current(&reference)
        .expect("one released slot admits a healthy observation twin");
    drop(observed);
    let healthy = world
        .port
        .retain_exact(&world.basis_b)
        .expect("the same released slot admits healthy external retention");
    assert!(matches!(
        world.port.release_exact(healthy),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
}
