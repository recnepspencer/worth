use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::graph_provider::tests::support::attempt;
    use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::graph_provider::tests::support::call;
    use std::sync::Arc;
    use worth_query_execution::court_doors_for_test::domain_computation::provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation;
    use worth_query_execution::court_doors_for_test::domain_computation::domain_evidence_binding::execution_snapshot::WorthQueryBoundExecutionSnapshotIdentity;

#[test]
fn exact_execution_association_denies_each_independent_owner_axis_swap() {
    let exact_attempt = attempt();
    let foreign_attempt = attempt();
    let exact =
        WorthQueryBoundGraphExecutionAssociation::capture(&call(&exact_attempt, "same-scope"));
    let foreign =
        WorthQueryBoundGraphExecutionAssociation::capture(&call(&foreign_attempt, "same-scope"));
    let expected_snapshot =
        WorthQueryBoundExecutionSnapshotIdentity::capture(Arc::from("snapshot"));
    let authority = exact_attempt.attempt.binding_authority();
    let session = exact_attempt.attempt.provider_session_for_test();

    assert!(exact.admits_execution(authority, session, None, &expected_snapshot));

    let mut session_swap = exact.clone();
    session_swap.provider_session = foreign.provider_session.clone();
    assert!(!session_swap.admits_execution(authority, session, None, &expected_snapshot));

    let mut attempt_swap = exact.clone();
    attempt_swap.attempt = foreign.attempt.clone();
    assert!(!attempt_swap.admits_execution(authority, session, None, &expected_snapshot));

    assert!(!exact.admits_execution(
        authority,
        session,
        Some("foreign-stage"),
        &expected_snapshot,
    ));

    let foreign_snapshot =
        WorthQueryBoundExecutionSnapshotIdentity::capture(Arc::from("foreign-snapshot"));
    assert!(!exact.admits_execution(authority, session, None, &foreign_snapshot));
}
