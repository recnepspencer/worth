use crate::branch::admit_runtime_signal_branch_observation;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::super::SignalOwnerCancellationSource;

mod concurrency;
mod high_water;
mod recovery;
mod reservation_lifecycle;
mod restore_selection;

#[test]
fn distinct_sibling_snapshots_do_not_alias_exact_retention_targets() {
    let mut graph = SignalGraph::new();
    let weather = graph.create_node();
    let berth = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
        .expect("dispatch initially depends on weather");
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("real runtime admits the shared starting state");
    let (storm, storm_basis) = runtime
        .fork_signal_branch("storm-route", &initial)
        .expect("storm branch forks from the shared source")
        .into_parts();
    let (maintenance, maintenance_basis) = runtime
        .fork_signal_branch("maintenance-route", &initial)
        .expect("maintenance branch forks from the same source")
        .into_parts();
    assert_ne!(storm.id, maintenance.id);
    assert_eq!(storm.parent_branch_id, maintenance.parent_branch_id);

    let (_, mutation, _) = runtime
        .owner_port_slots()
        .expect("real owner partition seals");
    let owner = mutation.upgrade_owner().expect("runtime retains its owner");
    let admission = owner.admit().expect("snapshot scenario admits");
    let storm_cell = owner
        .lookup_cell(&admission, storm.id)
        .expect("storm exists");
    let maintenance_cell = owner
        .lookup_cell(&admission, maintenance.id)
        .expect("maintenance exists");
    let cancellation = SignalOwnerCancellationSource::new();
    let output_retention = owner
        .acquire_admitted_retention(&admission, storm.id)
        .expect("movement output retention reserves before movement");
    let changed = storm_cell
        .advance_exact::<(), (), _>(
            &admission,
            &storm_basis,
            &mut (),
            &cancellation.token(),
            |transaction| {
                transaction.set_dependencies(dispatch, [DependencyEdge::new(berth, Aspect::new(0))])
            },
        )
        .expect("storm route changes its real dispatch dependency");
    let (observation, _) = changed.into_parts();
    let changed_basis =
        admit_runtime_signal_branch_observation(observation, storm.id, output_retention);

    let storm_snapshot = storm_cell
        .capture_snapshot_exact(
            &changed_basis,
            owner
                .metadata
                .reserve_snapshot(&admission, &storm_cell)
                .expect("storm reserves"),
            &cancellation.token(),
        )
        .expect("storm snapshot captures changed state");
    let maintenance_snapshot = maintenance_cell
        .capture_snapshot_exact(
            &maintenance_basis,
            owner
                .metadata
                .reserve_snapshot(&admission, &maintenance_cell)
                .expect("maintenance reserves"),
            &cancellation.token(),
        )
        .expect("maintenance snapshot captures unchanged sibling state");
    assert_eq!(
        storm_snapshot
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![berth]),
        "storm capture contains the performed dependency change"
    );
    assert_eq!(
        maintenance_snapshot
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![weather]),
        "maintenance capture independently preserves its original dependency"
    );

    let storm_id = storm_snapshot.snapshot().meta.snapshot_id;
    let maintenance_id = maintenance_snapshot.snapshot().meta.snapshot_id;
    let storm_retained_basis = admit_runtime_signal_branch_observation(
        storm_snapshot.observation().clone(),
        storm.id,
        owner
            .acquire_admitted_retention(&admission, storm.id)
            .expect("the storm snapshot target receives admitted custody"),
    );
    let maintenance_retained_basis = admit_runtime_signal_branch_observation(
        maintenance_snapshot.observation().clone(),
        maintenance.id,
        owner
            .acquire_admitted_retention(&admission, maintenance.id)
            .expect("the maintenance snapshot target receives admitted custody"),
    );
    let storm_lease = owner
        .acquire_external_retention(&admission, &storm_retained_basis)
        .expect("the actual storm snapshot is retained");
    let maintenance_lease = owner
        .acquire_external_retention(&admission, &maintenance_retained_basis)
        .expect("the actual maintenance snapshot is retained");
    let targets_distinct = storm_lease.retained_target() != maintenance_lease.retained_target();
    let receipt = storm_lease.release();
    assert_eq!(
        (
            storm_id != maintenance_id,
            targets_distinct,
            receipt.remaining_target_leases(),
            receipt.remaining_branch_leases(),
        ),
        (true, true, 0, 0),
        "different sibling states require distinct owner snapshot IDs and exact targets; storm={storm_id:?}, maintenance={maintenance_id:?}"
    );
    assert_eq!(maintenance_lease.release().remaining_target_leases(), 0);
}
