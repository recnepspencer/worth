//! Immutable business consequence data, never reusable mutation or signing authority.
use super::{ApplicationFieldBindingLocus, ApplicationSigningEntityCoordinates};
use serde::{Deserialize, Serialize};
mod binding;
#[cfg(test)]
mod tests;
pub use binding::ApplicationSigningSupersessionBinding;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningSupersessionParts {
    pub evidence: ApplicationSigningEntityCoordinates,
    pub subject: ApplicationSigningEntityCoordinates,
    pub actor: ApplicationSigningEntityCoordinates,
    pub operation: String,
    pub logical_request: [u8; 32],
    pub changed_dependencies: Vec<ApplicationFieldBindingLocus>,
}
impl ApplicationSigningSupersessionParts {
    pub fn maximum_encoded_bytes(&self) -> Option<usize> {
        Self::quote(&self.operation, &self.changed_dependencies)
    }
    pub fn quote(operation: &str, dependencies: &[ApplicationFieldBindingLocus]) -> Option<usize> {
        dependencies.iter().try_fold(
            2048usize.checked_add(operation.len().checked_mul(6)?)?,
            |size, field| {
                [field.entity(), field.aspect(), field.field()]
                    .into_iter()
                    .try_fold(size.checked_add(128)?, |size, text| {
                        size.checked_add(text.len().checked_mul(6)?)
                    })
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSigningSupersession(ApplicationSigningSupersessionParts);
impl ApplicationSigningSupersession {
    pub fn from_parts(
        parts: ApplicationSigningSupersessionParts,
    ) -> Result<Self, super::ApplicationValueValidationDenial> {
        use super::ApplicationScalarValueBinding;
        let value = Self(parts);
        ApplicationSigningSupersessionBinding::validate(&value)?;
        Ok(value)
    }
    pub fn parts(&self) -> &ApplicationSigningSupersessionParts {
        &self.0
    }
}
