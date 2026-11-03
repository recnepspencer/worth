use worth_relational::facade::indexes::BoundedEntityFieldLookupAdmissionStop;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::decision_facts::comparison::indexed::comparison_stop;
#[test]
fn read_fold_index_accounting_overflow_is_a_request_stop() {
    let denied = comparison_stop(BoundedEntityFieldLookupAdmissionStop::AccountingOverflow)
        .expect_err("accounting overflow cannot answer not equal");
    assert_eq!(denied.kind(), worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetDenialKind::ExecutionRequest(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkCounterOverflow,
            partition_identity: None, policy_ancestor: None,
        }));
    assert!(!comparison_stop(BoundedEntityFieldLookupAdmissionStop::ExactBasisRequired).unwrap());
}
