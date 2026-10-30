use worth_signal::facade::core::{
    LoweredResourcePolicyBundle, ResourceNodeDeclaration, ResourceNodeId, ResourcePayloadContract,
    ResourcePayloadContractId, SignalGraph,
};
use worth_signal::facade::runtime::SignalRuntime;

fn main() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let declaration = ResourceNodeDeclaration::new(
        ResourceNodeId::from_node(node),
        ResourcePayloadContract::new(ResourcePayloadContractId::new(7)),
    );
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build(graph);
    runtime
        .declare_resource_node(declaration)
        .expect("resource declaration should lower");
    let descriptor = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(node))
        .expect("lowered descriptor should exist");
    let lowered = descriptor.lowered_policy_bundle().clone();

    let _forged = LoweredResourcePolicyBundle {
        retry: lowered.retry().clone(),
        timeout: lowered.timeout().clone(),
        cancellation: lowered.cancellation().clone(),
        stale_after: lowered.stale_after().clone(),
        supersession: lowered.supersession().clone(),
        revalidation: lowered.revalidation().clone(),
        observation: lowered.observation().clone(),
        output_continuity: lowered.output_continuity().clone(),
        retention: lowered.retention().clone(),
        diagnostics: lowered.diagnostics().clone(),
        registry_digest: lowered.registry_digest().clone(),
        bundle_digest: lowered.bundle_digest().clone(),
    };
}
