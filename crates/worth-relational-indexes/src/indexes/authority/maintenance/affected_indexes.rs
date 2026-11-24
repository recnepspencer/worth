//! Dispatch installed maintainers from changed kinds, never from the index roster.
use super::root_view::IndexRootView;
use super::{change_routing::ChangeRouting, changes::ChangedRecords, work::MaintenanceWork};
use crate::contracts::indexes::data::definition::DerivedIndexId;
use crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind as Denial;
use std::collections::BTreeSet;

pub(super) fn select(
    routes: &crate::contracts::indexes::data::installed_routing::InstalledIndexRouting,
    change: &worth_relational_branch::internal_port::branch::maintenance::CommitChange,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    changes: &ChangedRecords,
    before: &IndexRootView,
    after: &IndexRootView,
    work: &mut MaintenanceWork,
) -> Result<BTreeSet<DerivedIndexId>, Denial> {
    if let Some(touches) = change.touches() {
        use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouch as Touch;
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
                            matches!(
                                record,
                                crate::contracts::transactions::data::primitives::RecordRef::Entity(
                                    _
                                )
                            ),
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
