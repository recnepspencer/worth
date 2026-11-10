use std::sync::Arc;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::ConsumedOutputCurrentness;
    use im::OrdSet;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::SourceSettlementCurrentness;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::verification::pending_upstream::upstream_settled;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;

#[test]
fn only_a_settled_upstream_can_resolve_a_pending_mark() {
    with_committed_own_write(|_, _, cell, _| {
        let identity = Arc::clone(cell.get().unwrap().settlement_identity_for_test());
        assert!(upstream_settled(
            ConsumedOutputCurrentness::CanonicallyEqualClean(identity)
        ));
        assert!(upstream_settled(ConsumedOutputCurrentness::Direct(
            SourceSettlementCurrentness::Clean
        )));
        assert!(!upstream_settled(
            ConsumedOutputCurrentness::PendingEqualSuccessor
        ));
        assert!(!upstream_settled(ConsumedOutputCurrentness::Direct(
            SourceSettlementCurrentness::Dirty(OrdSet::new())
        )));
    });
}
