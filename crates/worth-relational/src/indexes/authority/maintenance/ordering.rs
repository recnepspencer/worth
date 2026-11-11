use super::root_view::IndexRootView;
use super::{
    change_routing::ChangeRouting, entry_edits::edit, persistent_allocation::UngroupedAllocation,
    reads, record_metadata, work::MaintenanceWork,
};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::{
    DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial, RelatedEntityEndpoint,
    RelatedEntityOrderingEntry, RelatedEntityOrderingField,
};
use crate::indexes::projected_field_values::{
    compare_related_entries, entity_index_projection_scope,
};
use crate::visibility::materialization::read_records::authoritative_state_query_locus_value;

struct OrderingContract<'a> {
    relation_kind: KindId,
    parent_endpoint: RelatedEntityEndpoint,
    child_kind: KindId,
    ordering: &'a [RelatedEntityOrderingField],
}

pub(super) fn refresh(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entries: &mut DerivedIndexEntryMap<EntityId, RelatedEntityOrderingEntry>,
    contract: (
        KindId,
        RelatedEntityEndpoint,
        KindId,
        &[RelatedEntityOrderingField],
    ),
    changes: &ChangeRouting,
    before: Option<&IndexRootView>,
    after: &IndexRootView,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let mut allocation = UngroupedAllocation::new();
    let contract = OrderingContract {
        relation_kind: contract.0,
        parent_endpoint: contract.1,
        child_kind: contract.2,
        ordering: contract.3,
    };
    let mut affected = std::collections::BTreeSet::new();
    for relation in changes.relations(contract.relation_kind, work)? {
        work.charge(1)?;
        work.ordered::<RelationId, ()>(affected.len(), 1, 0)?;
        affected.insert(relation.id);
    }
    for child in changes.entities(contract.child_kind, work)? {
        work.charge(1)?;
        let old = child_values(execution, before, child.id, &contract, work)?;
        let new = child_values(execution, Some(after), child.id, &contract, work)?;
        if old == new {
            continue;
        }
        for view in before.into_iter().chain(std::iter::once(after)) {
            let outgoing = contract.parent_endpoint == RelatedEntityEndpoint::TargetParent;
            for relation in reads::adjacency(
                execution,
                view,
                child.id,
                contract.relation_kind,
                outgoing,
                work,
            )? {
                work.charge(1)?;
                work.ordered::<RelationId, ()>(affected.len(), 1, 0)?;
                affected.insert(relation.id);
            }
        }
    }
    for relation in affected {
        let old = row(execution, before, relation, &contract, work)?;
        let new = row(execution, Some(after), relation, &contract, work)?;
        if old == new {
            continue;
        }
        let compare = |left: &RelatedEntityOrderingEntry, right: &RelatedEntityOrderingEntry| {
            compare_related_entries(left, right, contract.ordering)
        };
        if let Some((parent, row)) = old {
            edit(entries, parent, row, false, compare, &mut allocation, work)?;
        }
        if let Some((parent, row)) = new {
            edit(entries, parent, row, true, compare, &mut allocation, work)?;
        }
    }
    Ok(())
}

fn child_values(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: Option<&IndexRootView>,
    child: EntityId,
    contract: &OrderingContract<'_>,
    work: &mut MaintenanceWork,
) -> Result<Option<Vec<worth_foundational::facade::AspectValue>>, Denial> {
    let Some(view) = view else {
        return Ok(None);
    };
    if record_metadata::entity_kind(execution, Some(view), child, work)?
        != Some(contract.child_kind)
    {
        return Ok(None);
    }
    let Some(head) = view.head(execution, child.partition_id, work)? else {
        return Ok(None);
    };
    let Some(record) = head.entity_arena().get_slot(child.slot_index()) else {
        return Ok(None);
    };
    let plans = view.root().schema_authority().aspect_plans();
    work.lookup(plans.entity_plans.len(), 1)?;
    let Some(plan) = plans.entity_plans.get(&contract.child_kind) else {
        return Ok(None);
    };
    work.array::<worth_foundational::facade::AspectValue>(contract.ordering.len())?;
    let mut values = Vec::with_capacity(contract.ordering.len());
    for field in contract.ordering {
        super::field_keys::prepare_scope(plan, field.locator(), work)?;
        if entity_index_projection_scope(plan, field.locator()).is_none() {
            return Ok(None);
        }
        let Some(value) = authoritative_state_query_locus_value(
            record.extra().authoritative_aspect_state(),
            field.locator(),
        ) else {
            return Ok(None);
        };
        work.prepare(1, value.owned_allocation_capacity_bytes() as u64)?;
        values.push(value.clone());
    }
    Ok(Some(values))
}

fn row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: Option<&IndexRootView>,
    id: RelationId,
    contract: &OrderingContract<'_>,
    work: &mut MaintenanceWork,
) -> Result<Option<(EntityId, RelatedEntityOrderingEntry)>, Denial> {
    let Some(relation) = record_metadata::relation(execution, view, id, work)? else {
        return Ok(None);
    };
    if relation.kind != contract.relation_kind {
        return Ok(None);
    }
    let (parent, child) = match contract.parent_endpoint {
        RelatedEntityEndpoint::SourceParent => (relation.source, relation.target),
        RelatedEntityEndpoint::TargetParent => (relation.target, relation.source),
    };
    Ok(
        child_values(execution, view, child, contract, work)?.map(|values| {
            (
                parent,
                RelatedEntityOrderingEntry::new(values, child, relation.id),
            )
        }),
    )
}

pub(super) fn project_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &IndexRootView,
    id: RelationId,
    contract: (
        KindId,
        RelatedEntityEndpoint,
        KindId,
        &[RelatedEntityOrderingField],
    ),
    work: &mut MaintenanceWork,
) -> Result<Option<(EntityId, RelatedEntityOrderingEntry)>, Denial> {
    row(
        execution,
        Some(view),
        id,
        &OrderingContract {
            relation_kind: contract.0,
            parent_endpoint: contract.1,
            child_kind: contract.2,
            ordering: contract.3,
        },
        work,
    )
}
