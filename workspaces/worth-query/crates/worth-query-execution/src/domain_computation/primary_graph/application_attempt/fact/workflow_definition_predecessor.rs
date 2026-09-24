use worth_relational::facade::identity::{EntityId, KindId};

use super::{observe_adjacency, WorthQueryApplicationAdjacencyDirection};

pub(super) fn remains_equal(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    lineage: Option<EntityId>,
    expected_definition: Option<EntityId>,
    maximum_work_units: usize,
) -> bool {
    lineage.is_some_and(|lineage| {
        observe_adjacency(
            runtime,
            snapshot,
            relation_kind,
            lineage,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            maximum_work_units,
        )
        .is_some_and(|relations| match expected_definition {
            Some(expected) => {
                matches!(relations.as_slice(), [relation] if relation.to == expected)
            }
            None => relations.is_empty(),
        })
    })
}
