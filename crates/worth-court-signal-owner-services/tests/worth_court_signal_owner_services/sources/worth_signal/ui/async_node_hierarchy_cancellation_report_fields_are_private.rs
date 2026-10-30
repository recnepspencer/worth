use worth_signal::facade::AsyncNodeHierarchyCancellationReport;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _report = AsyncNodeHierarchyCancellationReport {
        root_node: fake(),
        affected_nodes: fake(),
        propagated_hierarchy_width: 0,
        replay_digest: fake(),
        cancellation: fake(),
    };
}
