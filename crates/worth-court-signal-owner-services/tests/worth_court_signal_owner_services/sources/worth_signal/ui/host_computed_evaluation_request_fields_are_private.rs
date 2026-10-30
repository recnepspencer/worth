use worth_signal::facade::{
    DependencyEdge, HostComputedApiFamily, HostComputedDescriptor, HostComputedDescriptorId,
    HostComputedEvaluationRequest, NodeId,
};

fn main() {
    let _ = HostComputedEvaluationRequest {
        descriptor: HostComputedDescriptor {
            descriptor_id: HostComputedDescriptorId::new(1),
            node: NodeId::new(1, 0),
            api_family: HostComputedApiFamily::EasyClosure,
        },
        previous_dependencies: vec![DependencyEdge::new(NodeId::new(2, 0), worth_signal::facade::Aspect::new(0))],
    };
}
