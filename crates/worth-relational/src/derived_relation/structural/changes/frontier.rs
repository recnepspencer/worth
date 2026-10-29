//! Reobserve Native-issued frontier records on both sides; no caller deltas.
use super::*;
use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind};
use crate::transactions::data::RecordRef;
use crate::validation::data::{StructuralReadError, StructuralRelationRecord};

pub(super) fn acquire(
    proposed: &StructuralRelationView<'_>,
    committed: &StructuralRelationView<'_>,
    touched: &TouchedStructuralSet,
    definition: &Handle,
) -> Result<(Vec<RecordRef>, Vec<AdjacencyDelta>), Denial> {
    let budget = proposed.read_budget();
    let count = touched
        .direct_visible_entity_ids()
        .len()
        .checked_add(touched.visible_relation_ids().len())
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    let bytes = count
        .checked_mul(size_of::<RecordRef>() + 2 * size_of::<AdjacencyDelta>())
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    budget
        .claim_retained(bytes as u64)
        .map_err(Denial::Structural)?;
    let mut records = Vec::new();
    let mut adjacency = Vec::new();
    records
        .try_reserve_exact(count)
        .map_err(|_| allocation_failed())?;
    adjacency
        .try_reserve_exact(
            count
                .checked_mul(2)
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
        )
        .map_err(|_| allocation_failed())?;
    for &id in touched.direct_visible_entity_ids() {
        budget.checkpoint(1).map_err(Denial::Structural)?;
        if entity(proposed, id, definition)? || entity(committed, id, definition)? {
            records.push(RecordRef::Entity(id));
        }
    }
    for &id in touched.visible_relation_ids() {
        budget.checkpoint(1).map_err(Denial::Structural)?;
        let before = relation(committed, id, definition)?;
        let after = relation(proposed, id, definition)?;
        if before.is_none() && after.is_none() {
            continue;
        }
        records.push(RecordRef::Relation(id));
        if before == after {
            continue;
        }
        if let Some(record) = before {
            adjacency.push(delta(record, false));
        }
        if let Some(record) = after {
            adjacency.push(delta(record, true));
        }
    }
    budget.check_live().map_err(Denial::Structural)?;
    Ok((records, adjacency))
}

fn entity(
    view: &StructuralRelationView<'_>,
    id: crate::identity::data::EntityId,
    definition: &Handle,
) -> Result<bool, Denial> {
    match view.readable_entity_kind(id) {
        Ok(Some(kind)) => Ok(definition.entry.carriers.contains(&kind)),
        Ok(None) | Err(StructuralReadError::RecordUnavailable) => Ok(false),
        Err(cause) => Err(view.derived_refusal(cause)),
    }
}

fn relation(
    view: &StructuralRelationView<'_>,
    id: crate::identity::data::RelationId,
    definition: &Handle,
) -> Result<Option<StructuralRelationRecord>, Denial> {
    match view.derived_source_relation(id, definition) {
        Ok(record) => Ok(record),
        Err(StructuralReadError::RecordUnavailable) => Ok(None),
        Err(cause) => Err(view.derived_refusal(cause)),
    }
}

fn allocation_failed() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
}

fn delta(record: StructuralRelationRecord, created: bool) -> AdjacencyDelta {
    let kind = if created {
        AdjacencyDeltaKind::Created {
            source: record.source,
            target: record.target,
        }
    } else {
        AdjacencyDeltaKind::Deleted {
            source: record.source,
            target: record.target,
        }
    };
    AdjacencyDelta {
        relation_id: record.relation_id,
        kind_id: record.kind_id,
        kind,
    }
}
