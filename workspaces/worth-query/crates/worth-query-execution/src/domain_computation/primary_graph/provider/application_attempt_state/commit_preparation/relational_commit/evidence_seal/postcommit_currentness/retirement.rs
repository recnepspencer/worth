//! Exact output retirements replace their own reads with committed lifecycle truth.
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use std::collections::BTreeSet;
use worth_relational::facade::identity::EntityId;

/// The entity this commit retired that `fact` was read at; `None` for every
/// other fact. It reads nothing.
pub(super) fn retired_anchor(fact: &Fact, retired: &BTreeSet<EntityId>) -> Option<EntityId> {
    let anchor = match fact {
        Fact::SourceEntity { entity_id }
        | Fact::Entity { entity_id, .. }
        | Fact::SourceAspectRevision { entity_id, .. }
        | Fact::SourceFieldRevision { entity_id, .. }
        | Fact::Field { entity_id, .. }
        | Fact::AbsentField { entity_id, .. } => Some(*entity_id),
        Fact::SourceAdjacencyRevision { anchor, .. } | Fact::Adjacency { anchor, .. } => {
            Some(*anchor)
        }
        // Pair reads are rebased at their outgoing anchor. External anchors
        // still compare their native adjacency after the output endpoint retires.
        Fact::Relation { from, .. } => Some(*from),
        _ => None,
    };
    anchor.filter(|entity| retired.contains(entity))
}

/// The committed retirement that replaces `fact`; `None` when the snapshot
/// does not hold that retirement at its own version.
pub(super) fn committed_retirement(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity_id: EntityId,
    fact: &Fact,
) -> Option<Fact> {
    let view = runtime.read_truth().project_snapshot(snapshot)?;
    let record = view.entity_retirement(entity_id)?;
    if record.deleted_at_version() != view.version_id() {
        return None;
    }
    Some(Fact::RetiredOutputEntity {
        entity_id,
        kind: record.kind_id(),
        created_at: record.created_at_version(),
        deleted_at: record.deleted_at_version(),
        read_locator: fact.locator_identity(),
    })
}

#[cfg(test)]
pub(super) fn rebase(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: Vec<Fact>,
    retired: &BTreeSet<EntityId>,
) -> Option<Vec<Fact>> {
    facts
        .into_iter()
        .map(|fact| match retired_anchor(&fact, retired) {
            Some(entity_id) => committed_retirement(runtime, snapshot, entity_id, &fact),
            None => Some(fact),
        })
        .collect()
}

#[cfg(test)]
mod tests;
