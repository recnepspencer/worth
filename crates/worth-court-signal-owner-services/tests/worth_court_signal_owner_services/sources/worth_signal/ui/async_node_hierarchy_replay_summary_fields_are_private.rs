use worth_signal::facade::AsyncNodeHierarchyReplaySummary;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _summary = AsyncNodeHierarchyReplaySummary {
        root_node: fake(),
        hierarchy_nodes: fake(),
        active_request_handles: fake(),
        hierarchy_depth: 0,
        lifecycle_digest: fake(),
        in_flight_digest: fake(),
        replay_digest: fake(),
        performance: fake(),
    };
}
