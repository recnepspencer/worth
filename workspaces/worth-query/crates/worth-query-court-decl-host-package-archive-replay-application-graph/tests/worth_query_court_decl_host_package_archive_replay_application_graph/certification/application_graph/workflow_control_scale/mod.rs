pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_control_scale::*;

#[test]
fn repeated_control_fragment_100_nodes_publishes_and_starts_with_flat_bytes_per_node() {
    assert_bytes_per_node_stay_flat(qualify(11, 918_600), qualify(22, 918_630));
}

#[test]
fn repeated_control_fragment_1000_nodes_publishes_and_starts() {
    qualify(111, 918_610);
}
