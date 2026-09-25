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
    observe_adjacency_checked(
        runtime,
        snapshot,
        relation_kind,
        anchor,
        direction,
        maximum_work_units,
    )
    .ok()
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum AdjacencyObservationDenial {
    SnapshotUnavailable,
    WorkBudgetExceeded,
}

pub(in crate::domain_computation::primary_graph) fn observe_adjacency_checked(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: usize,
) -> Result<Vec<WorthQueryApplicationObservedRelation>, AdjacencyObservationDenial> {
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or(AdjacencyObservationDenial::SnapshotUnavailable)?;
    let frontier = std::collections::BTreeSet::from([anchor]);
    // This scalar API budgets examined relations plus returned endpoints. The
    // exact-root frontier primitive additionally charges its one anchor list.
    let work = maximum_work_units
        .checked_add(1)
        .ok_or(AdjacencyObservationDenial::WorkBudgetExceeded)?;
    let read = match direction {
        WorthQueryApplicationAdjacencyDirection::Outgoing => {
            view.bounded_outgoing_relations_for_frontier(&frontier, relation_kind, work)
        }
        WorthQueryApplicationAdjacencyDirection::Incoming => {
            view.bounded_incoming_relations_for_frontier(&frontier, relation_kind, work)
        }
    }
    .map_err(|_| AdjacencyObservationDenial::WorkBudgetExceeded)?;
    Ok(read
        .into_records()
        .into_iter()
        .map(|record| WorthQueryApplicationObservedRelation {
            relation_id: record.relation_id,
            from: record.source,
            to: record.target,
        })
        .collect())
}
