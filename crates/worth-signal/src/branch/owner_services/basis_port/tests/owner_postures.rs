use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use worth_proof::TransitionOutcome;

use crate::branch::{
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchBasisObservationDenial,
    SignalBranchBasisReadmissionDenial, SignalBranchRetainedReadmissionDenial,
    SignalBranchRetentionReleaseDenial, SignalBranchRetentionReleaseOutcome,
    SignalBranchRetentionTerminalOutcome, SignalBranchRetirementReason,
    SignalOwnerLifecycleObservation,
};

use super::world::{
    assert_retention_cleanup_with_identity_advance, basis_port_world, issue_reference,
};

const PROGRESS_BOUND: Duration = Duration::from_secs(2);

#[test]
fn retired_quarantined_closing_and_gone_postures_remain_exact() {
    retired_reference_is_terminal();
    quarantined_cell_is_not_flattened_to_unknown();
    closing_owner_denies_new_basis_work();
    gone_owner_is_stably_unavailable();
}

fn retired_reference_is_terminal() {
    let (mut runtime, _, branch, basis) =
        super::super::super::tests::runtime_root::runtime_with_two_branches();
    let plan = match runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the real runtime issues linear retirement authority: {other:?}"),
    };
    let (port, _, _) = runtime.owner_port_slots().expect("the runtime seals");
    let reference = issue_reference(&port, plan.admitted_basis());
    let owner = port.upgrade_owner().expect("the sealed owner remains live");
    let admission = owner.admit().expect("retirement admits");
    owner
        .reserve_retirement(&admission, branch.id)
        .expect("the exact retirement reserves its canonical target")
        .execute(
            plan,
            &super::super::super::SignalOwnerCancellationSource::new().token(),
        )
        .expect("the exact retirement performs");
    drop(admission);

    assert!(matches!(
        port.observe_current(&reference),
        Err(SignalBranchBasisObservationDenial::ManagedReferenceDenied {
            denial: ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded,
        })
    ));
}

fn quarantined_cell_is_not_flattened_to_unknown() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let admission = owner.admit().expect("the panic operation admits");
    let cell = owner
        .lookup_cell(&admission, world.branch_b.id)
        .expect("the production registry supplies branch B");
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = cell.advance_exact::<(), (), _>(
            &admission,
            &world.basis_b,
            &mut (),
            &super::super::super::SignalOwnerCancellationSource::new().token(),
            |_| panic!("inject branch-local transaction panic"),
        );
    }));
    assert!(panic.is_err());
    drop(admission);
    let before = owner.retention_ledger_observation();

    assert!(matches!(
        world
            .port
            .readmit_exact(&reference, world.basis_b.descriptor()),
        Err(SignalBranchBasisReadmissionDenial::QuarantinedBranch { branch_id })
            if branch_id == world.branch_b.id
    ));
    assert_retention_cleanup_with_identity_advance(
        &before,
        &owner.retention_ledger_observation(),
        0,
    );
}

fn closing_owner_denies_new_basis_work() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let descriptor = world.basis_b.descriptor().clone();
    let lease = world
        .port
        .retain_exact(&world.basis_b)
        .expect("the matching lease opens before close");
    let prior_admission = owner.admit().expect("work admits before close");
    let closing_owner = owner.clone();
    let (closed_tx, closed_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = closed_tx.send(closing_owner.close());
    });
    let deadline = Instant::now() + PROGRESS_BOUND;
    while owner.lifecycle_observation() != SignalOwnerLifecycleObservation::Closing
        && Instant::now() < deadline
    {
        thread::yield_now();
    }
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    let before_closing_snapshot = owner.cost_snapshot();
    assert!(world.port.owner_service_cost_snapshot().is_err());
    assert_only_owner_upgrade_changed(before_closing_snapshot, owner.cost_snapshot());
    assert!(matches!(
        world.port.observe_current(&reference),
        Err(SignalBranchBasisObservationDenial::OwnerUnavailable(_))
    ));
    assert!(matches!(
        world.port.readmit_retained_exact(&descriptor, &lease),
        Err(SignalBranchRetainedReadmissionDenial::OwnerUnavailable(_))
    ));
    let lease = match world.port.release_exact(lease) {
        SignalBranchRetentionReleaseOutcome::Denied {
            lease,
            denial: SignalBranchRetentionReleaseDenial::OwnerUnavailable(_),
        } => lease,
        other => panic!("post-closing weak release returns its live lease: {other:?}"),
    };
    assert_eq!(
        lease.release().outcome(),
        SignalBranchRetentionTerminalOutcome::OwnerUnavailable
    );
    drop(prior_admission);
    assert_eq!(closed_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    let before_closed_snapshot = owner.cost_snapshot();
    assert!(world.port.owner_service_cost_snapshot().is_err());
    assert_only_owner_upgrade_changed(before_closed_snapshot, owner.cost_snapshot());
}

fn gone_owner_is_stably_unavailable() {
    let world = basis_port_world();
    let port = world.port.clone();
    let reference = issue_reference(&port, &world.basis_b);
    let lease = port
        .retain_exact(&world.basis_b)
        .expect("the rooted owner opens one external obligation");
    drop(world);

    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert!(port.owner_service_cost_snapshot().is_err());
    assert!(port.owner_service_cost_snapshot().is_err());
    assert!(matches!(
        port.observe_current(&reference),
        Err(SignalBranchBasisObservationDenial::OwnerUnavailable(_))
    ));
    let lease = match port.release_exact(lease) {
        SignalBranchRetentionReleaseOutcome::Denied {
            lease,
            denial: SignalBranchRetentionReleaseDenial::OwnerUnavailable(_),
        } => lease,
        other => panic!("weak-port loss returns the still-live lease: {other:?}"),
    };
    assert_eq!(
        lease.release().outcome(),
        SignalBranchRetentionTerminalOutcome::OwnerUnavailable,
        "the direct lease path remains terminal and records owner loss once"
    );
}

fn assert_only_owner_upgrade_changed(
    before: super::super::super::SignalOwnerServiceCostSnapshot,
    after: super::super::super::SignalOwnerServiceCostSnapshot,
) {
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 1
    );
    assert_eq!(
        after.admission_records_scanned(),
        before.admission_records_scanned()
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups()
    );
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations()
    );
    assert_eq!(
        after.branch_registry_entries_scanned(),
        before.branch_registry_entries_scanned()
    );
    assert_eq!(after.target_cell_contacts(), before.target_cell_contacts());
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts()
    );
    assert_eq!(after.fork_source_captures(), before.fork_source_captures());
    assert_eq!(
        after.fork_destination_preparations(),
        before.fork_destination_preparations()
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations()
    );
    assert_eq!(
        after.forked_mutable_graph_nodes_copied(),
        before.forked_mutable_graph_nodes_copied()
    );
    assert_eq!(
        after.diagnostic_events_recorded(),
        before.diagnostic_events_recorded()
    );
    assert_eq!(
        after.diagnostic_events_dropped(),
        before.diagnostic_events_dropped()
    );
    assert_eq!(after.close_batches(), before.close_batches());
}
