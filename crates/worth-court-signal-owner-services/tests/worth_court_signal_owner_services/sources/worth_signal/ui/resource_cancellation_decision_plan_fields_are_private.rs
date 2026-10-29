use worth_signal::facade::core::{
    ResourceCancellationDecisionClass, ResourceCancellationDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceCancellationDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.cancellation.runtime-denial-only"),
        class: ResourceCancellationDecisionClass::RuntimeDenialOnly,
        requests_host_advisory: false,
        grace_period: None,
        declared_dependent_cancellation_nodes: vec![],
        decision_digest: fake(),
    };
}
