use worth_signal::facade::core::{
    ResourceStaleAfterDecisionClass, ResourceStaleAfterDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceStaleAfterDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.stale-after.runtime-stale-after"),
        class: ResourceStaleAfterDecisionClass::RuntimeStaleAfter,
        stale_after: fake(),
        decision_digest: fake(),
    };
}
