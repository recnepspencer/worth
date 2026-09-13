use crate::branch::SignalBranchRetentionTerminalOutcome;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;

use super::super::super::SignalOwnerCancellationSource;
use super::super::runtime_root::runtime_with_two_branches_from_graph;

#[test]
fn retention_preflight_accepts_an_exact_available_historical_target_without_currentness() {
    let mut graph = SignalGraph::new();
    let first = graph.create_node();
    let second = graph.create_node();
    let third = graph.create_node();
    let dependent = graph.create_node();
    graph
        .set_dependencies(dependent, [DependencyEdge::new(first, Aspect::new(1))])
        .expect("historical retention fixture installs");
    let (mut runtime, _, branch, basis) = runtime_with_two_branches_from_graph(graph);
    let (_, mutation, _) = runtime.owner_port_slots().expect("historical owner seals");
    let owner = mutation
        .upgrade_owner()
        .expect("historical owner remains live");
    let admission = owner.admit().expect("historical acquisition admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("historical target cell is live");

    let first_advance = owner
        .reserve_advance_output(&admission, &cell)
        .expect("first semantic output reserves")
        .advance::<(), (), _>(
            &basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                transaction
                    .set_dependencies(dependent, [DependencyEdge::new(second, Aspect::new(2))])
            },
        )
        .into_result()
        .expect("first semantic mutation performs");
    let (first_basis, _) = first_advance.into_parts();
    let first_capture = owner
        .reserve_snapshot_outputs(&admission, &cell)
        .expect("first capture outputs reserve")
        .capture(&first_basis, &SignalOwnerCancellationSource::new().token())
        .expect("first historical snapshot captures")
        .into_outcome();
    let (first_snapshot, historical_basis) = first_capture.into_parts();

    let second_advance = owner
        .reserve_advance_output(&admission, &cell)
        .expect("second semantic output reserves")
        .advance::<(), (), _>(
            &historical_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                transaction
                    .set_dependencies(dependent, [DependencyEdge::new(third, Aspect::new(3))])
            },
        )
        .into_result()
        .expect("the live cell advances beyond the historical basis");
    let (second_basis, _) = second_advance.into_parts();
    let second_capture = owner
        .reserve_snapshot_outputs(&admission, &cell)
        .expect("second capture outputs reserve")
        .capture(&second_basis, &SignalOwnerCancellationSource::new().token())
        .expect("new current snapshot captures")
        .into_outcome();
    let (second_snapshot, current_basis) = second_capture.into_parts();
    assert_ne!(
        first_snapshot.snapshot().meta.snapshot_id,
        second_snapshot.snapshot().meta.snapshot_id
    );
    assert_ne!(
        historical_basis.observation().generation(),
        current_basis.observation().generation()
    );
    assert_eq!(
        historical_basis
            .observation()
            .target()
            .as_basis()
            .and_then(|target| target.snapshot_id()),
        Some(first_snapshot.snapshot().meta.snapshot_id.0)
    );
    assert_eq!(
        current_basis
            .observation()
            .target()
            .as_basis()
            .and_then(|target| target.snapshot_id()),
        Some(second_snapshot.snapshot().meta.snapshot_id.0)
    );

    let ledger_before = owner.retention_ledger_observation();
    let contacts_before = owner.cost_snapshot().retention_registry_contacts();
    let lease = owner
        .acquire_external_retention(&admission, &historical_basis)
        .expect("available historical target is lawful without a currentness check");
    let acquired = owner.retention_ledger_observation();
    assert_eq!(acquired.next_lease_id, ledger_before.next_lease_id + 1);
    assert_eq!(acquired.used_capacity, ledger_before.used_capacity + 1);
    assert_eq!(acquired.external_count_by_branch, vec![(branch.id, 1)]);
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        contacts_before + 1
    );
    assert_eq!(
        lease.release().outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    let mut released = ledger_before;
    released.next_lease_id += 1;
    assert_eq!(owner.retention_ledger_observation(), released);
}
