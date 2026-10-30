use worth_signal::facade::AsyncNodeAdmissionClassification;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _classification = AsyncNodeAdmissionClassification {
        node: fake(),
        node_state: fake(),
        lifecycle_class: fake(),
        condition: fake(),
        class: fake(),
        condition_block_class: fake(),
        dirty_aspects: fake(),
        max_dependency_delta: 0,
        previous_value_reference: fake(),
        decision_digest: fake(),
        performance: fake(),
    };
}
