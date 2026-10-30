use worth_signal::facade::core::{ResourceTimeoutDecisionClass, ResourceTimeoutDecisionPlan};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceTimeoutDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.timeout.fixed-timeout"),
        class: ResourceTimeoutDecisionClass::FixedTimeout,
        timeout: fake(),
        decision_digest: fake(),
    };
}
