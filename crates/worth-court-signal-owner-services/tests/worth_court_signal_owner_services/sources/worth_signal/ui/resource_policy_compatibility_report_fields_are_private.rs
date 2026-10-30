use worth_signal::facade::core::{
    ResourceNodeId, ResourcePolicyCompatibilityFamilyReport, ResourcePolicyCompatibilityReport,
    SignalGraph,
};
use worth_signal::facade::{ResourceBoundaryPerformanceEnvelope, ResourceDescriptorId};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();

    let _report = ResourcePolicyCompatibilityReport {
        descriptor_id: ResourceDescriptorId::new(1),
        node: ResourceNodeId::from_node(node),
        compared_width: 9,
        incompatible_width: 1,
        families: vec![fake::<ResourcePolicyCompatibilityFamilyReport>()],
        compatibility_digest: fake(),
        performance: fake::<ResourceBoundaryPerformanceEnvelope>(),
    };
}
