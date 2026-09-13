use crate::branch::{
    SignalBranchRetainedReadmissionDenial, SignalBranchRetentionReleaseDenial,
    SignalBranchRetentionReleaseOutcome, SignalBranchRetentionTerminalOutcome,
    SignalOwnerLifecycleObservation,
};

use super::world::{advance_exact, basis_port_world, issue_reference};

#[test]
fn basis_port_observation_and_readmission_method_matrix_uses_one_real_cell() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let expected = world.basis_b.observation().clone();
    let before = world
        .port
        .owner_service_cost_snapshot()
        .expect("the rooted port reports structural work");

    let observed = world
        .port
        .observe_current(&reference)
        .expect("managed observation admits the current exact basis");
    let readmitted = world
        .port
        .readmit_exact(&reference, world.basis_b.descriptor())
        .expect("managed readmission compares the complete descriptor");
    world
        .port
        .compare_current_exact(&readmitted)
        .expect("the newly readmitted basis is current");
    let after = world
        .port
        .owner_service_cost_snapshot()
        .expect("the rooted port reports post-operation work");

    assert_eq!(observed.observation(), &expected);
    assert_eq!(readmitted.observation(), &expected);
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 4
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups() + 3
    );
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations()
    );
    assert_eq!(
        after.target_cell_contacts(),
        before.target_cell_contacts() + 3
    );
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts(),
        "canonical Ready reuse validates without a retention-owner contact"
    );
    assert_eq!(
        after.branch_registry_entries_scanned(),
        before.branch_registry_entries_scanned()
    );
}

#[test]
fn retained_readmission_preserves_historical_target_and_release_custody() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let historical = world.basis_b.descriptor().clone();
    let first = world
        .port
        .retain_exact(&world.basis_b)
        .expect("the first exact obligation opens");
    let second = world
        .port
        .retain_exact(&world.basis_b)
        .expect("the sibling exact obligation opens independently");
    let moved = advance_exact(&world.port, &world.basis_b);
    assert_ne!(moved, *historical.observation());

    let retained = world
        .port
        .readmit_retained_exact(&historical, &first)
        .expect("the live obligation readmits its historical target");
    let current = world
        .port
        .observe_current(&reference)
        .expect("the managed reference follows canonical movement");
    let later = world
        .port
        .retain_exact(&current)
        .expect("a later generation opens its own exact obligation");
    assert_eq!(retained.observation(), historical.observation());
    assert_eq!(current.observation(), &moved);

    let first_receipt = match world.port.release_exact(first) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("the issuing port releases its first obligation: {other:?}"),
    };
    assert_eq!(
        first_receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    assert_eq!(first_receipt.remaining_target_leases(), 2);
    assert_eq!(first_receipt.remaining_branch_leases(), 2);
    let second_receipt = match world.port.release_exact(second) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("the sibling obligation remains independently releasable: {other:?}"),
    };
    assert_eq!(second_receipt.remaining_target_leases(), 1);
    assert_eq!(second_receipt.remaining_branch_leases(), 1);
    let later_receipt = match world.port.release_exact(later) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("the later generation remains independently releasable: {other:?}"),
    };
    assert_eq!(later_receipt.remaining_target_leases(), 0);
    assert_eq!(later_receipt.remaining_branch_leases(), 0);
}

#[test]
fn foreign_retention_custody_returns_the_live_lease_to_its_issuer() {
    let issuer = basis_port_world();
    let receiver = basis_port_world();
    let descriptor = issuer.basis_b.descriptor().clone();
    let lease = issuer
        .port
        .retain_exact(&issuer.basis_b)
        .expect("the issuing port opens one exact obligation");

    assert!(matches!(
        receiver.port.readmit_retained_exact(&descriptor, &lease),
        Err(SignalBranchRetainedReadmissionDenial::ForeignRetention)
    ));
    let lease = match receiver.port.release_exact(lease) {
        SignalBranchRetentionReleaseOutcome::Denied {
            lease,
            denial: SignalBranchRetentionReleaseDenial::ForeignRuntime,
        } => lease,
        other => panic!("the foreign port returns the still-live obligation: {other:?}"),
    };
    assert!(matches!(
        issuer.port.release_exact(lease),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
}

#[test]
fn lifecycle_and_cost_inspection_account_for_their_own_weak_upgrades() {
    let world = basis_port_world();
    assert_eq!(
        world.port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Open
    );
    let owner = world
        .port
        .upgrade_owner()
        .expect("the open owner remains strongly rooted");
    let first = world
        .port
        .owner_service_cost_snapshot()
        .expect("the live owner reports its first diagnostic snapshot");
    assert_eq!(first, owner.cost_snapshot());
    let second = world
        .port
        .owner_service_cost_snapshot()
        .expect("the live owner reports its second diagnostic snapshot");
    assert_eq!(second, owner.cost_snapshot());
    assert_eq!(
        second.owner_upgrade_attempts(),
        first.owner_upgrade_attempts() + 1
    );
    assert_eq!(
        second.admission_records_scanned(),
        first.admission_records_scanned()
    );
    assert_eq!(
        second.branch_registry_lookups(),
        first.branch_registry_lookups()
    );
    assert_eq!(
        second.branch_registry_reservations(),
        first.branch_registry_reservations()
    );
    assert_eq!(
        second.branch_registry_entries_scanned(),
        first.branch_registry_entries_scanned()
    );
    assert_eq!(second.target_cell_contacts(), first.target_cell_contacts());
    assert_eq!(second.target_cell_waits(), first.target_cell_waits());
    assert_eq!(second.canonical_movements(), first.canonical_movements());
    assert_eq!(
        second.retention_registry_contacts(),
        first.retention_registry_contacts()
    );
    assert_eq!(second.fork_source_captures(), first.fork_source_captures());
    assert_eq!(
        second.fork_destination_preparations(),
        first.fork_destination_preparations()
    );
    assert_eq!(
        second.fork_destination_installations(),
        first.fork_destination_installations()
    );
    assert_eq!(
        second.forked_mutable_graph_nodes_copied(),
        first.forked_mutable_graph_nodes_copied()
    );
    assert_eq!(
        second.diagnostic_events_recorded(),
        first.diagnostic_events_recorded()
    );
    assert_eq!(
        second.diagnostic_events_dropped(),
        first.diagnostic_events_dropped()
    );
    assert_eq!(second.close_batches(), first.close_batches());
}
