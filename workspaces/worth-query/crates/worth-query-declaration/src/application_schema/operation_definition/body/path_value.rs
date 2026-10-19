//! Computed writes select existing targets through native paths, not caller IDs.
mod authoring;
mod validation;
use super::{ApplicationOperationBodyPathFieldSource, ApplicationOperationBodyProgram};
use crate::{
    application_schema::ApplicationOperationProgramTarget,
    portable_identity::WorthQueryPortableTypeIdentity,
};
pub(super) use validation::validate;

/// Inert assignment meaning. Selection, value admission and publication remain native.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyPathValueAssignment {
    pub target: ApplicationOperationBodyPathFieldSource,
    pub binding: WorthQueryPortableTypeIdentity,
    pub value: String,
}
impl ApplicationOperationBodyPathValueAssignment {
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("path-value-assignment-v1;");
        for part in [
            self.target.canonical_record().as_str(),
            self.binding.as_str(),
            &self.value,
        ] {
            write!(&mut record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
    pub fn write_target(&self) -> ApplicationOperationProgramTarget {
        ApplicationOperationProgramTarget::Write {
            entity: self.target.field.entity().into(),
            aspect: self.target.field.aspect().into(),
            field: self.target.field.field().into(),
        }
    }
    pub(super) fn fits_body(&self, body: &ApplicationOperationBodyProgram) -> bool {
        self.target.has_supported_shape()
            && Some(self.target.path.start_entity()) == body.scope_entity()
            && self.target.presence == crate::application_schema::ApplicationFieldPresence::Required
            && !self.value.is_empty()
            && self.value.trim() == self.value
            && body
                .values()
                .iter()
                .any(|value| value.identity() == self.value && value.binding() == &self.binding)
            && !body
                .creation()
                .is_some_and(|creation| creation.signing().is_some())
    }
}
impl ApplicationOperationBodyProgram {
    pub(super) fn has_duplicate_path_destinations(&self) -> bool {
        let mut writes = std::collections::BTreeSet::new();
        self.path_value_assignments().iter().any(|step| {
            !writes.insert((&step.target.path, &step.target.field))
                || (step.target.path.steps().is_empty()
                    && self.scope_fields().iter().any(|scope| {
                        scope.write_target().is_some() && scope.field() == &step.target.field
                    }))
        })
    }
    pub(super) fn validate_path_bindings(
        &self,
    ) -> Result<(), super::ApplicationOperationBodyAuthoringDenial> {
        if self
            .path_input_expectations()
            .iter()
            .any(|guard| !guard.fits_body(self))
            || self
                .path_value_assignments()
                .iter()
                .any(|step| !step.fits_body(self))
        {
            return Err(super::ApplicationOperationBodyAuthoringDenial::IncompatibleSourceBinding);
        }
        if self.has_duplicate_path_destinations() {
            return Err(super::ApplicationOperationBodyAuthoringDenial::DuplicateAssignment);
        }
        Ok(())
    }
}
