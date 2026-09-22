use super::{changes::ChangedRecords, entry_edits::edit, reads, work::MaintenanceWork};
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::{DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial};
use crate::indexes::projected_field_values::{
    entity_index_projection_scope, relation_index_projection_scope, IndexProjectionSource,
};
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::{
    entity_query_locus_comparison_key, relation_query_locus_comparison_key,
};
use worth_foundational::facade::AspectFieldLocator;

pub(super) fn entities(
    entries: &mut DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, EntityId>,
    locator: &AspectFieldLocator,
    changes: &ChangedRecords,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for id in &changes.entities {
        let old = entity_key(before, *id, locator, work)?;
        let new = entity_key(Some(after), *id, locator, work)?;
        if old == new {
            continue;
        }
        if let Some((key, record_id)) = old {
            edit(entries, key, record_id, false, Ord::cmp, work)?;
        }
        if let Some((key, record_id)) = new {
            edit(entries, key, record_id, true, Ord::cmp, work)?;
        }
    }
    Ok(())
}

fn entity_key(
    view: Option<&VisibilityProjectionView<'_>>,
    id: EntityId,
    locator: &AspectFieldLocator,
    work: &mut MaintenanceWork,
) -> Result<Option<(AuthoritativeFieldComparisonKey, EntityId)>, Denial> {
    let Some(record) = reads::entity(view, id, work)? else {
        return Ok(None);
    };
    let source = IndexProjectionSource::selected(view.unwrap());
    Ok(source
        .entity_aspect_plan(record.kind.kind_id)
        .and_then(|plan| entity_index_projection_scope(plan, locator))
        .and_then(|_| entity_query_locus_comparison_key(&record, locator))
        .map(|key| (key, record.entity_id)))
}

pub(super) fn relations(
    entries: &mut DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, RelationId>,
    locator: &AspectFieldLocator,
    changes: &ChangedRecords,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for id in &changes.relations {
        let old = relation_key(before, *id, locator, work)?;
        let new = relation_key(Some(after), *id, locator, work)?;
        if old == new {
            continue;
        }
        if let Some((key, record_id)) = old {
            edit(entries, key, record_id, false, Ord::cmp, work)?;
        }
        if let Some((key, record_id)) = new {
            edit(entries, key, record_id, true, Ord::cmp, work)?;
        }
    }
    Ok(())
}

fn relation_key(
    view: Option<&VisibilityProjectionView<'_>>,
    id: RelationId,
    locator: &AspectFieldLocator,
    work: &mut MaintenanceWork,
) -> Result<Option<(AuthoritativeFieldComparisonKey, RelationId)>, Denial> {
    let Some(record) = reads::relation(view, id, work)? else {
        return Ok(None);
    };
    let source = IndexProjectionSource::selected(view.unwrap());
    Ok(source
        .relation_aspect_plan(record.kind.kind_id)
        .and_then(|plan| relation_index_projection_scope(plan, locator))
        .and_then(|_| relation_query_locus_comparison_key(&record, locator))
        .map(|key| (key, record.relation_id)))
}
