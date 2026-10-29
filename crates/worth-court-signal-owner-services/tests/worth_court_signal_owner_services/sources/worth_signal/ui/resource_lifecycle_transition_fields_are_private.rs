use worth_signal::facade::{
    ResourceLifecycleClass, ResourceLifecycleOrdinal, ResourceLifecycleTransition,
    ResourceLifecycleTransitionKind, ResourceNodeId, ResourceOutputContinuity,
};

fn main() {
    let _ = ResourceLifecycleTransition {
        node: ResourceNodeId::from_node(worth_signal::facade::NodeId::new(0, 0)),
        from: ResourceLifecycleClass::Unrequested,
        to: ResourceLifecycleClass::Pending,
        kind: ResourceLifecycleTransitionKind::RequestAdmitted,
        ordinal: ResourceLifecycleOrdinal::ZERO,
        output_continuity: ResourceOutputContinuity::NoPriorOutput,
    };
}
