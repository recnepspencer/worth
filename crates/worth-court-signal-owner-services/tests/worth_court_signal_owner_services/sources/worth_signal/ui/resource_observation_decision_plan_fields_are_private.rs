use worth_signal::facade::core::{
    ResourceObservationDecisionClass, ResourceObservationDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceObservationDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.observation.lifecycle-only"),
        class: ResourceObservationDecisionClass::LifecycleOnly,
        decision_digest: fake(),
    };
}
