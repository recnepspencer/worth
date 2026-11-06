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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pending: &mut [PendingField],
    routing: &crate::indexes::data::InstalledIndexRouting,
    patch_changes: Option<&ChangedRecords>,
    cold_changes: Option<&ChangedRecords>,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    if pending.is_empty() {
        return Ok(());
    }
    let mut positions = BTreeMap::new();
    for (position, field) in pending.iter().enumerate() {
        let index = match field {
            PendingField::Entity { index_id, .. } | PendingField::Relation { index_id, .. } => {
                *index_id
            }
        };
        work.ordered::<DerivedIndexId, usize>(positions.len(), 1, 0)?;
        positions.insert(index, position);
    }
    for (patch, changes, old) in [(true, patch_changes, before), (false, cold_changes, None)] {
        let Some(changes) = changes else { continue };
        refresh_entities(
            execution,
            pending,
            routing,
            &positions,
            patch,
            &changes.entities,
            old,
            after,
            work,
        )?;
        refresh_relations(
            execution,
            pending,
            routing,
            &positions,
            patch,
            &changes.relations,
            old,
            after,
            work,
        )?;
    }
    for field in pending {
        work.prepare(1, 0)?;
        match field {
            PendingField::Entity {
                entries,
                edits,
                patch,
                ..
            } => {
                grouped(
                    entries,
                    std::mem::take(edits),
                    Ord::cmp,
                    |key| key.canonical_value_bytes().len() + 1,
                    !*patch,
                    work,
                )?;
            }
            PendingField::Relation {
                entries,
                edits,
                patch,
                ..
            } => {
                grouped(
                    entries,
                    std::mem::take(edits),
                    Ord::cmp,
                    |key| key.canonical_value_bytes().len() + 1,
                    !*patch,
                    work,
                )?;
            }
        }
    }
    Ok(())
}

fn refresh_entities(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pending: &mut [PendingField],
    routing: &crate::indexes::data::InstalledIndexRouting,
    positions: &BTreeMap<DerivedIndexId, usize>,
    patch: bool,
    changed: &std::collections::BTreeSet<EntityId>,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for id in changed {
        let (old, new, old_kind, new_kind) = if work.prepares_allocations() {
            (
                None,
                None,
                super::record_metadata::entity_kind(execution, before, *id, work)?,
                super::record_metadata::entity_kind(execution, Some(after), *id, work)?,
            )
        } else {
            let old = reads::entity(execution, before, *id, work)?;
            let new = reads::entity(execution, Some(after), *id, work)?;
            let old_kind = old.as_ref().map(|record| record.kind.kind_id);
            let new_kind = new.as_ref().map(|record| record.kind.kind_id);
            (old, new, old_kind, new_kind)
        };
        for position in super::field_routing::positions(
            routing,
            positions,
            true,
            old_kind
                .into_iter()
                .chain(new_kind.filter(|kind| Some(*kind) != old_kind)),
            work,
        )? {
            let field = &mut pending[position];
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
            work.prepare(1, 0)?;
            let (old_key, new_key) = if work.prepares_allocations() {
                (
                    super::field_keys::entity(before, *id, locator, work)?,
                    super::field_keys::entity(Some(after), *id, locator, work)?,
                )
            } else {
                (
                    entity_key(before, old.as_ref(), locator),
                    entity_key(Some(after), new.as_ref(), locator),
                )
            };
            prepare_key_comparison(
                old_key.as_ref().map(|(key, _)| key),
                new_key.as_ref().map(|(key, _)| key),
                work,
            )?;
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pending: &mut [PendingField],
    routing: &crate::indexes::data::InstalledIndexRouting,
    positions: &BTreeMap<DerivedIndexId, usize>,
    patch: bool,
    changed: &std::collections::BTreeSet<RelationId>,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for id in changed {
        let (old, new, old_kind, new_kind) = if work.prepares_allocations() {
            (
                None,
                None,
                super::record_metadata::relation(execution, before, *id, work)?.map(|r| r.kind),
                super::record_metadata::relation(execution, Some(after), *id, work)?
                    .map(|r| r.kind),
            )
        } else {
            let old = reads::relation(execution, before, *id, work)?;
            let new = reads::relation(execution, Some(after), *id, work)?;
            let old_kind = old.as_ref().map(|record| record.kind.kind_id);
            let new_kind = new.as_ref().map(|record| record.kind.kind_id);
            (old, new, old_kind, new_kind)
        };
        for position in super::field_routing::positions(
            routing,
            positions,
            false,
            old_kind
                .into_iter()
                .chain(new_kind.filter(|kind| Some(*kind) != old_kind)),
            work,
        )? {
            let field = &mut pending[position];
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
            work.prepare(1, 0)?;
            let (old_key, new_key) = if work.prepares_allocations() {
                (
                    super::field_keys::relation(before, *id, locator, work)?,
                    super::field_keys::relation(Some(after), *id, locator, work)?,
                )
            } else {
                (
                    relation_key(before, old.as_ref(), locator),
                    relation_key(Some(after), new.as_ref(), locator),
                )
            };
            prepare_key_comparison(
                old_key.as_ref().map(|(key, _)| key),
                new_key.as_ref().map(|(key, _)| key),
                work,
            )?;
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
    work.ordered::<AuthoritativeFieldComparisonKey, Vec<PendingEdit<R>>>(
        edits.len(),
        key.canonical_value_bytes().len() + 1,
        0,
    )?;
    let operations = edits.entry(key).or_default();
    work.grow_vec(operations)?;
    operations.push(PendingEdit { row, insert });
    Ok(())
}

fn prepare_key_comparison(
    old: Option<&AuthoritativeFieldComparisonKey>,
    new: Option<&AuthoritativeFieldComparisonKey>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let width = match (old, new) {
        (Some(old), Some(new)) => {
            old.canonical_value_bytes()
                .len()
                .min(new.canonical_value_bytes().len())
                + 1
        }
        _ => 1,
    };
    work.prepare(width as u64, 0)
}

fn entity_key(
    view: Option<&VisibilityProjectionView>,
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
    view: Option<&VisibilityProjectionView>,
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
