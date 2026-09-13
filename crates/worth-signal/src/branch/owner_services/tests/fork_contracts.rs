use std::sync::mpsc;
use std::thread;

use crate::branch::{validate_signal_branch_name, SignalBranchForkOperationDenial};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::node::NodeState;

use super::super::SignalOwnerCancellationSource;
use super::fork_sharing::seed_nonempty_persistent_branch_state;
use super::progress_bound::{wait_until_progress, worker_park, PROGRESS_BOUND};
use super::runtime_root::{runtime_with_two_branches, runtime_with_two_branches_from_graph};

#[test]
fn exact_fork_shares_graph_roots_and_isolates_touched_node_state() {
    let mut graph = SignalGraph::new();
    let nodes = (0..128).map(|_| graph.create_node()).collect::<Vec<_>>();
    for pair in nodes.windows(2) {
        graph
            .set_dependencies(pair[1], [DependencyEdge::new(pair[0], Aspect::new(0))])
            .expect("nontrivial source topology installs");
    }
    let (mut runtime, source_branch, _, _) = runtime_with_two_branches_from_graph(graph);
    seed_nonempty_persistent_branch_state(&mut runtime, nodes[0]);
    let source_basis = runtime
        .observe_signal_branch_basis(source_branch.clone())
        .expect("seeded source basis observes");
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("fork operation admits");
    let source_cell = owner
        .lookup_cell(&admission, source_branch.id)
        .expect("source cell is live");
    source_cell
        .with_state(&admission, |state, _| {
            state
                .state_mut()
                .graph_mut()
                .set_node_state(nodes[64], NodeState::Clean)
                .expect("isolation probe starts from a known clean source node");
            state
                .state_mut()
                .mutation_ledger_mut()
                .clear_all(Some(crate::state::SignalSnapshotId(701)));
        })
        .expect("source receives a mutation-sensitive pre-fork journal");
    let source_ledger = source_cell
        .with_state(&admission, |state, _| {
            state.state().mutation_ledger().clone()
        })
        .expect("source journal is inspectable before fork");
    let (source_branch_before, source_catalog_before) = source_cell
        .with_state(&admission, |source, _| {
            (
                source.state().graph().current_branch(),
                source.state().graph().known_branches(),
            )
        })
        .expect("source diagnostics are inspectable before fork");
    let original_children = owner
        .metadata
        .branch_children(&admission, source_branch.id)
        .expect("source lineage is inspectable before fork");
    let cancellation = SignalOwnerCancellationSource::new();
    let before = owner.cost_snapshot();
    let ready = owner
        .reserve_fork_output(&admission, &source_cell)
        .expect("fork output retention reserves")
        .fork(
            &source_basis,
            validate_signal_branch_name("phase-3-exact-fork").expect("identity validates"),
            &cancellation.token(),
        )
        .expect("destination installs after source capture releases its cell");
    let (destination, destination_basis) = ready.into_destination_parts();
    let installed = owner
        .lookup_cell(&admission, destination.id)
        .expect("the handed-off destination is canonically installed");

    installed
        .with_state(&admission, |state, _| {
            assert_eq!(state.branch_id(), destination.id);
            assert_eq!(state.state().branch_id(), destination.id);
            assert_eq!(state.state().graph().current_branch().id, destination.id);
            assert_eq!(state.state().graph().active_node_count(), 129);
        })
        .expect("installed destination is immediately complete");
    let destination_admission = owner.admit().expect("destination inspection admits");
    let source_identity = source_cell
        .with_state(&admission, |source, _| source.state().persistent_identity())
        .expect("source identity capture admits");
    let destination_identity = installed
        .with_state(&destination_admission, |destination, _| {
            destination.state().persistent_identity()
        })
        .expect("destination identity capture admits");
    let (source_branch_after, source_catalog_after) = source_cell
        .with_state(&admission, |source, _| {
            (
                source.state().graph().current_branch(),
                source.state().graph().known_branches(),
            )
        })
        .expect("source diagnostics remain inspectable after fork");
    assert_eq!(source_branch_after, source_branch_before);
    assert_eq!(source_catalog_after, source_catalog_before);
    let destination_catalog = installed
        .with_state(&destination_admission, |destination, _| {
            destination.state().graph().known_branches()
        })
        .expect("destination diagnostics are independently inspectable");
    assert_eq!(destination_catalog, vec![destination.clone()]);
    let sharing = source_identity.sharing_with(&destination_identity);
    assert!(sharing.graph.arena_root_shared);
    assert!(sharing.graph.topology_root_shared);
    assert!(sharing.graph.cause_root_shared);
    assert!(sharing.graph.schema_root_shared);
    assert!(sharing.graph.observation_root_shared);
    assert!(sharing.graph.keyed_roots_shared);
    assert!(sharing.config_roots_shared);
    assert!(sharing.derived_roots_shared);

    installed
        .with_state(&destination_admission, |destination, _| {
            destination
                .state_mut()
                .graph_mut()
                .set_node_state(nodes[64], NodeState::Dirty)
                .expect("destination mutation is valid");
        })
        .expect("destination mutation admits");
    source_cell
        .with_state(&admission, |source, _| {
            assert_eq!(
                source.state().graph().get_state(nodes[64]),
                Ok(NodeState::Clean)
            );
        })
        .expect("source remains independently admitted");
    let destination_after_mutation = installed
        .with_state(&destination_admission, |destination, _| {
            assert_eq!(
                destination.state().graph().get_state(nodes[64]),
                Ok(NodeState::Dirty)
            );
            destination.state().persistent_identity()
        })
        .expect("destination remains independently admitted");
    let sharing_after_mutation = source_identity.sharing_with(&destination_after_mutation);
    assert!(!sharing_after_mutation.graph.arena_root_shared);
    assert!(sharing_after_mutation.graph.topology_root_shared);
    assert!(sharing_after_mutation.graph.cause_root_shared);
    assert!(sharing_after_mutation.config_roots_shared);
    assert!(sharing_after_mutation.derived_roots_shared);
    assert_ne!(
        source_identity.hot_page_identities[1], destination_after_mutation.hot_page_identities[1],
        "the touched hot page must detach"
    );
    assert_eq!(
        source_identity.hot_page_identities[0], destination_after_mutation.hot_page_identities[0],
        "a distant hot page must retain allocation identity"
    );
    let after = owner.cost_snapshot();
    assert_eq!(
        after.fork_source_captures(),
        before.fork_source_captures() + 1
    );
    assert_eq!(
        after.fork_destination_preparations(),
        before.fork_destination_preparations() + 1
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations() + 1
    );
    assert_eq!(
        after.forked_mutable_graph_nodes_copied(),
        before.forked_mutable_graph_nodes_copied(),
        "a lawful persistent fork copies no mutable graph node"
    );
    assert_eq!(owner.live_count(), 3);
    assert_eq!(owner.reservation_count(), 0);
    let observed_source_ledger = source_cell
        .with_state(&admission, |state, _| {
            state.state().mutation_ledger().clone()
        })
        .expect("successful fork source remains inspectable");
    let mut expected_source_ledger = source_ledger;
    expected_source_ledger.clear_all(source_branch.head_snapshot_id);
    assert_eq!(observed_source_ledger, expected_source_ledger);
    let children = owner
        .metadata
        .branch_children(&admission, source_branch.id)
        .expect("successful fork lineage is committed");
    let mut expected_children = original_children;
    expected_children.push(destination.id);
    expected_children.sort_unstable();
    assert_eq!(children, expected_children);
    drop(destination_basis);
}

#[test]
fn late_fork_cancellation_drops_preconstructed_destination_without_source_movement() {
    let (mut runtime, sibling_branch, source_branch, source_basis) = runtime_with_two_branches();
    let sibling_basis = runtime
        .observe_signal_branch_basis(sibling_branch.clone())
        .expect("the unrelated branch admits before sealing");
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let setup = owner.admit().expect("setup admits");
    let source_cell = owner
        .lookup_cell(&setup, source_branch.id)
        .expect("source cell is live");
    source_cell
        .with_state(&setup, |state, _| {
            state
                .state_mut()
                .mutation_ledger_mut()
                .clear_all(Some(crate::state::SignalSnapshotId(701)));
        })
        .expect("source receives a mutation-sensitive baseline");
    let source_ledger = source_cell
        .with_state(&setup, |state, _| state.state().mutation_ledger().clone())
        .expect("source ledger is inspectable");
    let original_children = owner
        .metadata
        .branch_children(&setup, source_branch.id)
        .expect("lineage inspection is owner-admitted");
    drop(setup);

    let cancellation = SignalOwnerCancellationSource::new();
    let before = owner.cost_snapshot();
    let waits_before = source_cell.cost_snapshot().waits();
    let (holder_tx, holder_rx) = mpsc::sync_channel(1);
    let (fork_tx, fork_rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        let (park, mut control) = worker_park();
        scope.spawn(|| {
            let holder_admission = owner
                .admit()
                .expect("holder admits in its executing thread");
            let result = source_cell.with_state(&holder_admission, |_, _| {
                park.park("late-cancellation source-cell holder");
            });
            let _ = holder_tx.send(result);
        });
        control.wait_until_parked("late-cancellation source-cell holder");
        scope.spawn(|| {
            let fork_admission = owner
                .admit()
                .expect("fork admits independently in its executing thread");
            let reservation = owner
                .reserve_fork_destination(
                    &fork_admission,
                    &source_basis,
                    validate_signal_branch_name("phase-3-late-cancelled-fork")
                        .expect("identity validates"),
                )
                .expect("destination reserves");
            let source_custody = source_cell
                .acquire_fork_source_custody(&fork_admission)
                .expect("fork admission acquires exact source custody");
            let result = reservation
                .capture(source_custody, &source_basis, &cancellation.token())
                .and_then(|prepared| prepared.install())
                .map(|_| ());
            let _ = fork_tx.send(result);
        });
        let reached_wait = wait_until_progress("fork source-cell contention", || {
            source_cell.cost_snapshot().waits() == waits_before + 1
        });
        cancellation.cancel();
        control.release();
        let fork_result = fork_rx.recv_timeout(PROGRESS_BOUND);
        let holder_result = holder_rx.recv_timeout(PROGRESS_BOUND);
        assert!(
            reached_wait,
            "fork did not reach the bounded source-cell wait"
        );
        assert!(
            matches!(
                fork_result,
                Ok(Err(SignalBranchForkOperationDenial::CancelledNoMovement))
            ),
            "cancelled fork did not return the exact no-movement denial"
        );
        assert_eq!(holder_result, Ok(Ok(())));
    });

    let after = owner.cost_snapshot();
    assert_eq!(
        after.fork_destination_preparations(),
        before.fork_destination_preparations(),
        "branch-scoped source custody observes cancellation before destination state preconstruction"
    );
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations() + 1,
        "the already-created exact destination reservation is still unwound"
    );
    assert_eq!(after.fork_source_captures(), before.fork_source_captures());
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations()
    );
    assert_fork_source_and_capacity_unchanged(
        &owner,
        &source_cell,
        source_branch.id,
        &source_ledger,
        &original_children,
    );

    let sibling_admission = owner.admit().expect("sibling progress admits");
    let sibling_cell = owner
        .lookup_cell(&sibling_admission, sibling_branch.id)
        .expect("the unrelated cell remains live");
    let sibling_observation = sibling_cell
        .advance_exact::<(), (), _>(
            &sibling_admission,
            &sibling_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("an unrelated canonical movement follows cancellation cleanup")
        .into_parts()
        .0;
    assert_eq!(
        sibling_cell
            .observe_exact(&sibling_admission)
            .expect("the unrelated performed state remains canonical"),
        sibling_observation
    );

    let healthy_admission = owner.admit().expect("healthy twin admits");
    let (healthy_handle, healthy_basis) = owner
        .reserve_fork_output(&healthy_admission, &source_cell)
        .expect("healthy output custody reserves")
        .fork(
            &source_basis,
            validate_signal_branch_name("phase-3-healthy-fork").expect("identity validates"),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("healthy twin installs immediately after cancellation cleanup")
        .into_destination_parts();
    assert_eq!(healthy_basis.owner_branch_id(), healthy_handle.id);
    assert_eq!(owner.live_count(), 3);
}

fn assert_fork_source_and_capacity_unchanged(
    owner: &std::sync::Arc<super::super::SignalOwner<(), (), ()>>,
    source_cell: &std::sync::Arc<
        super::super::SignalBranchExecutionCell<super::super::SignalBranchCellState<(), (), ()>>,
    >,
    source_branch_id: crate::state::SignalBranchId,
    source_ledger: &crate::logic::transaction::BranchMutationLedger,
    original_children: &[crate::state::SignalBranchId],
) {
    let admission = owner.admit().expect("cleanup inspection admits");
    let observed_ledger = source_cell
        .with_state(&admission, |state, _| {
            state.state().mutation_ledger().clone()
        })
        .expect("source remains inspectable");
    assert_eq!(&observed_ledger, source_ledger);
    assert_eq!(owner.live_count(), 2);
    assert_eq!(owner.reservation_count(), 0);
    assert_eq!(
        owner
            .metadata
            .branch_children(&admission, source_branch_id)
            .expect("lineage cleanup is owner-admitted"),
        original_children
    );
}
