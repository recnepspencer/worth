//! Installation-facing projections of retained portable operation meaning.

use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;

use super::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableOperationGraphReadScope, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily,
};

/// Borrowed canonical-position view over one exported logical record.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryPortablePackageRecordView<'a> {
    canonical_index: u32,
    record: &'a WorthQueryPortablePackageRecord,
}

impl<'a> WorthQueryPortablePackageRecordView<'a> {
    pub(crate) const fn new(
        canonical_index: u32,
        record: &'a WorthQueryPortablePackageRecord,
    ) -> Self {
        Self {
            canonical_index,
            record,
        }
    }

    pub const fn canonical_index(self) -> u32 {
        self.canonical_index
    }

    pub const fn family(self) -> WorthQueryPortablePackageRecordFamily {
        self.record.family()
    }

    pub const fn record(self) -> &'a WorthQueryPortablePackageRecord {
        self.record
    }
}

impl WorthQueryPortableApplicationOperationContractRecord {
    pub(crate) fn decision_read_targets(
        &self,
    ) -> Result<Vec<ApplicationOperationDecisionReadTarget>, ()> {
        let mut targets = Vec::new();
        for read in self.graph_reads() {
            match read {
                WorthQueryPortableOperationGraphReadScope::Entity { entity, .. } => {
                    targets.push(ApplicationOperationDecisionReadTarget::Entity {
                        entity: entity.clone(),
                    });
                }
                WorthQueryPortableOperationGraphReadScope::NativeProjection {
                    entity,
                    aspect,
                    mask,
                    ..
                } => {
                    for path in mask.paths() {
                        let [field] = path.fields() else {
                            return Err(());
                        };
                        targets.push(ApplicationOperationDecisionReadTarget::Field {
                            entity: entity.clone(),
                            aspect: aspect.as_str().to_owned(),
                            field: field.as_str().to_owned(),
                        });
                    }
                }
                WorthQueryPortableOperationGraphReadScope::Relation {
                    relation, from, to, ..
                } => targets.push(ApplicationOperationDecisionReadTarget::Relation {
                    relation: relation.clone(),
                    from: from.clone(),
                    to: to.clone(),
                }),
            }
        }
        Ok(targets)
    }

    pub(crate) fn authored_program_width(&self) -> Option<usize> {
        self.touches().len().checked_add(self.emissions().len())
    }
}
