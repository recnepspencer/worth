use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::product_retirement::retained_among;
    use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support;

#[test]
fn a_provider_without_world_history_keeps_every_candidate() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    assert!(retained_among(None, std::slice::from_ref(&commit)).contains(&commit));
}
