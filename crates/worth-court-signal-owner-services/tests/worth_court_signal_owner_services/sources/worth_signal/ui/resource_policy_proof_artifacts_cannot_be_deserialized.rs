use worth_signal::facade::core::{
    DependencyChangeResourceRevalidationProof, DeniedResourcePolicyRestoreCompatibility,
    FrozenResourcePolicyDescriptor, FrozenResourcePolicyDescriptorSet,
    FulfilledLifecycleResourceRevalidationProof, LoweredResourcePolicyBundle,
    ObserverDemandResourceRevalidationProof, ResourcePolicyCompatibilityFamilyReport,
    ResourcePolicyCompatibilityReport, ResourcePolicyDescriptor,
    ResourcePolicyRestoreCompatibilityProof, ResourceResolvedPolicy,
    ResourceRevalidationCoalescing, ResourceRevalidationDecisionPlan,
    ResourceRevalidationFreshnessDecision, ResourceRetryDecisionPlan,
    ResourceStaleAfterDecisionPlan, ResourceTimeoutDecisionPlan,
    TerminalStateResourceRevalidationProof, ValidatedResourcePolicyDeclaration,
    ValidatedResourcePolicyReference,
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
    requires_deserialize_owned::<ResourceRetryDecisionPlan>();
    requires_deserialize_owned::<ResourceRevalidationDecisionPlan>();
    requires_deserialize_owned::<ResourceStaleAfterDecisionPlan>();
    requires_deserialize_owned::<ResourceTimeoutDecisionPlan>();
    requires_deserialize_owned::<DependencyChangeResourceRevalidationProof>();
    requires_deserialize_owned::<ObserverDemandResourceRevalidationProof>();
    requires_deserialize_owned::<TerminalStateResourceRevalidationProof>();
    requires_deserialize_owned::<FulfilledLifecycleResourceRevalidationProof>();
    requires_deserialize_owned::<ResourceRevalidationFreshnessDecision>();
    requires_deserialize_owned::<ResourceRevalidationCoalescing>();
}
