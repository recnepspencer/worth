use worth_signal::facade::core::{
    ResourceRetentionDecisionClass, ResourceRetentionDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceRetentionDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.retention.retain-all-transitions"),
        class: ResourceRetentionDecisionClass::RetainAllTransitions,
        decision_digest: fake(),
    };
}
