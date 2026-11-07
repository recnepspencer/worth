use super::root_view::IndexRootView;
use super::{change_routing::ChangeRouting, join, ordering, work::MaintenanceWork};
use crate::indexes::data::{
    DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexKind,
    DerivedIndexMaintenanceDenialKind,
};
pub(super) fn empty_entries(
    kind: &DerivedIndexKind,
    work: &mut MaintenanceWork,
) -> Result<DerivedIndexEntries, DerivedIndexMaintenanceDenialKind> {
    use super::persistent_allocation::prepare_empty_map;
    use crate::identity::data::{EntityId, RelationId};
    use crate::indexes::data::{RelatedEntityOrderingEntry, RelationJoinEntry, RelationJoinKey};
    use crate::storage::data::AuthoritativeFieldComparisonKey as Key;
    match kind {
        DerivedIndexKind::EntityField { .. } => prepare_empty_map::<Key, EntityId>(work)?,
        DerivedIndexKind::RelationField { .. } => prepare_empty_map::<Key, RelationId>(work)?,
        DerivedIndexKind::RelatedEntityOrdering { .. } => {
            prepare_empty_map::<EntityId, RelatedEntityOrderingEntry>(work)?
        }
        DerivedIndexKind::RelationJoin(_) => {
            prepare_empty_map::<RelationJoinKey, RelationJoinEntry>(work)?;
            let bytes = super::cold_projection::funding::tree_bytes::<(
                EntityId,
                im::OrdMap<EntityId, im::OrdSet<RelationId>>,
            )>(0)?
                + super::cold_projection::funding::tree_bytes::<(RelationId, (EntityId, EntityId))>(
                    0,
                )?;
            work.prepare(
                4,
                bytes * 2
                    + (std::mem::size_of::<crate::indexes::data::JoinLegsState>()
                        + 2 * std::mem::size_of::<usize>()) as u64,
            )?;
        }
    }
    Ok(match kind {
        DerivedIndexKind::EntityField { .. } => {
            DerivedIndexEntries::EntityField(Default::default())
        }
        DerivedIndexKind::RelationField { .. } => {
            DerivedIndexEntries::RelationField(Default::default())
        }
        DerivedIndexKind::RelatedEntityOrdering { .. } => {
            DerivedIndexEntries::RelatedEntityOrdering(Default::default())
        }
        DerivedIndexKind::RelationJoin(_) => {
            DerivedIndexEntries::RelationJoin(crate::indexes::data::RelationJoinIndex::maintained(
                Default::default(),
                Default::default(),
            ))
        }
    })
}

pub(super) fn update_entries(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    definition: &DerivedIndexDefinition,
    entries: &mut DerivedIndexEntries,
    changes: &ChangeRouting,
    before: Option<&IndexRootView>,
    after: &IndexRootView,
    work: &mut MaintenanceWork,
) -> Result<bool, DerivedIndexMaintenanceDenialKind> {
    match (&definition.kind, entries) {
        (
            DerivedIndexKind::RelatedEntityOrdering {
                relation_kind,
                parent_endpoint,
                child_kind,
                ordering,
            },
            DerivedIndexEntries::RelatedEntityOrdering(entries),
        ) => {
            let edits = work.counts().entry_edits;
            ordering::refresh(
                execution,
                entries,
                (*relation_kind, *parent_endpoint, *child_kind, ordering),
                changes,
                before,
                after,
                work,
            )?;
            Ok(work.counts().entry_edits != edits)
        }
        (DerivedIndexKind::RelationJoin(join), DerivedIndexEntries::RelationJoin(entries)) => {
            join::refresh(execution, entries, *join, changes, before, after, work)
        }
        _ => Err(DerivedIndexMaintenanceDenialKind::GenerationKindMismatch(
            definition.index_id,
        )),
    }
}
