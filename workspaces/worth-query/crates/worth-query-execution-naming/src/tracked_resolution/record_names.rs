//! One pass over the name index for the names many entity records carry.

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{
    LineageRecordTarget, PersistentNameOfRecord, PersistentNameSubjectKind,
};

use super::{ExecutionNameReadDenial, ExecutionNameReader, NameReadDependencies};
use crate::admission::{entity_scheme_subject, ObservedEntityNames};

impl ExecutionNameReader<'_> {
    /// Reads the name each of `entities` carries at the reader's basis,
    /// admitting the observation and holding the name index once for the
    /// whole set. Records each named record's lineage.
    ///
    /// A name born under a scheme the runtime has not installed, or under a
    /// scheme governing a relation or group, refuses the whole read.
    ///
    /// Complexity: O(entities) expected index probes; never O(names) or
    /// O(commits).
    pub fn observed_entity_names(
        &self,
        entities: impl IntoIterator<Item = EntityId>,
        dependencies: &mut NameReadDependencies,
    ) -> Result<ObservedEntityNames, ExecutionNameReadDenial> {
        let branch = self.branch().clone();
        self.with_view(|view| {
            let mut observed = ObservedEntityNames::default();
            for entity in entities {
                let record = LineageRecordTarget::Entity(entity);
                match view.name_of_record(record) {
                    PersistentNameOfRecord::Unnamed => observed.record_unnamed(entity),
                    PersistentNameOfRecord::Ended { .. } => observed.record_ended(entity),
                    PersistentNameOfRecord::Live { name, lineage } => {
                        if name.subject_kind() != PersistentNameSubjectKind::Entity {
                            return Err(ExecutionNameReadDenial::SubjectKindMismatch {
                                expected: PersistentNameSubjectKind::Entity,
                                actual: name.subject_kind(),
                            });
                        }
                        let scheme_id = view.scheme_of(name);
                        let installed = scheme_id
                            .and_then(|scheme| self.runtime.persistent_name_scheme_subject(scheme));
                        let (scheme_id, subject) = entity_scheme_subject(scheme_id, installed)?;
                        observed.record_named(entity, name, scheme_id, subject);
                        dependencies.record_live_record(&branch, lineage, record);
                    }
                }
            }
            Ok(observed)
        })?
    }
}
