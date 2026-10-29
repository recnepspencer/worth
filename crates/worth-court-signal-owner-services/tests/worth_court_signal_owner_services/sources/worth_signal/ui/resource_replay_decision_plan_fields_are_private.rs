use worth_signal::facade::{ResourceReplayDecisionClass, ResourceReplayDecisionPlan};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceReplayDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.replay.identical-only"),
        class: ResourceReplayDecisionClass::IdenticalOnly,
        decision_digest: fake(),
    };
}
