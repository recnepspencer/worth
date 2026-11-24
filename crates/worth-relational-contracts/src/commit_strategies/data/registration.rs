use std::fmt;

use serde::{Deserialize, Serialize};

use super::descriptor::CommitStrategyDescriptor;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitStrategyRegistration {
    descriptor: CommitStrategyDescriptor,
}

impl CommitStrategyRegistration {
    pub fn new(
        descriptor: CommitStrategyDescriptor,
    ) -> Result<Self, CommitStrategyRegistrationError> {
        validate_non_empty("semantic_name", descriptor.semantic_name().as_str())?;
        validate_non_empty("family_name", descriptor.family_name().as_str())?;
        validate_non_empty("intent_name", descriptor.intent_name().as_str())?;
        validate_non_empty("input_schema_name", descriptor.input_schema_name().as_str())?;
        validate_non_empty(
            "output_schema_name",
            descriptor.output_schema_name().as_str(),
        )?;
        validate_non_empty("artifact_name", descriptor.artifact_name().as_str())?;
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &CommitStrategyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitStrategyRegistrationError {
    field: &'static str,
}

impl fmt::Display for CommitStrategyRegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "commit strategy registration field `{}` must not be empty",
            self.field
        )
    }
}

impl std::error::Error for CommitStrategyRegistrationError {}

fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), CommitStrategyRegistrationError> {
    if value.trim().is_empty() {
        return Err(CommitStrategyRegistrationError { field });
    }
    Ok(())
}
