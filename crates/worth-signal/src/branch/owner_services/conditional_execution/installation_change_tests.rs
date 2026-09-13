use crate::data::aspect::{Aspect, AspectMask, SignalAspectLoweringOwner};
use crate::data::conditional_execution::{
    InstalledSignalConditionalContract, SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalVersionComparator,
};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use worth_proof::{ConditionalEvaluationSource, ConditionalSourceObservationOwner};

fn definition() -> SignalConditionalContractDefinition {
    SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::Always,
        dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
        trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
    }
}

#[test]
fn retirement_publishes_a_successor_without_rewriting_existing_admissions() {
    let mut graph = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let [surviving, retired] = [(); 2].map(|()| install_contract(&mut graph, &claimant));
    let retired_node = retired.node();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let _ports = runtime.owner_port_slots().unwrap();
    let source = ConditionalSourceObservationOwner::fresh();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source.authority())
        .unwrap();
    let before = service
        .admit_evaluation(
            &surviving,
            ConditionalEvaluationSource::AdmittedRelationalSource(source.admit("before")),
        )
        .unwrap();
    assert!(before
        .retained_basis
        .retained_node_observation(retired_node, Aspect::new(1))
        .is_some());

    service.retire_installed_contract(&retired).unwrap();

    assert!(before
        .retained_basis
        .retained_node_observation(retired_node, Aspect::new(1))
        .is_some());
    let after = service
        .admit_evaluation(
            &surviving,
            ConditionalEvaluationSource::AdmittedRelationalSource(source.admit("after")),
        )
        .unwrap();
    assert!(after
        .retained_basis
        .retained_node_observation(retired_node, Aspect::new(1))
        .is_none());
}

fn install_contract(
    graph: &mut SignalGraph,
    claimant: &SignalAspectLoweringOwner,
) -> InstalledSignalConditionalContract {
    let node = graph.node().build();
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh node must admit")
    };
    graph
        .install_conditional_contract(claimant, capability, definition())
        .unwrap()
}
