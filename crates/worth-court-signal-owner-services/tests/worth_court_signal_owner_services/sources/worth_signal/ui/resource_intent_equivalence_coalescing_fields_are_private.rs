use worth_signal::facade::{
    AdmittedResourceRequest, ResourceIntentEquivalenceCoalescing, ResourceLifecycleTransition,
    ResourcePolicyDigest, ResourceRequestHandle, ResourceRequestIntentDigest,
    ResourceSupersessionOrdinal,
};

fn forged_handle() -> ResourceRequestHandle {
    loop {}
}

fn forged_request() -> AdmittedResourceRequest {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn forged_intent_digest() -> ResourceRequestIntentDigest {
    loop {}
}

fn forged_policy_digest() -> ResourcePolicyDigest {
    loop {}
}

fn main() {
    let _ = ResourceIntentEquivalenceCoalescing {
        supersession_ordinal: ResourceSupersessionOrdinal::new(0),
        winner: forged_handle(),
        coalesced_request: forged_request(),
        intent_digest: forged_intent_digest(),
        policy_decision_digest: forged_policy_digest(),
        lifecycle_transition: forged_transition(),
    };
}
