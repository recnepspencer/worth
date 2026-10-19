//! Captured input guards select graph facts through the existing native path vocabulary.
mod authoring;
use super::{ApplicationOperationBodyPathFieldSource, ApplicationOperationBodyProgram};
use crate::{
    application_schema::{
        ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
use worth_foundational::facade::ScalarAspectType;

/// Inert guard meaning. Installed readers, codecs and publication retain actual authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyPathInputExpectation {
    pub source: ApplicationOperationBodyPathFieldSource,
    pub binding: WorthQueryPortableTypeIdentity,
    pub scalar_family: ScalarAspectType,
    pub member: String,
}
impl ApplicationOperationBodyPathInputExpectation {
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("path-input-expectation-v1;");
        for part in [
            self.source.canonical_record().as_str(),
            self.binding.as_str(),
            self.scalar_family.canonical_name(),
            &self.member,
        ] {
            write!(&mut record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
    pub(super) fn fits_body(&self, body: &ApplicationOperationBodyProgram) -> bool {
        self.source.has_supported_shape()
            && Some(self.source.path.start_entity()) == body.scope_entity()
            && !self.member.is_empty()
            && self.member.trim() == self.member
    }
}
pub(super) fn validate(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[Member],
) -> Result<(), Denial> {
    for guard in body.path_input_expectations() {
        if !guard.fits_body(body)
            || !guard.source.has_closed_reads(operation, members)
            || !members.iter().any(|member| {
                matches!(member, Member::Field { entity, aspect, field,
                value_type, scalar_family, presence, .. } if entity == guard.source.field.entity()
                    && aspect == guard.source.field.aspect() && field == guard.source.field.field()
                    && value_type == guard.binding.as_str() && scalar_family == &guard.scalar_family
                    && presence == &guard.source.presence)
            })
        {
            return Err(Denial::MissingOperationDecisionReadDependency);
        }
    }
    Ok(())
}
