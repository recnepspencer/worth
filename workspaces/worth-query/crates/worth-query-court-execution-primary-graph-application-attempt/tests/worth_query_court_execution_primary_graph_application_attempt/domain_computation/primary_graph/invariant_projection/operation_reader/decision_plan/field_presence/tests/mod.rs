use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::decision_plan::field_presence::may_write;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountNote;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountScore;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PatchAccountDraftOperation;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::WrongFieldRetentionOperation;

#[test]
fn optional_aspect_initialization_presence_follows_exact_installed_write_membership() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let schema = world.application.installed_schema();
    let writer = schema
        .installed_operation(PatchAccountDraftOperation::reference())
        .unwrap();
    let nonwriter = schema
        .installed_operation(WrongFieldRetentionOperation::reference())
        .unwrap();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    for field in [
        AccountNote::reference().field(),
        AccountScore::reference().field(),
    ] {
        let locator = (*graph.layout_for_test())
            .field_locator_for_test("Account", "AccountPolicy", field)
            .unwrap();
        assert!(may_write(writer.contracts().touches(), "Account", locator));
        assert!(!may_write(
            nonwriter.contracts().touches(),
            "Account",
            locator
        ));
    }
}
