use super::{change_routing::ChangeRouting, join, ordering, work::MaintenanceWork};
use crate::indexes::data::{
    DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexKind,
    DerivedIndexMaintenanceDenialKind,
};
use crate::runtime::VisibilityProjectionView;
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
        DerivedIndexKind::RelationJoin(_) => DerivedIndexEntries::RelationJoin(Default::default()),
    }
}

pub(super) fn update_entries(
    definition: &DerivedIndexDefinition,
    entries: &mut DerivedIndexEntries,
    changes: &ChangeRouting,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
    work: &mut MaintenanceWork,
) -> Result<(), DerivedIndexMaintenanceDenialKind> {
    match (&definition.kind, entries) {
        (
            DerivedIndexKind::RelatedEntityOrdering {
                relation_kind,
                parent_endpoint,
                child_kind,
                ordering,
            },
            DerivedIndexEntries::RelatedEntityOrdering(entries),
        ) => ordering::refresh(
            entries,
            (*relation_kind, *parent_endpoint, *child_kind, ordering),
            changes,
            before,
            after,
            work,
        ),
        (DerivedIndexKind::RelationJoin(join), DerivedIndexEntries::RelationJoin(entries)) => {
            join::refresh(entries, *join, changes, before, after, work)
        }
        _ => Err(DerivedIndexMaintenanceDenialKind::GenerationKindMismatch(
            definition.index_id,
        )),
    }
}
