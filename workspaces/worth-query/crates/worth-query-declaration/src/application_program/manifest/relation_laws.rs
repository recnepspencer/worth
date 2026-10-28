//! Relation-law records participate in the same canonical program manifest.
use super::{framed_record, ApplicationProgramManifest};
use crate::application_schema::ApplicationRelationLawMeaning;

impl ApplicationProgramManifest {
    pub(in crate::application_program) fn with_relation_laws(
        mut self,
        laws: &[ApplicationRelationLawMeaning],
    ) -> Self {
        let mut records = self.records.into_vec();
        records.extend(
            laws.iter()
                .map(|law| framed_record("relation-laws-v1", law.canonical_fields())),
        );
        records.sort_unstable();
        self.records = records.into_boxed_slice();
        self
    }
}
