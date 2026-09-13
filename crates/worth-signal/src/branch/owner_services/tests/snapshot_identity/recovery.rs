use crate::branch::{admit_runtime_signal_branch_observation, AdmittedSignalBranchSnapshot};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::super::super::SignalOwnerCancellationSource;

#[test]
fn capture_restore_capture_preserves_old_contents_and_issues_a_fresh_key() {
    let mut graph = SignalGraph::new();
    let weather = graph.create_node();
    let berth = graph.create_node();
    let depot = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
        .expect("the initial dependency installs");
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let branch = runtime.current_branch();
    let initial = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("the runtime admits its initial branch");
    let (_, mutation, _) = runtime.owner_port_slots().expect("the runtime seals");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("snapshot work admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the target cell is installed");
    let cancellation = SignalOwnerCancellationSource::new();

    let changed = cell
        .advance_exact::<(), (), _>(
            &admission,
            &initial,
            &mut (),
            &cancellation.token(),
            |transaction| {
                transaction.set_dependencies(dispatch, [DependencyEdge::new(berth, Aspect::new(0))])
            },
        )
        .expect("the first semantic state performs");
    let (changed_observation, _) = changed.into_parts();
    let changed_basis = admit_runtime_signal_branch_observation(
        changed_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the changed basis retains its branch"),
    );
    let capture_a = cell
        .capture_snapshot_exact(
            &changed_basis,
            owner
                .metadata
                .reserve_snapshot(&admission, &cell)
                .expect("snapshot A reserves"),
            &cancellation.token(),
        )
        .expect("snapshot A captures the berth dependency");
    assert_eq!(
        capture_a
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![berth])
    );
    let snapshot_a_id = capture_a.snapshot().meta.snapshot_id;
    let (capture_a_snapshot, capture_a_observation) = capture_a.into_parts();
    let basis_a = admit_runtime_signal_branch_observation(
        capture_a_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("snapshot A basis retains its branch"),
    );
    let admitted_a = AdmittedSignalBranchSnapshot::owner_issued(
        owner.runtime_instance_id(),
        capture_a_snapshot,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("snapshot A authority retains its branch"),
    );

    let reverted = cell
        .advance_exact::<(), (), _>(
            &admission,
            &basis_a,
            &mut (),
            &cancellation.token(),
            |transaction| {
                transaction
                    .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
            },
        )
        .expect("the second semantic state performs");
    let (reverted_observation, _) = reverted.into_parts();
    let reverted_basis = admit_runtime_signal_branch_observation(
        reverted_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the reverted basis retains its branch"),
    );
    let capture_b = cell
        .capture_snapshot_exact(
            &reverted_basis,
            owner
                .metadata
                .reserve_snapshot(&admission, &cell)
                .expect("snapshot B reserves"),
            &cancellation.token(),
        )
        .expect("snapshot B captures the weather dependency");
    assert_eq!(
        capture_b
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![weather])
    );
    let (capture_b_snapshot, capture_b_observation) = capture_b.into_parts();
    let snapshot_b_id = capture_b_snapshot.meta.snapshot_id;
    let basis_b = admit_runtime_signal_branch_observation(
        capture_b_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("snapshot B basis retains its branch"),
    );
    let admitted_b = AdmittedSignalBranchSnapshot::owner_issued(
        owner.runtime_instance_id(),
        capture_b_snapshot,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("snapshot B authority retains its branch"),
    );

    let state_a = owner
        .metadata
        .snapshot_state(&admission, &admitted_a)
        .expect("snapshot A lookup is owner-admitted")
        .expect("snapshot A remains installed");
    let restored_a = cell
        .restore_exact(
            &admission,
            &basis_b,
            &admitted_a,
            state_a,
            &cancellation.token(),
        )
        .expect("snapshot A restores after snapshot B");
    let restored_a_basis = admit_runtime_signal_branch_observation(
        restored_a.into_observation(),
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the restored A basis retains its branch"),
    );
    let changed_again = cell
        .advance_exact::<(), (), _>(
            &admission,
            &restored_a_basis,
            &mut (),
            &cancellation.token(),
            |transaction| {
                transaction.set_dependencies(dispatch, [DependencyEdge::new(depot, Aspect::new(0))])
            },
        )
        .expect("the third semantic state performs after restore");
    let (changed_again_observation, _) = changed_again.into_parts();
    let changed_again_basis = admit_runtime_signal_branch_observation(
        changed_again_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the third basis retains its branch"),
    );
    let capture_c = cell
        .capture_snapshot_exact(
            &changed_again_basis,
            owner
                .metadata
                .reserve_snapshot(&admission, &cell)
                .expect("snapshot C reserves after restore"),
            &cancellation.token(),
        )
        .expect("snapshot C captures restored A contents");
    let snapshot_c_id = capture_c.snapshot().meta.snapshot_id;
    assert_eq!(
        capture_c
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![depot])
    );
    assert_ne!(snapshot_c_id, snapshot_a_id);
    assert_ne!(snapshot_c_id, snapshot_b_id);

    let (_, capture_c_observation) = capture_c.into_parts();
    let basis_c = admit_runtime_signal_branch_observation(
        capture_c_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("snapshot C basis retains its branch"),
    );
    let state_b = owner
        .metadata
        .snapshot_state(&admission, &admitted_b)
        .expect("snapshot B lookup is owner-admitted")
        .expect("snapshot B was not overwritten by capture C");
    cell.restore_exact(
        &admission,
        &basis_c,
        &admitted_b,
        state_b,
        &cancellation.token(),
    )
    .expect("the old snapshot B still restores its real contents");
    cell.with_state(&admission, |state, _| {
        assert_eq!(
            state.state().mutation_ledger().baseline_snapshot_id,
            Some(snapshot_b_id),
            "snapshot B's stored metadata survives capture C"
        );
        assert_eq!(
            state.state().graph().dependency_sources_of(dispatch),
            Ok(vec![weather])
        );
    })
    .expect("the restored B cell remains healthy");
    let state_a_after_c = owner
        .metadata
        .snapshot_state(&admission, &admitted_a)
        .expect("snapshot A lookup remains owner-admitted")
        .expect("the original snapshot key remains installed after recapture");
    let current = cell
        .with_state(&admission, |state, _| state.observation())
        .expect("the restored B cell remains inspectable")
        .expect("the restored B state remains observable");
    let current_basis = admit_runtime_signal_branch_observation(
        current,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the current B basis retains its branch"),
    );
    cell.restore_exact(
        &admission,
        &current_basis,
        &admitted_a,
        state_a_after_c,
        &cancellation.token(),
    )
    .expect("snapshot A still restores its original distinct contents after C");
    cell.with_state(&admission, |state, _| {
        assert_eq!(
            state.state().mutation_ledger().baseline_snapshot_id,
            Some(snapshot_a_id),
            "snapshot A's stored metadata key cannot be replaced by snapshot C"
        );
        assert_eq!(
            state.state().graph().dependency_sources_of(dispatch),
            Ok(vec![berth])
        );
    })
    .expect("the restored A cell remains healthy");
}
