use std::sync::{mpsc, Arc};
use std::thread;

use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::super::owner::close_cleanup::SignalOwnerCloseBatchKind;
use super::super::SignalOwnerLifecycleObservation;
use super::progress_bound::PROGRESS_BOUND;
use super::runtime_root::runtime_with_two_branches;

#[test]
fn concurrent_owner_close_has_one_cleanup_owner_and_counts_only_real_batches() {
    let (mut runtime, _, _, _) = runtime_with_two_branches();
    let (basis, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis.upgrade_owner().expect("owner remains live");
    let (done_tx, done_rx) = mpsc::sync_channel(2);
    for _ in 0..2 {
        let closing_owner = Arc::clone(&owner);
        let done_tx = done_tx.clone();
        thread::spawn(move || {
            let _ = done_tx.send(closing_owner.close());
        });
    }
    drop(done_tx);

    assert_eq!(done_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(done_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert_eq!(owner.live_count(), 0);
    assert!(
        owner.cost_snapshot().close_batches() >= 1,
        "owner close counts registry or metadata work batches, never its phase transition"
    );
}

#[test]
fn close_batches_cross_sixty_four_and_detach_populated_cells_before_drop() {
    let mut graph = SignalGraph::new();
    let nodes = (0..96).map(|_| graph.create_node()).collect::<Vec<_>>();
    for pair in nodes.windows(2) {
        graph
            .set_dependencies(pair[1], [DependencyEdge::new(pair[0], Aspect::new(0))])
            .expect("the populated close graph accepts its dependency chain");
    }
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let selected = runtime.current_branch();
    let selected_basis = runtime
        .observe_signal_branch_basis(selected)
        .expect("the selected source admits");
    for index in 0..64 {
        let (_, destination_basis) = runtime
            .fork_signal_branch(format!("close-batch-{index}"), &selected_basis)
            .expect("the populated sibling forks")
            .into_parts();
        drop(destination_basis);
    }
    let (basis, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis.upgrade_owner().expect("owner remains live");
    assert_eq!(owner.live_count(), 65);

    let mut detached = Vec::new();
    owner
        .close_with_cleanup_observer(|kind, cleaned_entries| {
            detached.push((kind, cleaned_entries, owner.live_count()));
        })
        .expect("the populated owner closes");

    assert_eq!(
        detached,
        vec![
            (SignalOwnerCloseBatchKind::Registry, 64, 1),
            (SignalOwnerCloseBatchKind::Registry, 1, 0),
            (SignalOwnerCloseBatchKind::Metadata, 1, 0),
        ],
        "each observer runs after short-lock detachment and before heavy batch destruction"
    );
    assert_eq!(owner.cost_snapshot().close_batches(), 3);
    assert_eq!(owner.live_count(), 0);
}
