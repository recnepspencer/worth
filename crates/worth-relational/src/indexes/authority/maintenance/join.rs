use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::{
    DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial, RelationJoinDefinition,
    RelationJoinEntry, RelationJoinKey, RelationJoinLeg, RelationJoinSharedEndpoint,
};
use crate::indexes::projected_field_values::join_endpoints;
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::RelationReadRecord;

use super::{
    change_routing::ChangeRouting, entry_edits::edit, reads, record_metadata, work::MaintenanceWork,
};

pub(super) fn refresh(
    entries: &mut DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>,
    definition: RelationJoinDefinition,
    changes: &ChangeRouting,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let affected = affected_shared(definition, changes, before, after, work)?;
    for shared in affected {
        let old = rows_for_shared(before, shared, definition, work)?;
        let new = rows_for_shared(Some(after), shared, definition, work)?;
        for (key, row) in &old {
            if new.get(key) != Some(row) {
                edit(entries, *key, *row, false, Ord::cmp, work)?;
            }
        }
        for (key, row) in &new {
            if old.get(key) != Some(row) {
                edit(entries, *key, *row, true, Ord::cmp, work)?;
            }
        }
    }
    Ok(())
}

fn affected_shared(
    definition: RelationJoinDefinition,
    changes: &ChangeRouting,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<BTreeSet<EntityId>, Denial> {
    let mut affected = BTreeSet::new();
    let legs = [definition.left(), definition.right()];
    for (i, leg) in legs.iter().enumerate() {
        if i == 1 && leg.relation_kind() == legs[0].relation_kind() {
            continue;
        }
        for change in changes.relations(leg.relation_kind(), work)? {
            work.charge(1)?;
            for relation in change.old.into_iter().chain(change.new) {
                if relation.kind != leg.relation_kind() {
                    continue;
                }
                for matched in legs.iter().filter(|l| l.relation_kind() == relation.kind) {
                    let shared = match matched.shared_endpoint() {
                        RelationJoinSharedEndpoint::Source => relation.source,
                        RelationJoinSharedEndpoint::Target => relation.target,
                    };
                    work.charge(1)?;
                    work.ordered::<EntityId, ()>(affected.len(), 1, 0)?;
                    affected.insert(shared);
                }
            }
        }
    }
    let kinds = [
        definition.shared_entity_kind(),
        legs[0].external_entity_kind(),
        legs[1].external_entity_kind(),
    ];
    for (i, kind) in kinds.iter().enumerate() {
        if kinds[..i].contains(kind) {
            continue;
        }
        for change in changes.entities(*kind, work)? {
            work.charge(1)?;
            if change.old == change.new {
                continue;
            }
            if *kind == definition.shared_entity_kind() {
                work.charge(1)?;
                work.ordered::<EntityId, ()>(affected.len(), 1, 0)?;
                affected.insert(change.id);
            }
            for leg in legs.iter().filter(|l| l.external_entity_kind() == *kind) {
                for view in before.into_iter().chain(std::iter::once(after)) {
                    let outgoing = leg.shared_endpoint() == RelationJoinSharedEndpoint::Target;
                    for relation in
                        reads::adjacency(view, change.id, leg.relation_kind(), outgoing, work)?
                    {
                        work.charge(1)?;
                        work.ordered::<EntityId, ()>(affected.len(), 1, 0)?;
                        affected.insert(join_endpoints(&relation, leg.shared_endpoint()).0);
                    }
                }
            }
        }
    }
    Ok(affected)
}

fn rows_for_shared(
    view: Option<&VisibilityProjectionView<'_>>,
    shared: EntityId,
    definition: RelationJoinDefinition,
    work: &mut MaintenanceWork,
) -> Result<BTreeMap<RelationJoinKey, RelationJoinEntry>, Denial> {
    let Some(view) = view else {
        return Ok(BTreeMap::new());
    };
    if record_metadata::entity_kind(Some(view), shared, work)?
        != Some(definition.shared_entity_kind())
    {
        return Ok(BTreeMap::new());
    }
    let left = leg_relations(view, shared, definition.left(), work)?;
    let right = leg_relations(view, shared, definition.right(), work)?;
    let mut rows = BTreeMap::new();
    for (left_id, left_relation) in left {
        for (right_id, right_relation) in &right {
            work.derive_row()?;
            rows.insert(
                RelationJoinKey::new(left_id, *right_id),
                RelationJoinEntry::new(shared, left_relation, *right_relation),
            );
        }
    }
    Ok(rows)
}

fn leg_relations(
    view: &VisibilityProjectionView<'_>,
    shared: EntityId,
    leg: RelationJoinLeg,
    work: &mut MaintenanceWork,
) -> Result<BTreeMap<EntityId, RelationId>, Denial> {
    let outgoing = leg.shared_endpoint() == RelationJoinSharedEndpoint::Source;
    let mut selected = BTreeMap::new();
    for relation in reads::adjacency(view, shared, leg.relation_kind(), outgoing, work)? {
        insert_leg(&mut selected, view, shared, leg, &relation, work)?;
    }
    Ok(selected)
}

fn insert_leg(
    selected: &mut BTreeMap<EntityId, RelationId>,
    view: &VisibilityProjectionView<'_>,
    shared: EntityId,
    leg: RelationJoinLeg,
    relation: &RelationReadRecord,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let (observed_shared, external) = join_endpoints(relation, leg.shared_endpoint());
    if observed_shared != shared {
        return Ok(());
    }
    if record_metadata::entity_kind(Some(view), external, work)? != Some(leg.external_entity_kind())
    {
        return Ok(());
    }
    work.charge(1)?;
    selected
        .entry(external)
        .and_modify(|id| *id = (*id).min(relation.relation_id))
        .or_insert(relation.relation_id);
    Ok(())
}
