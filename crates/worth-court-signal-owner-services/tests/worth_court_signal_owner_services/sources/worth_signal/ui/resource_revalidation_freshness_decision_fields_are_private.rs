use worth_signal::facade::core::ResourceRevalidationFreshnessDecision;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _decision = ResourceRevalidationFreshnessDecision {
        class: fake(),
        freshness_digest: fake(),
        policy_decision_digest: fake(),
    };
}
