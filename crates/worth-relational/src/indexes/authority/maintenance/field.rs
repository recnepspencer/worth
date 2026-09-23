use super::{
    changes::ChangedRecords,
    entry_edits::{grouped, PendingEdit},
    reads,
    work::MaintenanceWork,
};
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::{
    DerivedIndexEntries, DerivedIndexEntryMap, DerivedIndexId,
    DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::indexes::projected_field_values::{
    entity_index_projection_scope, relation_index_projection_scope, IndexProjectionSource,
};
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::{AuthoritativeFieldComparisonKey, EntityReadRecord, RelationReadRecord};
use crate::visibility::materialization::read_records::{
    entity_query_locus_comparison_key, relation_query_locus_comparison_key,
};
use std::collections::BTreeMap;
use worth_foundational::facade::AspectFieldLocator;

pub(super) enum PendingField {
    Entity {
        index_id: DerivedIndexId,
        locator: AspectFieldLocator,
        entries: DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, EntityId>,
        edits: BTreeMap<AuthoritativeFieldComparisonKey, Vec<PendingEdit<EntityId>>>,
        patch: bool,
    },
    Relation {
        index_id: DerivedIndexId,
        locator: AspectFieldLocator,
        entries: DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, RelationId>,
        edits: BTreeMap<AuthoritativeFieldComparisonKey, Vec<PendingEdit<RelationId>>>,
        patch: bool,
    },
}

impl PendingField {
    pub(super) fn into_prepared(self) -> (DerivedIndexId, DerivedIndexEntries) {
        match self {
            Self::Entity {
                index_id, entries, ..
            } => (index_id, DerivedIndexEntries::EntityField(entries)),
            Self::Relation {
                index_id, entries, ..
            } => (index_id, DerivedIndexEntries::RelationField(entries)),
        }
    }
}

pub(super) fn refresh(
    pending: &mut [PendingField],
    patch_changes: Option<&ChangedRecords>,
    cold_changes: Option<&ChangedRecords>,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for (patch, changes, old) in [(true, patch_changes, before), (false, cold_changes, None)] {
        let Some(changes) = changes else { continue };
        refresh_entities(pending, patch, &changes.entities, old, after, work)?;
        refresh_relations(pending, patch, &changes.relations, old, after, work)?;
    }
    for field in pending {
        match field {
            PendingField::Entity { entries, edits, .. } => {
                grouped(entries, std::mem::take(edits), Ord::cmp, work)?;
            }
            PendingField::Relation { entries, edits, .. } => {
                grouped(entries, std::mem::take(edits), Ord::cmp, work)?;
            }
        }
    }
    Ok(())
}

fn refresh_entities(
    pending: &mut [PendingField],
    patch: bool,
    changed: &std::collections::BTreeSet<EntityId>,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    if !pending
        .iter()
        .any(|field| matches!(field, PendingField::Entity { patch: lane, .. } if *lane == patch))
    {
        return Ok(());
    }
    for id in changed {
        let old = reads::entity(before, *id, work)?;
        let new = reads::entity(Some(after), *id, work)?;
        for field in pending.iter_mut() {
            let PendingField::Entity {
                locator,
                edits,
                patch: lane,
                ..
            } = field
            else {
                continue;
            };
            if *lane != patch {
                continue;
            }
            let old_key = entity_key(before, old.as_ref(), locator);
            let new_key = entity_key(Some(after), new.as_ref(), locator);
            if old_key == new_key {
                continue;
            }
            if let Some((key, record_id)) = old_key {
                queue(edits, key, record_id, false, work)?;
            }
            if let Some((key, record_id)) = new_key {
                queue(edits, key, record_id, true, work)?;
            }
        }
    }
    Ok(())
}

fn refresh_relations(
    pending: &mut [PendingField],
    patch: bool,
    changed: &std::collections::BTreeSet<RelationId>,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    if !pending
        .iter()
        .any(|field| matches!(field, PendingField::Relation { patch: lane, .. } if *lane == patch))
    {
        return Ok(());
    }
    for id in changed {
        let old = reads::relation(before, *id, work)?;
        let new = reads::relation(Some(after), *id, work)?;
        for field in pending.iter_mut() {
            let PendingField::Relation {
                locator,
                edits,
                patch: lane,
                ..
            } = field
            else {
                continue;
            };
            if *lane != patch {
                continue;
            }
            let old_key = relation_key(before, old.as_ref(), locator);
            let new_key = relation_key(Some(after), new.as_ref(), locator);
            if old_key == new_key {
                continue;
            }
            if let Some((key, record_id)) = old_key {
                queue(edits, key, record_id, false, work)?;
            }
            if let Some((key, record_id)) = new_key {
                queue(edits, key, record_id, true, work)?;
            }
        }
    }
    Ok(())
}

fn queue<R>(
    edits: &mut BTreeMap<AuthoritativeFieldComparisonKey, Vec<PendingEdit<R>>>,
    key: AuthoritativeFieldComparisonKey,
    row: R,
    insert: bool,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    work.charge(1)?;
    edits
        .entry(key)
        .or_default()
        .push(PendingEdit { row, insert });
    Ok(())
}

fn entity_key(
    view: Option<&VisibilityProjectionView<'_>>,
    record: Option<&EntityReadRecord>,
    locator: &AspectFieldLocator,
) -> Option<(AuthoritativeFieldComparisonKey, EntityId)> {
    let (Some(view), Some(record)) = (view, record) else {
        return None;
    };
    let source = IndexProjectionSource::selected(view);
    source
        .entity_aspect_plan(record.kind.kind_id)
        .and_then(|plan| entity_index_projection_scope(plan, locator))
        .and_then(|_| entity_query_locus_comparison_key(record, locator))
        .map(|key| (key, record.entity_id))
}

fn relation_key(
    view: Option<&VisibilityProjectionView<'_>>,
    record: Option<&RelationReadRecord>,
    locator: &AspectFieldLocator,
) -> Option<(AuthoritativeFieldComparisonKey, RelationId)> {
    let (Some(view), Some(record)) = (view, record) else {
        return None;
    };
    let source = IndexProjectionSource::selected(view);
    source
        .relation_aspect_plan(record.kind.kind_id)
        .and_then(|plan| relation_index_projection_scope(plan, locator))
        .and_then(|_| relation_query_locus_comparison_key(record, locator))
        .map(|key| (key, record.relation_id))
}
