use std::sync::mpsc;
use std::thread;

use crate::branch::owner_services::{
    SignalOwnerAdmissionDenial, SignalOwnerCancellationSource, SignalOwnerLifecycleObservation,
};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;

use super::progress_bound::PROGRESS_BOUND;
use super::runtime_root::runtime_with_two_branches_from_graph;

#[test]
fn root_drop_inside_admitted_callback_requests_close_without_self_deadlock() {
    let mut graph = SignalGraph::new();
    let source_a = graph.create_node();
    let source_b = graph.create_node();
    let derived = graph.create_node();
    graph
        .set_dependencies(derived, [DependencyEdge::new(source_a, Aspect::new(0))])
        .expect("effectful callback fixture installs source A");
    let (mut runtime, _, branch, basis) = runtime_with_two_branches_from_graph(graph);
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let (done_tx, done_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let admission = owner.admit().expect("canonical callback admits");
        let cell = owner
            .lookup_cell(&admission, branch.id)
            .expect("the canonical target cell is installed");
        let cancellation = SignalOwnerCancellationSource::new();
        let outcome = cell.advance_exact::<(), (), _>(
            &admission,
            &basis,
            &mut (),
            &cancellation.token(),
            move |transaction| {
                drop(runtime);
                transaction
                    .set_dependencies(derived, [DependencyEdge::new(source_b, Aspect::new(0))])
            },
        );
        let (observation, transaction) = outcome
            .expect("root destruction does not erase performed callback work")
            .into_parts();
        let canonical = cell
            .observe_exact(&admission)
            .expect("already-admitted work can observe the performed canonical state");
        let dependency_sources = cell
            .with_state(&admission, |state, _| {
                state.state().graph().dependency_sources_of(derived)
            })
            .expect("already-admitted work can inspect the populated performed state");
        let closing = owner.lifecycle_observation();
        let late_denial = owner
            .admit()
            .expect_err("a close request rejects every later admission");
        let movements = cell.cost_snapshot().movements();
        drop(admission);
        let closed = owner.lifecycle_observation();
        let _ = done_tx.send((
            observation,
            canonical,
            transaction.touched_nodes,
            dependency_sources,
            movements,
            closing,
            late_denial,
            closed,
        ));
    });

    let (
        observation,
        canonical,
        touched_nodes,
        dependency_sources,
        movements,
        closing,
        late_denial,
        closed,
    ) = done_rx
        .recv_timeout(PROGRESS_BOUND)
        .expect("root destruction must not wait on its own callback admission");
    assert_eq!(canonical, observation);
    assert_eq!(canonical.generation().get(), 1);
    assert!(touched_nodes > 0);
    assert_eq!(dependency_sources, Ok(vec![source_b]));
    assert_eq!(movements, 1);
    assert_eq!(closing, SignalOwnerLifecycleObservation::Closing);
    assert_eq!(late_denial, SignalOwnerAdmissionDenial::OwnerUnavailable);
    assert_eq!(closed, SignalOwnerLifecycleObservation::Closed);
}
