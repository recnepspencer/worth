use worth_signal::facade::core::{
    ResourceCostContractId, ResourcePolicyCompatibilityPosture, ResourcePolicyDescriptor,
    ResourcePolicyDescriptorId, ResourcePolicyKind, ResourcePolicyName, ResourcePolicyVersion,
};

fn main() {
    let _descriptor = ResourcePolicyDescriptor::new(
        ResourcePolicyDescriptorId::new(1),
        ResourcePolicyKind::Retry,
        ResourcePolicyName::new("external.retry"),
        ResourcePolicyVersion::INITIAL,
        ResourceCostContractId::new(5),
        ResourcePolicyCompatibilityPosture::ExactDescriptorMatch,
    );
}
