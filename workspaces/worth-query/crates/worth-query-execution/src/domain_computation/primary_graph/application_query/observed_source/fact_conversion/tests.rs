use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, VersionId},
    runtime::RelationalAdjacencyDirection,
};

use super::Fact;

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
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(vec![endpoint]), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        };
    let path = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(4)),
            comparison_work_limit: 1,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(vec![]), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        };
    assert!(projected
        .merge_same_source_fact(
            path,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            None
        )
        .unwrap());
    assert!(
        matches!(&projected, Fact::SourceAdjacencyRevision { endpoints, .. } if endpoints.as_ref() == &[endpoint])
    );
    let changed = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(3),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(5)),
            comparison_work_limit: 1,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(vec![]), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        };
    assert!(!projected
        .merge_same_source_fact(
            changed,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            None
        )
        .unwrap());
}
