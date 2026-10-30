use worth_signal::facade::core::{
    LoweredAsyncNodeCapabilityBundle, LoweredResourcePolicyBundle, ResourcePayloadContractDigest,
};
use worth_signal::facade::NodeId;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _bundle = LoweredAsyncNodeCapabilityBundle {
        node: NodeId::new(1, 0),
        payload_contract_digest: fake::<ResourcePayloadContractDigest>(),
        lowered: fake::<LoweredResourcePolicyBundle>(),
    };
}
