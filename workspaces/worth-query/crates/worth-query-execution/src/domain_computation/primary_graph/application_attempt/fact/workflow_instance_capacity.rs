use worth_relational::facade::identity::{EntityId, KindId};

use super::{
    observe_adjacency, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationObservedRelation,
};

pub(super) fn remains_equal(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    lineage: EntityId,
    maximum_instances: usize,
    expected: &[WorthQueryApplicationObservedRelation],
) -> bool {
    expected.len() < maximum_instances
        && observe_adjacency(
            runtime,
            snapshot,
            relation_kind,
            lineage,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            maximum_instances.saturating_mul(2).saturating_add(1),
        )
        .is_some_and(|current| current == expected)
}
