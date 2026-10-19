//! Durable association of certified source facts with the immutable fields that captured them.
use super::{ApplicationSigningEntityCoordinates, ApplicationSigningProvenanceParts};
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationScalarValueBinding, ApplicationValueValidationDenial,
};
use serde::{Deserialize, Serialize};
mod binding;
#[cfg(test)]
mod tests;
pub use binding::ApplicationSigningSubjectProvenanceBinding;

/// Descriptive correspondence, not a source observation, mutation permit or effectiveness decision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningSubjectDependency {
    pub source: ApplicationSigningEntityCoordinates,
    pub source_field: ApplicationFieldBindingLocus,
    pub captured_field: ApplicationFieldBindingLocus,
    pub value_binding: String,
    /// Native source field revision at the signing observation; never a local execution counter.
    pub native_revision: u64,
}
impl ApplicationSigningSubjectDependency {
    /// JSON/storage admission quote from the actual installed correspondence.
    pub fn maximum_encoded_bytes(&self) -> Option<usize> {
        Self::quote(
            &self.source_field,
            &self.captured_field,
            &self.value_binding,
        )
    }
    pub fn quote(
        source: &ApplicationFieldBindingLocus,
        captured: &ApplicationFieldBindingLocus,
        binding: &str,
    ) -> Option<usize> {
        [
            source.entity(),
            source.aspect(),
            source.field(),
            captured.entity(),
            captured.aspect(),
            captured.field(),
            binding,
        ]
        .into_iter()
        .try_fold(512usize, |total, text| {
            total.checked_add(text.len().checked_mul(6)?)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningSubjectProvenanceParts {
    pub authentication: ApplicationSigningProvenanceParts,
    pub dependencies: Vec<ApplicationSigningSubjectDependency>,
}
impl ApplicationSigningSubjectProvenanceParts {
    pub fn maximum_encoded_bytes(&self) -> Option<usize> {
        self.dependencies.iter().try_fold(
            self.authentication
                .metadata
                .maximum_encoded_bytes()?
                .checked_add(128)?,
            |total, dependency| total.checked_add(dependency.maximum_encoded_bytes()?),
        )
    }
}

/// V2 signed-subject data. The original v1 carrier remains a distinct, supported format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSigningSubjectProvenance(ApplicationSigningSubjectProvenanceParts);
impl ApplicationSigningSubjectProvenance {
    pub fn from_parts(
        parts: ApplicationSigningSubjectProvenanceParts,
    ) -> Result<Self, ApplicationValueValidationDenial> {
        let value = Self(parts);
        ApplicationSigningSubjectProvenanceBinding::validate(&value)?;
        Ok(value)
    }
    pub fn parts(&self) -> &ApplicationSigningSubjectProvenanceParts {
        &self.0
    }
    pub fn authentication(&self) -> &ApplicationSigningProvenanceParts {
        &self.0.authentication
    }
    pub fn dependencies(&self) -> &[ApplicationSigningSubjectDependency] {
        &self.0.dependencies
    }
}
