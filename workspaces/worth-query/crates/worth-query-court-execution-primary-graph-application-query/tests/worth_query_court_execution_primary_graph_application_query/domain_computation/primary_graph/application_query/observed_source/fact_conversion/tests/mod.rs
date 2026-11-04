pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::identity::VersionId;
pub use worth_relational::facade::runtime::RelationalAdjacencyDirection;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;

#[test]
fn overlapping_path_and_projection_adjacency_merge_at_one_native_revision() {
    let anchor = EntityId::new(PartitionId::main(), 1, 1);
    let endpoint = EntityId::new(PartitionId::main(), 2, 1);
    let mut projected = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(4)),
            comparison_work_limit: 1,
            endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&[endpoint], worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        };
    let path = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(4)),
            comparison_work_limit: 1,
            endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&[], worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        };
    assert!(projected
        .merge_same_source_fact_for_test(
            path,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)
        )
        .unwrap());
    assert!(
        matches!(&projected, Fact::SourceAdjacencyRevision { endpoints, .. } if endpoints.as_ref() == [endpoint])
    );
    let changed = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(5)),
            comparison_work_limit: 1,
            endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&[], worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        };
    assert!(!projected
        .merge_same_source_fact_for_test(
            changed,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)
        )
        .unwrap());
}

#[path = "expansion/mod.rs"]
pub mod expansion;
