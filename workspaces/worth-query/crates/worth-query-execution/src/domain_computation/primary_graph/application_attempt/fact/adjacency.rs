use worth_relational::facade::identity::{EntityId, KindId};

use super::{WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationObservedRelation};

pub(super) fn remains_equal(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: usize,
    expected: &[WorthQueryApplicationObservedRelation],
) -> bool {
    observe_adjacency(
        runtime,
        snapshot,
        relation_kind,
        anchor,
        direction,
        maximum_work_units,
    )
    .is_some_and(|current| current == expected)
}

pub(in crate::domain_computation::primary_graph) fn observe_adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: usize,
) -> Option<Vec<WorthQueryApplicationObservedRelation>> {
    let read = match direction {
        WorthQueryApplicationAdjacencyDirection::Outgoing => runtime
            .read_truth()
            .bounded_outgoing_relations_of_kind_at_version(
                anchor,
                relation_kind,
                snapshot.version_id(),
                maximum_work_units,
            ),
        WorthQueryApplicationAdjacencyDirection::Incoming => runtime
            .read_truth()
            .bounded_incoming_relations_of_kind_at_version(
                anchor,
                relation_kind,
                snapshot.version_id(),
                maximum_work_units,
            ),
    }
    .ok()?;
    Some(
        read.into_records()
            .into_iter()
            .map(|record| WorthQueryApplicationObservedRelation {
                relation_id: record.relation_id,
                from: record.source,
                to: record.target,
            })
            .collect(),
    )
}
