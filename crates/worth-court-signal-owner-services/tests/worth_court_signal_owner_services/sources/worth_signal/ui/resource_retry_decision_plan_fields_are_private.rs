use worth_signal::facade::core::{ResourceRetryDecisionClass, ResourceRetryDecisionPlan};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceRetryDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.retry.fixed-delay"),
        class: ResourceRetryDecisionClass::FixedDelay,
        initial_delay: fake(),
        multiplier: Some(2),
        max_delay: fake(),
        decision_digest: fake(),
    };
}
