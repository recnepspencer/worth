use worth_signal::facade::{
    RejectedResourceRequest, ResourceLifecycleTransition, ResourceNodeId, ResourceRejectionOrdinal,
    ResourceRejectionReason, ResourceRequestHandle,
};

fn forged_handle() -> ResourceRequestHandle {
    loop {}
}

fn forged_node() -> ResourceNodeId {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn main() {
    let _ = RejectedResourceRequest {
        handle: forged_handle(),
        node: forged_node(),
        rejection_ordinal: ResourceRejectionOrdinal::new(0),
        reason: ResourceRejectionReason::SemanticFailure,
        lifecycle_transition: forged_transition(),
    };
}
