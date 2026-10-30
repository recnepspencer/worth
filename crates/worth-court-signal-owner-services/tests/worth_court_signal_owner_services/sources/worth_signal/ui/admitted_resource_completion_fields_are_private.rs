use worth_signal::facade::{
    AdmittedResourceCompletion, ResourceAttemptId, ResourceCompletionOrdinal, ResourceDescriptorId,
    ResourceLifecycleTransition, ResourceNodeId, ResourceRequestHandle,
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
    let _ = AdmittedResourceCompletion {
        handle: forged_handle(),
        attempt: ResourceAttemptId::new(0),
        node: forged_node(),
        descriptor_id: ResourceDescriptorId::new(0),
        completion_ordinal: ResourceCompletionOrdinal::new(0),
        payload_byte_len: 0,
        lifecycle_transition: forged_transition(),
    };
}
