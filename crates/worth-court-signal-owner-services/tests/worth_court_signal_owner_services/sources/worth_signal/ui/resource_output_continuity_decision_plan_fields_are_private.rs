use worth_signal::facade::core::{
    ResourceOutputContinuityDecisionClass, ResourceOutputContinuityDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceOutputContinuityDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from(
            "signal.resource.output-continuity.preserve-lifecycle-output-separation",
        ),
        class: ResourceOutputContinuityDecisionClass::PreserveWhilePending,
        decision_digest: fake(),
    };
}
