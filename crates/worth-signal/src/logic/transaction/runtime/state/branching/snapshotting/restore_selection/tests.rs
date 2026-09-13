use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

#[test]
fn restore_selection_validation_precedes_active_and_stored_movement() {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let derived = graph.create_node();
    graph
        .set_dependencies(derived, [DependencyEdge::new(source, Aspect::new(0))])
        .expect("populated restore boundary fixture installs");
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let active_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the active source basis observes");
    let destination = runtime
        .fork_signal_branch("stored-restore-destination", &active_basis)
        .expect("the real stored restore destination forks")
        .created_branch()
        .clone();
    let active_before = runtime.current_branch();
    let mismatched_active_state = runtime
        .capture_heavy_branch_state()
        .expect("non-moving capture prepares a mismatched restored state");

    let denial =
        runtime.install_snapshot_restore_selection(destination.id, mismatched_active_state);
    assert!(denial.is_err());
    assert_eq!(runtime.current_branch(), active_before);
    assert_eq!(
        runtime.graph().dependency_sources_of(derived),
        Ok(vec![source]),
        "validation denial cannot replace the active graph with Default"
    );
    let stored = runtime
        .branches
        .branch_state(destination.id)
        .expect("validation denial cannot remove the stored restore target");
    assert_eq!(
        stored.graph().dependency_sources_of(derived),
        Ok(vec![source]),
        "validation denial preserves the exact stored target state"
    );
}
