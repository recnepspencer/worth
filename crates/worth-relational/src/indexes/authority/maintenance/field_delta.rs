//! One applied image updates data entries through the ordinary field-key owner.
use super::{
    entry_edits::{grouped, PendingEdit},
    root_view::IndexRootView,
    work::MaintenanceWork,
};
use crate::{
    identity::data::EntityId,
    indexes::data::{DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial},
    storage::data::AuthoritativeFieldComparisonKey,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::AspectFieldLocator;

pub(super) fn entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entries: &mut DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, EntityId>,
    locator: &AspectFieldLocator,
    before: &IndexRootView,
    after: &IndexRootView,
    changed: &[EntityId],
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let mut seen = BTreeSet::new();
    let mut edits = BTreeMap::new();
    for id in changed {
        work.ordered::<EntityId, ()>(seen.len(), 1, 0)?;
        if !seen.insert(*id) {
            continue;
        }
        entity_keys(
            execution,
            *id,
            locator,
            Some(before),
            after,
            &mut edits,
            work,
        )?;
    }
    grouped(
        entries,
        edits,
        Ord::cmp,
        |key| key.canonical_value_bytes().len() + 1,
        false,
        work,
    )
}

pub(super) fn entity_keys(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    id: EntityId,
    locator: &AspectFieldLocator,
    before: Option<&IndexRootView>,
    after: &IndexRootView,
    edits: &mut BTreeMap<AuthoritativeFieldComparisonKey, Vec<PendingEdit<EntityId>>>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let old = super::field_keys::entity(execution, before, id, locator, work)?;
    let new = super::field_keys::entity(execution, Some(after), id, locator, work)?;
    super::field::prepare_key_comparison(
        old.as_ref().map(|(key, _)| key),
        new.as_ref().map(|(key, _)| key),
        work,
    )?;
    if old == new {
        return Ok(());
    }
    if let Some((key, id)) = old {
        queue(edits, key, id, false, work)?;
    }
    if let Some((key, id)) = new {
        queue(edits, key, id, true, work)?;
    }
    Ok(())
}

pub(super) fn queue<R>(
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
