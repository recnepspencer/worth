use worth_signal::facade::core::{
    DependencyChangeResourceRevalidationProof, FulfilledLifecycleResourceRevalidationProof,
    ObserverDemandResourceRevalidationProof, ResourceRetryDecisionPlan,
    ResourceRevalidationCoalescing, ResourceRevalidationDecisionPlan,
    ResourceRevalidationFreshnessDecision, ResourceStaleAfterDecisionPlan,
    ResourceTimeoutDecisionPlan, TerminalStateResourceRevalidationProof,
};

fn requires_deserialize_owned<T: serde::de::DeserializeOwned>() {}

fn main() {
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
