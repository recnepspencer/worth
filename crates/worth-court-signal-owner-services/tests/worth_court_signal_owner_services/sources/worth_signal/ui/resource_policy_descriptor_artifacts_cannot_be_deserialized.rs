use worth_signal::facade::core::{
    DeniedResourcePolicyRestoreCompatibility, FrozenResourcePolicyDescriptor,
    FrozenResourcePolicyDescriptorSet, LoweredResourcePolicyBundle,
    ResourcePolicyCompatibilityFamilyReport, ResourcePolicyCompatibilityReport,
    ResourcePolicyDescriptor, ResourcePolicyRestoreCompatibilityProof, ResourceResolvedPolicy,
    ValidatedResourcePolicyDeclaration, ValidatedResourcePolicyReference,
};

fn requires_deserialize_owned<T: serde::de::DeserializeOwned>() {}

fn main() {
    requires_deserialize_owned::<ResourcePolicyDescriptor>();
    requires_deserialize_owned::<ResourceResolvedPolicy>();
    requires_deserialize_owned::<ValidatedResourcePolicyReference>();
    requires_deserialize_owned::<ValidatedResourcePolicyDeclaration>();
    requires_deserialize_owned::<FrozenResourcePolicyDescriptor>();
    requires_deserialize_owned::<FrozenResourcePolicyDescriptorSet>();
    requires_deserialize_owned::<ResourcePolicyCompatibilityFamilyReport>();
    requires_deserialize_owned::<ResourcePolicyCompatibilityReport>();
    requires_deserialize_owned::<ResourcePolicyRestoreCompatibilityProof>();
    requires_deserialize_owned::<DeniedResourcePolicyRestoreCompatibility>();
    requires_deserialize_owned::<LoweredResourcePolicyBundle>();
}
