use worth_signal::facade::core::{
    AsyncNodeCapabilityAliasLoweringProof, ResourcePayloadContractDigest, ResourcePolicyDigest,
};
use worth_signal::facade::NodeId;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _proof = AsyncNodeCapabilityAliasLoweringProof {
        node: NodeId::new(1, 0),
        capability_registry_digest: fake::<ResourcePolicyDigest>(),
        legacy_registry_digest: fake::<ResourcePolicyDigest>(),
        capability_bundle_digest: fake::<ResourcePolicyDigest>(),
        legacy_bundle_digest: fake::<ResourcePolicyDigest>(),
        capability_payload_contract_digest: fake::<ResourcePayloadContractDigest>(),
        legacy_payload_contract_digest: fake::<ResourcePayloadContractDigest>(),
        compared_width: 3,
    };
}
