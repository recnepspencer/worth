use super::root_view::IndexRootView;
use super::{change_routing::ChangeRouting, join, ordering, work::MaintenanceWork};
use crate::indexes::data::{
    DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexKind,
    DerivedIndexMaintenanceDenialKind,
};
pub(super) fn empty_entries(kind: &DerivedIndexKind) -> DerivedIndexEntries {
    match kind {
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
    }
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
