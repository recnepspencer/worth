use super::{changes::ChangedRecords, entry_edits::edit, reads, work::MaintenanceWork};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::{
    DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial, RelatedEntityEndpoint,
    RelatedEntityOrderingEntry, RelatedEntityOrderingField,
};
use crate::indexes::projected_field_values::{
    compare_related_entries, entity_index_projection_scope, IndexProjectionSource,
};
use crate::runtime::VisibilityProjectionView;
use crate::visibility::materialization::read_records::entity_query_locus_value;

struct OrderingContract<'a> {
    relation_kind: KindId,
    parent_endpoint: RelatedEntityEndpoint,
    child_kind: KindId,
    ordering: &'a [RelatedEntityOrderingField],
}

pub(super) fn refresh(
    entries: &mut DerivedIndexEntryMap<EntityId, RelatedEntityOrderingEntry>,
    contract: (
        KindId,
        RelatedEntityEndpoint,
        KindId,
        &[RelatedEntityOrderingField],
    ),
    changes: &ChangedRecords,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let contract = OrderingContract {
        relation_kind: contract.0,
        parent_endpoint: contract.1,
        child_kind: contract.2,
        ordering: contract.3,
    };
    work.charge(changes.relations.len())?;
    let mut affected = changes.relations.clone();
    for child in &changes.entities {
        let old = child_values(before, *child, &contract, work)?;
        let new = child_values(Some(after), *child, &contract, work)?;
        if old == new {
            continue;
        }
        for view in before.into_iter().chain(std::iter::once(after)) {
            let outgoing = contract.parent_endpoint == RelatedEntityEndpoint::TargetParent;
            for relation in reads::adjacency(view, *child, contract.relation_kind, outgoing, work)?
            {
                work.charge(1)?;
                affected.insert(relation.relation_id);
            }
        }
    }
    for relation in affected {
        let old = row(before, relation, &contract, work)?;
        let new = row(Some(after), relation, &contract, work)?;
        if old == new {
            continue;
        }
        let compare = |left: &RelatedEntityOrderingEntry, right: &RelatedEntityOrderingEntry| {
            compare_related_entries(left, right, contract.ordering)
        };
        if let Some((parent, row)) = old {
            edit(entries, parent, row, false, compare, work)?;
        }
        if let Some((parent, row)) = new {
            edit(entries, parent, row, true, compare, work)?;
        }
    }
    Ok(())
}

fn child_values(
    view: Option<&VisibilityProjectionView<'_>>,
    child: EntityId,
    contract: &OrderingContract<'_>,
    work: &mut MaintenanceWork,
) -> Result<Option<Vec<worth_foundational::facade::AspectValue>>, Denial> {
    let Some(record) = reads::entity(view, child, work)? else {
        return Ok(None);
    };
    if record.kind.kind_id != contract.child_kind {
        return Ok(None);
    }
    let source = IndexProjectionSource::selected(view.unwrap());
    let Some(plan) = source.entity_aspect_plan(contract.child_kind) else {
        return Ok(None);
    };
    work.charge(contract.ordering.len())?;
    Ok(contract
        .ordering
        .iter()
        .map(|field| {
            entity_index_projection_scope(plan, field.locator())?;
            entity_query_locus_value(&record, field.locator()).cloned()
        })
        .collect())
}

fn row(
    view: Option<&VisibilityProjectionView<'_>>,
    id: RelationId,
    contract: &OrderingContract<'_>,
    work: &mut MaintenanceWork,
) -> Result<Option<(EntityId, RelatedEntityOrderingEntry)>, Denial> {
    let Some(relation) = reads::relation(view, id, work)? else {
        return Ok(None);
    };
    if relation.kind.kind_id != contract.relation_kind {
        return Ok(None);
    }
    let (parent, child) = match contract.parent_endpoint {
        RelatedEntityEndpoint::SourceParent => (relation.source, relation.target),
        RelatedEntityEndpoint::TargetParent => (relation.target, relation.source),
    };
    Ok(child_values(view, child, contract, work)?.map(|values| {
        (
            parent,
            RelatedEntityOrderingEntry::new(values, child, relation.relation_id),
        )
    }))
}
