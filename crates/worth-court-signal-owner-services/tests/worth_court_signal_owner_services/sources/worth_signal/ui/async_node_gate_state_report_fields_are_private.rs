use worth_signal::facade::AsyncNodeGateStateReport;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _report = AsyncNodeGateStateReport {
        node: fake(),
        upstream_dependency_count: 0,
        downstream_subscriber_count: 0,
        lifecycle_class: fake(),
        active_request_handle: fake(),
        committed_output_identity: fake(),
        output_continuity: fake(),
        latest_observation_match_count: 0,
        downstream_dependence_facts: fake(),
        gate_digest: fake(),
        performance: fake(),
    };
}
