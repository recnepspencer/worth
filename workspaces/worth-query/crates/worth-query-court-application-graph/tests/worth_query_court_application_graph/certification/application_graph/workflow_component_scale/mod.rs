pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_component_scale::*;

#[test]
fn repeated_component_100_nodes_publishes_and_starts_with_flat_bytes_per_node() {
    assert_marginal_bytes_per_node_stay_flat(11, 918_500);
}

#[test]
fn repeated_component_1000_nodes_publishes_and_starts() {
    let (small_nodes, small_bytes) = qualify(11, 918_508);
    let (large_nodes, large_bytes) = qualify(111, 918_510);
    assert!(large_bytes > small_bytes);
    assert!(
        large_bytes * small_nodes <= small_bytes * large_nodes * 2,
        "publication bytes must grow linearly with expanded nodes: {small_nodes}/{small_bytes} -> {large_nodes}/{large_bytes}"
    );
}
