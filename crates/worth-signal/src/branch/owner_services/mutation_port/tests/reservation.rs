use crate::branch::{validate_signal_branch_name, SignalBranchForkOperationDenial};

use super::super::super::SignalOwnerCancellationSource;
use super::world::MutationWorld;

fn name(value: &str) -> crate::branch::ValidatedSignalBranchName {
    validate_signal_branch_name(value).expect("reservation test name validates")
}

#[test]
fn held_reservation_occupies_name_and_output_capacity_until_drop() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.retention_ledger_observation();
    let reservation = world
        .port
        .reserve_fork_exact(name("held-reservation"), &world.source_basis)
        .expect("the owner issues a move-only reservation");
    let held = world.owner.retention_ledger_observation();

    assert_eq!(world.owner.reservation_count(), 1);
    assert_eq!(
        held.reserved_admitted_lease_count,
        before.reserved_admitted_lease_count + 1
    );
    assert_eq!(held.used_capacity, before.used_capacity + 1);
    assert!(matches!(
        world
            .port
            .reserve_fork_exact(name("held-reservation"), &world.source_basis),
        Err(SignalBranchForkOperationDenial::NameAlreadyReserved)
    ));

    drop(reservation);
    let after = world.owner.retention_ledger_observation();
    assert_eq!(world.owner.reservation_count(), 0);
    assert_eq!(after.reserved_admitted_lease_count, 0);
    assert_eq!(after.used_capacity, before.used_capacity);

    let retry = world
        .port
        .reserve_fork_exact(name("held-reservation"), &world.source_basis)
        .expect("dropping the first reservation releases its name");
    drop(retry);
}

#[test]
fn successful_consume_installs_once_without_rereserving() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let reservation = world
        .port
        .reserve_fork_exact(name("consumed-reservation"), &world.source_basis)
        .expect("the owner issues the destination reservation");
    assert_eq!(world.owner.reservation_count(), 1);

    let issued = world
        .port
        .fork_reserved_exact(reservation, &SignalOwnerCancellationSource::new().token())
        .expect("the owner consumes its exact destination reservation");
    let after = world.owner.cost_snapshot();

    assert_eq!(issued.created_branch().name, "consumed-reservation");
    assert_eq!(
        issued.created_branch().parent_branch_id,
        Some(world.source_branch.id)
    );
    assert_eq!(world.owner.reservation_count(), 0);
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations() + 1,
        "consume does not re-reserve the destination"
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations() + 1
    );
}

#[test]
fn installed_name_denies_a_second_reservation() {
    let world = MutationWorld::<()>::new();
    let reservation = world
        .port
        .reserve_fork_exact(name("installed-name"), &world.source_basis)
        .expect("the first owner reservation succeeds");
    let _outcome = world
        .port
        .fork_reserved_exact(reservation, &SignalOwnerCancellationSource::new().token())
        .expect("the first reservation installs its branch");

    assert!(matches!(
        world
            .port
            .reserve_fork_exact(name("installed-name"), &world.source_basis),
        Err(SignalBranchForkOperationDenial::NameAlreadyInstalled)
    ));
}

#[test]
fn foreign_source_denies_before_any_destination_reservation_effect() {
    let world = MutationWorld::<()>::new();
    let foreign = MutationWorld::<()>::new();
    let before = world.owner.retention_ledger_observation();
    assert!(matches!(
        world
            .port
            .reserve_fork_exact(name("foreign-source"), &foreign.source_basis),
        Err(SignalBranchForkOperationDenial::BasisMismatch { .. })
    ));
    let after = world.owner.retention_ledger_observation();
    assert_eq!(world.owner.reservation_count(), 0);
    assert_eq!(after.used_capacity, before.used_capacity);
    assert_eq!(after.reserved_admitted_lease_count, 0);
}

#[test]
fn foreign_mutation_port_cannot_consume_another_owner_reservation() {
    let world = MutationWorld::<()>::new();
    let foreign = MutationWorld::<()>::new();
    let before = world.owner.retention_ledger_observation();
    let foreign_cost_before = foreign.owner.cost_snapshot();
    let reservation = world
        .port
        .reserve_fork_exact(name("foreign-port"), &world.source_basis)
        .expect("the issuing port owns the reservation");

    assert!(matches!(
        foreign
            .port
            .fork_reserved_exact(reservation, &SignalOwnerCancellationSource::new().token()),
        Err(SignalBranchForkOperationDenial::OwnerUnavailable(_))
    ));
    let after = world.owner.retention_ledger_observation();
    assert_eq!(world.owner.reservation_count(), 0);
    assert_eq!(after.used_capacity, before.used_capacity);
    assert_eq!(after.reserved_admitted_lease_count, 0);
    assert_eq!(foreign.owner.cost_snapshot(), foreign_cost_before);
}

#[test]
fn stale_source_denies_before_any_destination_reservation_effect() {
    let world = MutationWorld::<()>::new();
    world
        .port
        .advance_exact(
            &world.source_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the source can move before a held reservation is requested");
    let before = world.owner.retention_ledger_observation();

    assert!(matches!(
        world
            .port
            .reserve_fork_exact(name("stale-before-issuance"), &world.source_basis),
        Err(SignalBranchForkOperationDenial::BasisMismatch { .. })
    ));
    let after = world.owner.retention_ledger_observation();
    assert_eq!(world.owner.reservation_count(), 0);
    assert_eq!(after.used_capacity, before.used_capacity);
    assert_eq!(after.reserved_admitted_lease_count, 0);
}

#[test]
fn source_staleness_after_issuance_denies_on_consume_and_releases_capacity() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.retention_ledger_observation();
    let reservation = world
        .port
        .reserve_fork_exact(name("stale-after-issuance"), &world.source_basis)
        .expect("the current exact source issues a reservation");
    world
        .port
        .advance_exact(
            &world.source_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("a source movement can race a held reservation");

    assert!(matches!(
        world
            .port
            .fork_reserved_exact(reservation, &SignalOwnerCancellationSource::new().token()),
        Err(SignalBranchForkOperationDenial::BasisMismatch { .. })
    ));
    let after = world.owner.retention_ledger_observation();
    assert_eq!(world.owner.reservation_count(), 0);
    assert_eq!(after.used_capacity, before.used_capacity);
    assert_eq!(after.reserved_admitted_lease_count, 0);
}

#[test]
fn closed_owner_denies_consumption_and_drops_pending_capacity() {
    let world = MutationWorld::<()>::new();
    let reservation = world
        .port
        .reserve_fork_exact(name("closed-owner"), &world.source_basis)
        .expect("the open owner issues a destination reservation");
    world
        .owner
        .close()
        .expect("the owner closes after issuance");

    assert!(matches!(
        world
            .port
            .fork_reserved_exact(reservation, &SignalOwnerCancellationSource::new().token()),
        Err(SignalBranchForkOperationDenial::OwnerUnavailable(_))
    ));
    assert_eq!(world.owner.reservation_count(), 0);
}

#[test]
fn reservation_capacity_denial_uses_the_existing_bounded_registry() {
    let world = MutationWorld::<()>::new();
    let admission = world.owner.admit().expect("capacity setup admits");
    let mut held = Vec::new();
    loop {
        let held_name = name(&format!("internal-held-{}", held.len()));
        match world
            .owner
            .reserve_fork_destination(&admission, &world.source_basis, held_name)
        {
            Ok(reservation) => held.push(reservation),
            Err(SignalBranchForkOperationDenial::ReservationCapacityExhausted { .. }) => break,
            Err(denial) => panic!("unexpected reservation setup denial: {denial:?}"),
        }
    }

    assert!(matches!(
        world
            .port
            .reserve_fork_exact(name("public-capacity"), &world.source_basis),
        Err(SignalBranchForkOperationDenial::ReservationCapacityExhausted { .. })
    ));
    drop(held);
    drop(admission);
    assert_eq!(world.owner.reservation_count(), 0);
}
