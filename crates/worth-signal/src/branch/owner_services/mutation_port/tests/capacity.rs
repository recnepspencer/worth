use crate::branch::{validate_signal_branch_name, SignalBranchForkOperationDenial};

use super::super::super::SignalOwnerCancellationSource;
use super::world::MutationWorld;

#[test]
fn fork_registry_reservation_exhaustion_precedes_capture_and_releases_cleanly() {
    let world = MutationWorld::<()>::new();
    let admission = world.owner.admit().expect("capacity setup admits");
    let mut held = Vec::new();
    loop {
        let name = validate_signal_branch_name(format!("held-fork-reservation-{}", held.len()))
            .expect("reservation name validates");
        match world
            .owner
            .reserve_fork_destination(&admission, &world.source_basis, name)
        {
            Ok(reservation) => held.push(reservation),
            Err(SignalBranchForkOperationDenial::ReservationCapacityExhausted { .. }) => break,
            Err(denial) => panic!("unexpected reservation setup denial: {denial:?}"),
        }
    }
    assert!(
        !held.is_empty(),
        "the bounded reservation table was exercised"
    );
    assert_eq!(world.owner.reservation_count(), held.len());
    let ledger_before = world.owner.retention_ledger_observation();
    let before = world.owner.cost_snapshot();

    assert!(matches!(
        world.port.fork_exact(
            validate_signal_branch_name("port-reservation-capacity-denial")
                .expect("port name validates"),
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchForkOperationDenial::ReservationCapacityExhausted { .. })
    ));
    let after = world.owner.cost_snapshot();
    let ledger_after = world.owner.retention_ledger_observation();
    assert_eq!(after.target_cell_contacts(), before.target_cell_contacts());
    assert_eq!(after.fork_source_captures(), before.fork_source_captures());
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations()
    );
    assert_eq!(ledger_after.used_capacity, ledger_before.used_capacity);
    assert_eq!(ledger_after.reserved_admitted_lease_count, 0);

    drop(held);
    drop(admission);
    assert_eq!(world.owner.reservation_count(), 0);
    world
        .port
        .fork_exact(
            validate_signal_branch_name("port-reservation-capacity-recovered")
                .expect("healthy name validates"),
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("dropping every held reservation permits a healthy real-port fork");
}
