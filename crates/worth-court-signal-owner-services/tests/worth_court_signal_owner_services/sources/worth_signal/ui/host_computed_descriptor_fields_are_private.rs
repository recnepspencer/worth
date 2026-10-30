use worth_signal::facade::{HostComputedApiFamily, HostComputedDescriptor, HostComputedDescriptorId, NodeId};

fn main() {
    let _ = HostComputedDescriptor {
        descriptor_id: HostComputedDescriptorId::new(1),
        node: NodeId::new(1, 0),
        api_family: HostComputedApiFamily::EasyClosure,
    };
}
