//! V3 graph correspondence records selected identities, never live source authority.
mod binding;
#[cfg(test)]
mod tests;
use super::{
    ApplicationSigningEntityCoordinates, ApplicationSigningProvenanceParts,
    ApplicationSigningSubjectDependency,
};
use crate::application_query::RelationDirection;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationScalarValueBinding, ApplicationValueValidationDenial,
};
pub use binding::ApplicationSigningGraphSubjectProvenanceBinding;
use serde::{Deserialize, Serialize};

/// Descriptive native observation. None is meaningful only for a field whose
/// enclosing aspect was proven absent; it is not an unknown/missing observation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "presence", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApplicationSigningSubjectObservedFieldVersion {
    Present { native_revision: u64 },
    Absent { native_revision: Option<u64> },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningGraphSubjectDependency {
    pub source: ApplicationSigningEntityCoordinates,
    pub source_field: ApplicationFieldBindingLocus,
    pub captured_field: ApplicationFieldBindingLocus,
    pub value_binding: String,
    pub observed_version: ApplicationSigningSubjectObservedFieldVersion,
}
impl ApplicationSigningGraphSubjectDependency {
    pub fn maximum_encoded_bytes(&self) -> Option<usize> {
        ApplicationSigningSubjectDependency::quote(
            &self.source_field,
            &self.captured_field,
            &self.value_binding,
        )?
        .checked_add(128)
    }
}

/// One selected hop of an explicitly certified source path at the signing observation.
/// Native adjacency revisions are admission evidence, not an effectiveness clock.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningSubjectSelectionDependency {
    pub captured_field: ApplicationFieldBindingLocus,
    pub anchor: ApplicationSigningEntityCoordinates,
    pub selected: ApplicationSigningEntityCoordinates,
    pub relation: String,
    pub direction: RelationDirection,
    pub native_revision: u64,
}
impl ApplicationSigningSubjectSelectionDependency {
    pub fn maximum_encoded_bytes(&self) -> Option<usize> {
        Self::quote(&self.captured_field, &self.relation)
    }
    pub fn quote(field: &ApplicationFieldBindingLocus, relation: &str) -> Option<usize> {
        [field.entity(), field.aspect(), field.field(), relation]
            .into_iter()
            .try_fold(512usize, |sum, text| {
                sum.checked_add(text.len().checked_mul(6)?)
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningGraphSubjectProvenanceParts {
    pub authentication: ApplicationSigningProvenanceParts,
    pub dependencies: Vec<ApplicationSigningGraphSubjectDependency>,
    /// Grouped by captured field; each group's native path order is retained.
    pub selections: Vec<ApplicationSigningSubjectSelectionDependency>,
}
impl ApplicationSigningGraphSubjectProvenanceParts {
    pub fn maximum_encoded_bytes(&self) -> Option<usize> {
        let fields = self.dependencies.iter().try_fold(
            self.authentication
                .metadata
                .maximum_encoded_bytes()?
                .checked_add(128)?,
            |sum, dependency| sum.checked_add(dependency.maximum_encoded_bytes()?),
        )?;
        self.selections.iter().try_fold(fields, |sum, selection| {
            sum.checked_add(selection.maximum_encoded_bytes()?)
        })
    }
}

/// Distinct from v2: a copied or decoded value cannot satisfy native signing admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSigningGraphSubjectProvenance(ApplicationSigningGraphSubjectProvenanceParts);
impl ApplicationSigningGraphSubjectProvenance {
    pub fn from_parts(
        parts: ApplicationSigningGraphSubjectProvenanceParts,
    ) -> Result<Self, ApplicationValueValidationDenial> {
        let value = Self(parts);
        ApplicationSigningGraphSubjectProvenanceBinding::validate(&value)?;
        Ok(value)
    }
    pub fn parts(&self) -> &ApplicationSigningGraphSubjectProvenanceParts {
        &self.0
    }
    pub fn authentication(&self) -> &ApplicationSigningProvenanceParts {
        &self.0.authentication
    }
    pub fn dependencies(&self) -> &[ApplicationSigningGraphSubjectDependency] {
        &self.0.dependencies
    }
    pub fn selections(&self) -> &[ApplicationSigningSubjectSelectionDependency] {
        &self.0.selections
    }
}
