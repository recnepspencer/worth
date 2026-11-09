//! Record-local field dispatch from the exact sealed change description.
use super::work::MaintenanceWork;
use crate::history::data::RelationalDescriptiveTouch as Touch;
use crate::indexes::data::{
    DerivedIndexId, DerivedIndexMaintenanceDenialKind as Denial, InstalledIndexRouting,
};
use crate::transactions::data::RecordRef;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct FieldChanges(BTreeMap<RecordRef, BTreeSet<DerivedIndexId>>);
impl FieldChanges {
    pub(super) fn prepare(
        change: &crate::storage::substrate::maintenance::CommitChange,
        routes: &InstalledIndexRouting,
        work: &mut MaintenanceWork,
    ) -> Result<Option<Self>, Denial> {
        let Some(touches) = change.touches() else {
            return Ok(None);
        };
        let mut fields = Self(BTreeMap::new());
        for touch in touches {
            work.charge(1)?;
            let (record, indexes) = match touch {
                Touch::EntityLifecycle { entity, kind } => {
                    (RecordRef::Entity(*entity), routes.entities(*kind))
                }
                Touch::RelationLifecycle { relation, kind } => {
                    (RecordRef::Relation(*relation), routes.relations(*kind))
                }
                Touch::FieldRevision {
                    record,
                    kind,
                    aspect,
                    path,
                    ..
                } => {
                    let [field] = path.fields() else {
                        continue;
                    };
                    (
                        record.clone(),
                        routes.field(matches!(record, RecordRef::Entity(_)), *kind, aspect, field),
                    )
                }
                _ => continue,
            };
            for index in indexes {
                work.ordered::<RecordRef, BTreeSet<DerivedIndexId>>(fields.0.len(), 1, 0)?;
                let selected = fields.0.entry(record.clone()).or_default();
                work.ordered::<DerivedIndexId, ()>(selected.len(), 1, 0)?;
                selected.insert(*index);
            }
        }
        Ok(Some(fields))
    }
    pub(super) fn get(
        &self,
        record: RecordRef,
        work: &mut MaintenanceWork,
    ) -> Result<Option<&BTreeSet<DerivedIndexId>>, Denial> {
        work.lookup(self.0.len(), 1)?;
        Ok(self.0.get(&record))
    }
}
