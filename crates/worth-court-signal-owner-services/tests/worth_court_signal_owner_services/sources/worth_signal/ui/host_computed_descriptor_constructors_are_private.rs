use worth_signal::facade::{
    HostComputedApiFamily, HostComputedDescriptor, HostComputedDescriptorId, NodeId,
};

fn main() {
    let id = HostComputedDescriptorId::new(1);
    let _ = HostComputedDescriptor::new(id, NodeId::new(1, 0), HostComputedApiFamily::EasyClosure);
}
