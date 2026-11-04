pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::fact_store::AuthoringSourceFacts;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::fact_store::RetainedSourceFacts;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::source_facts::storage::SourceFacts;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::source_facts::merge_source_facts;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::identity::VersionId;
pub use worth_relational::facade::runtime::RelationalAdjacencyDirection;

pub fn adjacency(
    revision: u64,
    limit: usize,
    endpoints: Vec<EntityId>,
) -> WorthQueryApplicationObservedFact {
    WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind: KindId::new(3),
        anchor: EntityId::new(PartitionId::main(), 1, 1),
        direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: Some(VersionId(revision)),
        comparison_work_limit: limit,
        endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&endpoints, worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
    }
}

pub fn control() -> StorageControl<'static, 'static> {
    StorageControl::new_for_test(
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        None,
    )
}
pub fn keyed(
    facts: impl IntoIterator<Item = WorthQueryApplicationObservedFact>,
) -> Option<RetainedSourceFacts> {
    let mut store = AuthoringSourceFacts::new(control()).unwrap();
    for fact in facts {
        store.capture_for_test(fact, control()).unwrap();
    }
    Some(store.finish_for_test(control()).unwrap())
}
