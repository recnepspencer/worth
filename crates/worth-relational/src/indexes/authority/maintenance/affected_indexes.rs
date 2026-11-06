//! Dispatch installed maintainers from changed kinds, never from the index roster.
use super::{change_routing::ChangeRouting, changes::ChangedRecords, work::MaintenanceWork};
use crate::indexes::data::{DerivedIndexId, DerivedIndexMaintenanceDenialKind as Denial};
use crate::runtime::VisibilityProjectionView;
use std::collections::BTreeSet;

pub(super) fn select(
    runtime: &crate::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    changes: &ChangedRecords,
    before: &VisibilityProjectionView,
    after: &VisibilityProjectionView,
    work: &mut MaintenanceWork,
) -> Result<BTreeSet<DerivedIndexId>, Denial> {
    let schema = after
        .selected_schema_authority()
        .ok_or(Denial::CommitMismatch)?
        .schema_version();
    let routes = runtime
        .indexes
        .definition_read_binding()
        .snapshot()
        .routing(schema)
        .ok_or(Denial::ColdReconstructionRequired)?;
    if let Some(touches) = after
        .selected_root()
        .and_then(|root| root.canonical_envelope())
        .and_then(|envelope| envelope.descriptive_touches().exact_touches())
    {
        use crate::history::data::RelationalDescriptiveTouch as Touch;
        let mut indexes = BTreeSet::new();
        let mut add = |id| -> Result<(), Denial> {
            work.ordered::<DerivedIndexId, ()>(indexes.len(), 1, 0)?;
            indexes.insert(id);
            Ok(())
        };
        for touch in touches {
            match touch {
                Touch::IndexMembership {
                    index,
                    old_key,
                    new_key,
                    ..
                } if old_key != new_key => add(*index)?,
                Touch::EntityLifecycle { kind, .. } => {
                    for index in routes.entities(*kind) {
                        add(*index)?;
                    }
                }
                Touch::RelationLifecycle { kind, .. } => {
                    for index in routes.relations(*kind) {
                        add(*index)?;
                    }
                }
                Touch::RelationMembership { kind, .. } => {
                    for index in routes.topology(*kind, false) {
                        add(*index)?;
                    }
                }
                Touch::FieldRevision {
                    record,
                    kind,
                    aspect,
                    path,
                    ..
                } => {
                    if let [field] = path.fields() {
                        for index in routes.field(
                            matches!(record, crate::transactions::data::RecordRef::Entity(_)),
                            *kind,
                            aspect,
                            field,
                        ) {
                            add(*index)?;
                        }
                    }
                }
                _ => {}
            }
        }
        return Ok(indexes);
    }
    let changes = ChangeRouting::classify(execution, changes, Some(before), after, work)?;
    let mut indexes = BTreeSet::new();
    for (kind, entity) in changes.kinds() {
        work.lookup(routes.kind_count(), 1)?;
        let matched = if entity {
            routes.entities(kind)
        } else {
            routes.relations(kind)
        };
        for id in matched {
            work.ordered::<DerivedIndexId, ()>(indexes.len(), 1, 0)?;
            indexes.insert(*id);
        }
    }
    Ok(indexes)
}
