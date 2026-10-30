pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_component_scale::*;

#[test]
#[ignore = "scheduled public 10k-node repeated-component qualification"]
fn repeated_component_10000_nodes_publishes_and_starts() {
    qualify(1111, 918_520);
}
