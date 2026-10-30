use worth_signal::facade::{ResourceDescriptorId, ResourceRuntimeSummary};

fn main() {
    let _ = ResourceRuntimeSummary {
        descriptor_count: 0,
        declared_resource_node_count: 0,
        in_flight_request_count: 0,
        retained_lifecycle_history_count: 0,
        active_in_flight_node_count: 0,
        denied_completion_count: 0,
        next_descriptor_id: ResourceDescriptorId::new(0),
    };
}
