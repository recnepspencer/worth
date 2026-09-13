use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use crate::branch::{
    validate_signal_branch_name, SignalBranchAdvanceDenial, SignalBranchForkOperationDenial,
    SignalBranchRestoreDenial, SignalBranchRetentionAcquisitionDenial,
    SignalBranchSnapshotCaptureDenial, SignalOwnerLifecycleObservation,
};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;

use super::super::SignalOwnerCancellationSource;
use super::progress_bound::{observe_within, PROGRESS_BOUND};
use super::runtime_root::{runtime_with_two_branches, runtime_with_two_branches_from_graph};

const MAXIMUM_ACTIVE_LEASES: usize = 4_096;

#[test]
fn admitted_output_capacity_reserves_pre_effect_and_cancellation_restores_exactly() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("capacity fixture admits");
    assert_eq!(owner.admitted_retention_count(branch.id), 1);

    let reservation = owner
        .reserve_admitted_output_slots_for_test(&admission, branch.id, MAXIMUM_ACTIVE_LEASES - 1)
        .expect("all remaining output capacity reserves before movement");
    assert_eq!(
        owner.admitted_retention_count(branch.id),
        1,
        "pending output capacity is not fabricated issued authority"
    );
    assert!(matches!(
        owner.reserve_admitted_output_slots_for_test(&admission, branch.id, 1),
        Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
            maximum_active_leases: MAXIMUM_ACTIVE_LEASES
        })
    ));
    drop(reservation);

    let healthy = owner
        .reserve_admitted_output_slots_for_test(&admission, branch.id, MAXIMUM_ACTIVE_LEASES - 1)
        .expect("cancellation returns every unused slot exactly");
    drop(healthy);
    drop(basis);
    assert_eq!(owner.admitted_retention_count(branch.id), 0);
}

#[test]
fn carried_advance_output_handoff_survives_close_fencing_new_work() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("operation admits before close");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the close-race target cell is live");
    let reservation = owner
        .reserve_advance_output(&admission, &cell)
        .expect("advance output capacity reserves before movement");
    let cancellation = SignalOwnerCancellationSource::new();
    let performed = reservation
        .advance::<(), (), _>(&basis, &mut (), &cancellation.token(), |_| Ok(()))
        .into_result()
        .expect("the pre-close admitted movement performs");
    let (closed_tx, closed_rx) = mpsc::sync_channel(1);
    let closing_owner = Arc::clone(&owner);
    thread::spawn(move || {
        let _ = closed_tx.send(closing_owner.close());
    });
    let observed_owner = Arc::clone(&owner);
    assert_eq!(
        observe_within(move || {
            (observed_owner.lifecycle_observation() == SignalOwnerLifecycleObservation::Closing)
                .then_some(SignalOwnerLifecycleObservation::Closing)
        }),
        Ok(SignalOwnerLifecycleObservation::Closing)
    );

    let (refreshed, _) = performed.into_parts();
    assert_eq!(refreshed.owner_branch_id(), branch.id);
    assert_eq!(refreshed.observation().generation().get(), 1);
    drop(refreshed);
    drop(basis);
    drop(admission);
    assert_eq!(closed_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
}

#[test]
fn every_named_output_seam_denies_at_its_exact_pre_effect_capacity() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("output preflight admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the capacity target cell is live");

    let one_slot_left = owner
        .reserve_admitted_output_slots_for_test(&admission, branch.id, MAXIMUM_ACTIVE_LEASES - 2)
        .expect("fixture leaves exactly one output slot");
    assert!(matches!(
        owner.reserve_snapshot_outputs(&admission, &cell),
        Err(SignalBranchSnapshotCaptureDenial::RetentionUnavailable {
            denial: SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases: MAXIMUM_ACTIVE_LEASES,
            }
        })
    ));
    drop(one_slot_left);
    let unused_snapshot = owner
        .reserve_snapshot_outputs(&admission, &cell)
        .expect("capture reserves both outputs after denial");
    drop(unused_snapshot);

    let full = owner
        .reserve_admitted_output_slots_for_test(&admission, branch.id, MAXIMUM_ACTIVE_LEASES - 1)
        .expect("fixture fills every remaining slot");
    assert!(matches!(
        owner.reserve_advance_output(&admission, &cell),
        Err(SignalBranchAdvanceDenial::RetentionUnavailable {
            denial: SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases: MAXIMUM_ACTIVE_LEASES,
            }
        })
    ));
    assert!(matches!(
        owner.reserve_restore_output(&admission, &cell),
        Err(SignalBranchRestoreDenial::RetentionUnavailable {
            denial: SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases: MAXIMUM_ACTIVE_LEASES,
            }
        })
    ));
    assert!(matches!(
        owner.reserve_fork_output(&admission, &cell),
        Err(SignalBranchForkOperationDenial::RetentionUnavailable {
            denial: SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases: MAXIMUM_ACTIVE_LEASES,
            }
        })
    ));
    drop(full);
    let healthy = owner
        .reserve_admitted_output_slots_for_test(&admission, branch.id, MAXIMUM_ACTIVE_LEASES - 1)
        .expect("all unused named reservations returned their capacity exactly");
    drop(healthy);
    drop(basis);
}

#[test]
fn named_outputs_convert_populated_advance_capture_restore_and_fork_movements() {
    let mut graph = SignalGraph::new();
    let weather = graph.create_node();
    let berth = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
        .expect("the populated output world installs");
    let (mut runtime, _, branch, starting_basis) = runtime_with_two_branches_from_graph(graph);
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("output movements admit");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the populated source cell is live");
    let cancellation = SignalOwnerCancellationSource::new();

    let advance_output = owner
        .reserve_advance_output(&admission, &cell)
        .expect("advance output reserves before movement");
    let advanced = advance_output
        .advance::<(), (), _>(
            &starting_basis,
            &mut (),
            &cancellation.token(),
            |transaction| {
                transaction.set_dependencies(dispatch, [DependencyEdge::new(berth, Aspect::new(0))])
            },
        )
        .into_result()
        .expect("the populated source advances");
    let (advanced_basis, advanced_transaction) = advanced.into_parts();
    assert!(advanced_transaction.touched_nodes > 0);

    let snapshot_output = owner
        .reserve_snapshot_outputs(&admission, &cell)
        .expect("capture reserves snapshot plus refreshed basis");
    let capture = snapshot_output
        .capture(&advanced_basis, &cancellation.token())
        .expect("the populated source captures");
    let capture_outcome = capture.into_outcome();
    let (snapshot, captured_basis) = capture_outcome.into_parts();
    assert_eq!(
        snapshot
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![berth])
    );

    let second_advance_output = owner
        .reserve_advance_output(&admission, &cell)
        .expect("a second advance output reserves");
    let reverted = second_advance_output
        .advance::<(), (), _>(
            &captured_basis,
            &mut (),
            &cancellation.token(),
            |transaction| {
                transaction
                    .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
            },
        )
        .into_result()
        .expect("the live cell diverges after capture");
    let (reverted_basis, reverted_transaction) = reverted.into_parts();
    assert!(reverted_transaction.touched_nodes > 0);

    let restore_output = owner
        .reserve_restore_output(&admission, &cell)
        .expect("restore output reserves before movement");
    let restored = restore_output
        .restore(&reverted_basis, &snapshot, &cancellation.token())
        .expect("the capture restores its berth dependency");
    let restored_basis = restored.into_basis();
    cell.with_state(&admission, |state, _| {
        assert_eq!(
            state.state().graph().dependency_sources_of(dispatch),
            Ok(vec![berth])
        );
    })
    .expect("the restored cell remains healthy");

    let source_retention_before_fork = owner.admitted_or_reserved_retention_count(branch.id);
    let fork_output = owner
        .reserve_fork_output(&admission, &cell)
        .expect("fork destination output reserves on the source branch");
    let installed = fork_output
        .fork(
            &restored_basis,
            validate_signal_branch_name("ready-output-destination")
                .expect("the destination identity is valid"),
            &cancellation.token(),
        )
        .expect("the populated destination installs");
    let destination_cell = Arc::clone(installed.installed().cell());
    let destination_id = destination_cell.branch_id();
    let issued_handle = destination_cell
        .with_state(&admission, |state, _| state.handle().clone())
        .expect("the installed destination exposes its owner-issued handle");
    assert_eq!(owner.admitted_retention_count(destination_id), 0);
    assert_eq!(
        owner.admitted_or_reserved_retention_count(destination_id),
        1
    );
    assert_eq!(
        owner.admitted_or_reserved_retention_count(branch.id),
        source_retention_before_fork,
        "the published destination reservation is not left charged to the source"
    );
    let (destination_handle, destination_basis) = installed.into_destination_parts();
    assert_eq!(destination_handle, issued_handle);
    assert_eq!(destination_handle.id, destination_id);
    assert_eq!(destination_handle.parent_branch_id, Some(branch.id));
    assert_eq!(destination_basis.owner_branch_id(), destination_id);
    assert_eq!(owner.admitted_retention_count(destination_id), 1);
    assert_eq!(
        owner.admitted_or_reserved_retention_count(destination_id),
        1
    );
    destination_cell
        .with_state(&admission, |state, _| {
            assert_eq!(
                state.state().graph().dependency_sources_of(dispatch),
                Ok(vec![berth])
            );
        })
        .expect("the fork output names the populated destination cell");
}

#[test]
fn ready_fork_output_pins_its_published_destination_and_drop_returns_capacity() {
    let (mut runtime, _, source_branch, source_basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("fork output admits");
    let source_cell = owner
        .lookup_cell(&admission, source_branch.id)
        .expect("the source cell is live");
    let cancellation = SignalOwnerCancellationSource::new();

    let ready = owner
        .reserve_fork_output(&admission, &source_cell)
        .expect("fork output capacity reserves before movement")
        .fork(
            &source_basis,
            validate_signal_branch_name("ready-output-cancellation")
                .expect("the destination identity is valid"),
            &cancellation.token(),
        )
        .expect("the destination publishes with its pending output");
    let destination_id = ready.installed().cell().branch_id();
    assert_eq!(owner.admitted_retention_count(destination_id), 0);
    assert_eq!(
        owner.admitted_or_reserved_retention_count(destination_id),
        1,
        "the published destination is protected before basis conversion"
    );
    assert_eq!(
        owner.admitted_or_reserved_retention_count(source_branch.id),
        1,
        "the pending destination capacity is no longer charged to the source"
    );

    drop(ready);
    assert_eq!(
        owner.admitted_or_reserved_retention_count(destination_id),
        0,
        "ready-output cancellation returns the destination slot exactly"
    );
    assert_eq!(
        owner.admitted_or_reserved_retention_count(source_branch.id),
        1
    );
}
