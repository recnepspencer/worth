use worth_signal::facade::{SignalAspectLoweringOwner, SignalGraph, SignalRuntime};

fn main() {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let source_owner = worth_proof::ConditionalSourceObservationOwner::fresh();
    let _ordinary = runtime.owner_component_services().unwrap();
    let _conditional = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
}
