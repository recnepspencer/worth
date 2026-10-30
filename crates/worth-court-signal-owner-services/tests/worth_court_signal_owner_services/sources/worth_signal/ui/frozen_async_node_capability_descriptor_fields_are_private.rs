use worth_signal::facade::core::{
    FrozenAsyncNodeCapabilityDescriptor, FrozenResourcePolicyDescriptorSet,
    ResourcePayloadContractDigest,
};
use worth_signal::facade::NodeId;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _frozen = FrozenAsyncNodeCapabilityDescriptor {
        node: NodeId::new(1, 0),
        payload_contract_digest: fake::<ResourcePayloadContractDigest>(),
        frozen: fake::<FrozenResourcePolicyDescriptorSet>(),
    };
}
