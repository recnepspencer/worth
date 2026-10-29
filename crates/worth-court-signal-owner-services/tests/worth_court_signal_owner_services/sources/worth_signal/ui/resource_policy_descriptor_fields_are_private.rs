use worth_signal::facade::core::{
    ResourceCostContractId, ResourcePolicyCompatibilityPosture, ResourcePolicyDescriptor,
    ResourcePolicyDescriptorId, ResourcePolicyDigest, ResourcePolicyKind, ResourcePolicyName,
    ResourcePolicyVersion,
};

fn main() {
    let _descriptor = ResourcePolicyDescriptor {
        id: ResourcePolicyDescriptorId::new(1),
        kind: ResourcePolicyKind::Retry,
        semantic_name: ResourcePolicyName::new("external.retry"),
        version: ResourcePolicyVersion::INITIAL,
        descriptor_digest: ResourcePolicyDigest::new("forged"),
        cost_contract: ResourceCostContractId::new(5),
        compatibility_posture: ResourcePolicyCompatibilityPosture::ExactDescriptorMatch,
    };
}
