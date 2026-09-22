use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::{
    DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial, RelationJoinDefinition,
    RelationJoinEntry, RelationJoinKey, RelationJoinLeg, RelationJoinSharedEndpoint,
};
use crate::indexes::projected_field_values::join_endpoints;
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::RelationReadRecord;

use super::{changes::ChangedRecords, entry_edits::edit, reads, work::MaintenanceWork};

pub(super) fn refresh(
    entries: &mut DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>,
    definition: RelationJoinDefinition,
    changes: &ChangedRecords,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let mut affected = BTreeSet::new();
    for relation_id in &changes.relations {
        for view in before.into_iter().chain(std::iter::once(after)) {
            let Some(relation) = reads::relation(Some(view), *relation_id, work)? else {
                continue;
            };
            for leg in [definition.left(), definition.right()] {
                if relation.kind.kind_id == leg.relation_kind() {
                    work.charge(1)?;
                    affected.insert(join_endpoints(&relation, leg.shared_endpoint()).0);
                }
            }
        }
    }
    for entity_id in &changes.entities {
        let old_kind = reads::entity(before, *entity_id, work)?.map(|record| record.kind.kind_id);
        let new_kind =
            reads::entity(Some(after), *entity_id, work)?.map(|record| record.kind.kind_id);
        if old_kind == new_kind {
            continue;
        }
        if old_kind == Some(definition.shared_entity_kind())
            || new_kind == Some(definition.shared_entity_kind())
        {
            work.charge(1)?;
            affected.insert(*entity_id);
        }
        for leg in [definition.left(), definition.right()] {
            if old_kind != Some(leg.external_entity_kind())
                && new_kind != Some(leg.external_entity_kind())
            {
                continue;
            }
            for view in before.into_iter().chain(std::iter::once(after)) {
                let outgoing = leg.shared_endpoint() == RelationJoinSharedEndpoint::Target;
                for relation in
                    reads::adjacency(view, *entity_id, leg.relation_kind(), outgoing, work)?
                {
                    work.charge(1)?;
                    affected.insert(join_endpoints(&relation, leg.shared_endpoint()).0);
                }
            }
        }
    }
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

fn rows_for_shared(
    view: Option<&VisibilityProjectionView<'_>>,
    shared: EntityId,
    definition: RelationJoinDefinition,
    work: &mut MaintenanceWork,
) -> Result<BTreeMap<RelationJoinKey, RelationJoinEntry>, Denial> {
    let Some(view) = view else {
        return Ok(BTreeMap::new());
    };
    if reads::entity(Some(view), shared, work)?.map(|record| record.kind.kind_id)
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
    if reads::entity(Some(view), external, work)?.map(|record| record.kind.kind_id)
        != Some(leg.external_entity_kind())
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
