use crate::branch::SignalBranchSnapshotReconstructionDenial;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::transaction::SignalRuntime;

use super::super::super::SignalOwnerCancellationSource;

fn populated_graph() -> (SignalGraph, NodeId, NodeId) {
    let mut graph = SignalGraph::new();
    let weather = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
        .expect("the populated snapshot world installs");
    (graph, weather, dispatch)
}

#[test]
fn unsealed_sibling_captures_retain_the_inherited_global_identity_contract() {
    let (graph, _, _) = populated_graph();
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the unsealed runtime admits its starting branch");
    let (_, first_basis) = runtime
        .fork_signal_branch("unsealed-first", &initial)
        .expect("the first sibling forks")
        .into_parts();
    let (_, second_basis) = runtime
        .fork_signal_branch("unsealed-second", &initial)
        .expect("the second sibling forks")
        .into_parts();

    let first = runtime
        .capture_signal_branch_snapshot(&first_basis)
        .expect("the first unsealed sibling captures");
    let second = runtime
        .capture_signal_branch_snapshot(&second_basis)
        .expect("the second unsealed sibling captures");

    assert_ne!(
        first.snapshot().meta.snapshot_id,
        second.snapshot().meta.snapshot_id,
        "the inherited unsealed owner already issues runtime-global identities"
    );
}

#[test]
fn capture_fork_then_owner_capture_advances_past_the_inherited_identity() {
    let (graph, weather, dispatch) = populated_graph();
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the unsealed runtime admits its starting branch");
    let first = runtime
        .capture_signal_branch_snapshot(&initial)
        .expect("the pre-seal capture populates real snapshot storage");
    let first_id = first.snapshot().meta.snapshot_id;
    let (child, child_basis) = runtime
        .fork_signal_branch("post-capture-child", first.captured_basis())
        .expect("the child inherits the captured high-water state")
        .into_parts();
    let child_id = child.id;

    let (_, mutation, _) = runtime
        .owner_port_slots()
        .expect("the populated runtime seals");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("the owner admits capture");
    let cell = owner
        .lookup_cell(&admission, child_id)
        .expect("the inherited child cell is installed");
    let capture = cell
        .capture_snapshot_exact(
            &child_basis,
            owner
                .metadata
                .reserve_snapshot(&admission, &cell)
                .expect("the transferred allocator reserves"),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the owner captures after the pre-seal fork");

    assert!(
        capture.snapshot().meta.snapshot_id > first_id,
        "sealing must transfer the populated pre-seal high-water mark"
    );
    assert_eq!(
        capture
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![weather]),
        "the populated child state survives capture, fork, and direct sealing"
    );
}

#[test]
fn preseal_restore_cannot_rewind_the_allocator_transferred_to_the_owner() {
    let (graph, weather, dispatch) = populated_graph();
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let branch = runtime.current_branch();
    let initial = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("the unsealed runtime admits its starting branch");
    let first = runtime
        .capture_signal_branch_snapshot(&initial)
        .expect("the first historical state captures");
    let second = runtime
        .capture_signal_branch_snapshot(first.captured_basis())
        .expect("the second historical state captures");
    let second_id = second.snapshot().meta.snapshot_id;
    let restored = runtime
        .restore_signal_branch(second.captured_basis(), first.admitted_snapshot())
        .expect("the unsealed runtime restores the older historical state");

    let (_, mutation, _) = runtime
        .owner_port_slots()
        .expect("the restored runtime seals");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("the owner admits capture");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the restored cell is installed");
    let capture = cell
        .capture_snapshot_exact(
            &restored,
            owner
                .metadata
                .reserve_snapshot(&admission, &cell)
                .expect("the historical high-water reserves"),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the owner captures after historical restoration");

    assert!(
        capture.snapshot().meta.snapshot_id > second_id,
        "restored diagnostic history must not rewind owner-global identity"
    );
    assert_eq!(
        capture
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![weather]),
        "the populated state survives capture, historical restore, and direct sealing"
    );
}

#[test]
fn reconstructed_populated_snapshot_hands_off_one_active_state_and_fresh_high_water() {
    let (source_graph, weather, dispatch) = populated_graph();
    let mut source = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(source_graph);
    let source_basis = source
        .observe_signal_branch_basis(source.current_branch())
        .expect("the source admits its populated root");
    let portable = source
        .capture_signal_branch_snapshot(&source_basis)
        .expect("the source captures a portable populated snapshot")
        .snapshot()
        .clone();
    let imported_id = portable.meta.snapshot_id;

    let mut target = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(SignalGraph::new());
    let target_branch = target.current_branch();
    let pristine = target
        .observe_signal_branch_basis(target_branch.clone())
        .expect("the target admits its pristine root");
    let (_, reconstructed_basis) = target
        .reconstruct_signal_branch_snapshot(&pristine, &portable)
        .expect("the populated snapshot reconstructs into the pristine owner")
        .into_parts();
    let (_, mutation, _) = target
        .owner_port_slots()
        .expect("reconstruction hands off one canonical active state directly");
    let owner = mutation
        .upgrade_owner()
        .expect("the target owner remains live");
    let admission = owner.admit().expect("the target admits owner work");
    let cell = owner
        .lookup_cell(&admission, target_branch.id)
        .expect("the reconstructed active cell is installed");
    cell.with_state(&admission, |state, _| {
        assert_eq!(
            state.state().graph().dependency_sources_of(dispatch),
            Ok(vec![weather])
        );
    })
    .expect("the imported dependency state remains inspectable");
    let capture = cell
        .capture_snapshot_exact(
            &reconstructed_basis,
            owner
                .metadata
                .reserve_snapshot(&admission, &cell)
                .expect("the imported high-water reserves a fresh identity"),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the reconstructed cell captures");
    assert!(capture.snapshot().meta.snapshot_id > imported_id);
}

#[test]
fn reconstruction_rejects_a_branch_pristine_root_after_owner_identity_use() {
    let (source_graph, _, _) = populated_graph();
    let mut source = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(source_graph);
    let source_basis = source
        .observe_signal_branch_basis(source.current_branch())
        .expect("the source admits its root");
    let portable = source
        .capture_signal_branch_snapshot(&source_basis)
        .expect("the source captures portable identity zero")
        .snapshot()
        .clone();

    let (target_graph, _, _) = populated_graph();
    let mut target = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(target_graph);
    let root = target.current_branch();
    let root_basis = target
        .observe_signal_branch_basis(root.clone())
        .expect("the target admits its root");
    let (_, sibling_basis) = target
        .fork_signal_branch("reconstruction-sibling", &root_basis)
        .expect("the target creates a sibling before import")
        .into_parts();
    let sibling = target
        .capture_signal_branch_snapshot(&sibling_basis)
        .expect("the sibling consumes target-owner identity zero");
    assert_eq!(
        sibling.snapshot().meta.snapshot_id,
        portable.meta.snapshot_id
    );
    let current_root = target
        .observe_signal_branch_basis(root.clone())
        .expect("the root remains branch-pristine after sibling capture");
    let before = current_root.observation().clone();

    assert!(matches!(
        target.reconstruct_signal_branch_snapshot(&current_root, &portable),
        Err(SignalBranchSnapshotReconstructionDenial::NonPristineRuntime)
    ));
    let after = target
        .observe_signal_branch_basis(root)
        .expect("the denied import leaves the root observable");
    assert!(after.observation().compare(&before).is_ok());

    let (mut used_graph, _, _) = populated_graph();
    used_graph
        .diagnostics_state_mut()
        .synchronize_branch_snapshot_allocator(1, 1);
    let mut used_target = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(used_graph);
    let used_root = used_target.current_branch();
    let used_basis = used_target
        .observe_signal_branch_basis(used_root.clone())
        .expect("the high-water target admits its otherwise pristine root");
    assert!(matches!(
        used_target.reconstruct_signal_branch_snapshot(&used_basis, &portable),
        Err(SignalBranchSnapshotReconstructionDenial::NonPristineRuntime)
    ));
}

#[test]
fn sibling_capture_then_merge_uses_the_runtime_global_snapshot_identity() {
    let (graph, _, _) = populated_graph();
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the merge world admits its root");
    let (_, source_basis) = runtime
        .fork_signal_branch("merge-source", &initial)
        .expect("the source sibling forks")
        .into_parts();
    let (_, target_basis) = runtime
        .fork_signal_branch("merge-target", &initial)
        .expect("the target sibling forks")
        .into_parts();
    let source_capture = runtime
        .capture_signal_branch_snapshot(&source_basis)
        .expect("the source sibling advances the global identity high-water");
    let source_id = source_capture.snapshot().meta.snapshot_id;
    let merged = runtime
        .merge_branch(source_capture.captured_basis(), &target_basis)
        .expect("the lawful sibling merge completes");
    let target_id = merged
        .result()
        .target_snapshot_id_after
        .expect("merge finalization captures the target state");

    assert!(
        target_id > source_id,
        "merge artifact finalization must advance the runtime-global allocator"
    );
}
