use super::root_view::IndexRootView;
use super::{
    changes::ChangedRecords,
    entry_edits::{grouped, PendingEdit},
    work::MaintenanceWork,
};
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::{
    DerivedIndexEntries, DerivedIndexEntryMap, DerivedIndexId,
    DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectFieldLocator;

pub(super) enum PendingField {
    Entity {
        index_id: DerivedIndexId,
        locator: AspectFieldLocator,
        entries: DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, EntityId>,
        edits: BTreeMap<AuthoritativeFieldComparisonKey, Vec<PendingEdit<EntityId>>>,
        patch: bool,
        changed: bool,
    },
    Relation {
        index_id: DerivedIndexId,
        locator: AspectFieldLocator,
        entries: DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, RelationId>,
        edits: BTreeMap<AuthoritativeFieldComparisonKey, Vec<PendingEdit<RelationId>>>,
        patch: bool,
        changed: bool,
    },
}

impl PendingField {
    pub(super) fn into_prepared(self) -> (DerivedIndexId, DerivedIndexEntries, bool) {
        match self {
            Self::Entity {
                index_id,
                entries,
                changed,
                ..
            } => (index_id, DerivedIndexEntries::EntityField(entries), changed),
            Self::Relation {
                index_id,
                entries,
                changed,
                ..
            } => (
                index_id,
                DerivedIndexEntries::RelationField(entries),
                changed,
            ),
        }
    }
}

pub(super) fn refresh(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pending: &mut [PendingField],
    routing: &crate::indexes::data::InstalledIndexRouting,
    patch_changes: Option<&ChangedRecords>,
    cold_changes: Option<&ChangedRecords>,
    before: Option<&IndexRootView>,
    after: &IndexRootView,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    if pending.is_empty() {
        return Ok(());
    }
    let exact = after
        .root()
        .commit_change()
        .map(|change| super::field_changes::FieldChanges::prepare(change, routing, work))
        .transpose()?
        .flatten();
    collect_changes(
        execution,
        pending,
        routing,
        patch_changes,
        cold_changes,
        before,
        after,
        exact.as_ref(),
        work,
    )?;
    apply_changes(pending, work)
}

/// Applied and committed images share the same routing and key difference owner.
pub(super) fn collect_changes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pending: &mut [PendingField],
    routing: &crate::indexes::data::InstalledIndexRouting,
    patch_changes: Option<&ChangedRecords>,
    cold_changes: Option<&ChangedRecords>,
    before: Option<&IndexRootView>,
    after: &IndexRootView,
    exact: Option<&super::field_changes::FieldChanges>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
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
            if patch { exact } else { None },
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
            if patch { exact } else { None },
            patch,
            &changes.relations,
            old,
            after,
            work,
        )?;
    }
    Ok(())
}

pub(super) fn apply_changes(
    pending: &mut [PendingField],
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for field in pending {
        work.prepare(1, 0)?;
        match field {
            PendingField::Entity {
                entries,
                edits,
                patch,
                changed,
                ..
            } => {
                *changed = !edits.is_empty();
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
                changed,
                ..
            } => {
                *changed = !edits.is_empty();
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
    exact: Option<&super::field_changes::FieldChanges>,
    patch: bool,
    changed: &std::collections::BTreeSet<EntityId>,
    before: Option<&IndexRootView>,
    after: &IndexRootView,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for id in changed {
        let old_kind = super::record_metadata::entity_kind(execution, before, *id, work)?;
        let new_kind = super::record_metadata::entity_kind(execution, Some(after), *id, work)?;
        for position in super::field_routing::positions(
            routing,
            positions,
            exact,
            crate::transactions::data::RecordRef::Entity(*id),
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
            super::field_delta::entity_keys(execution, *id, locator, before, after, edits, work)?;
        }
    }
    Ok(())
}

fn refresh_relations(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pending: &mut [PendingField],
    routing: &crate::indexes::data::InstalledIndexRouting,
    positions: &BTreeMap<DerivedIndexId, usize>,
    exact: Option<&super::field_changes::FieldChanges>,
    patch: bool,
    changed: &std::collections::BTreeSet<RelationId>,
    before: Option<&IndexRootView>,
    after: &IndexRootView,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for id in changed {
        let old_kind =
            super::record_metadata::relation(execution, before, *id, work)?.map(|r| r.kind);
        let new_kind =
            super::record_metadata::relation(execution, Some(after), *id, work)?.map(|r| r.kind);
        for position in super::field_routing::positions(
            routing,
            positions,
            exact,
            crate::transactions::data::RecordRef::Relation(*id),
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
            let (old_key, new_key) = (
                super::field_keys::relation(execution, before, *id, locator, work)?,
                super::field_keys::relation(execution, Some(after), *id, locator, work)?,
            );
            prepare_key_comparison(
                old_key.as_ref().map(|(key, _)| key),
                new_key.as_ref().map(|(key, _)| key),
                work,
            )?;
            if old_key == new_key {
                continue;
            }
            if let Some((key, record_id)) = old_key {
                super::field_delta::queue(edits, key, record_id, false, work)?;
            }
            if let Some((key, record_id)) = new_key {
                super::field_delta::queue(edits, key, record_id, true, work)?;
            }
        }
    }
    Ok(())
}

pub(super) fn prepare_key_comparison(
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
