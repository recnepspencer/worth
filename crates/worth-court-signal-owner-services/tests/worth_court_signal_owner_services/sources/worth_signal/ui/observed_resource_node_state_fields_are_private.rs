use worth_signal::facade::ObservedResourceNodeState;

fn fake<T>() -> T {
    panic!("not executed")
}

fn main() {
    let _state = ObservedResourceNodeState {
        node: fake(),
        lifecycle: fake(),
        lifecycle_ordinal: fake(),
        output_continuity: fake(),
        denied_completion: fake(),
        scheduled_retry: fake(),
        observation_decision_digest: fake(),
    };
}
