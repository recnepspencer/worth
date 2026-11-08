//! Basic installed rule identity checks, before executable erasure.
use super::{CustomInvariantDescriptor, CustomInvariantRegistration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomInvariantRegistrationError {
    EmptyRuleId,
    EmptyDisplayName,
    AggregateExecutionPoint,
}

impl CustomInvariantRegistration {
    pub(super) fn validate_descriptor(
        descriptor: &CustomInvariantDescriptor,
    ) -> Result<(), CustomInvariantRegistrationError> {
        if descriptor.identity.rule_id.as_str().trim().is_empty() {
            return Err(CustomInvariantRegistrationError::EmptyRuleId);
        }
        if descriptor.display_name.trim().is_empty() {
            return Err(CustomInvariantRegistrationError::EmptyDisplayName);
        }
        if descriptor.operational.execution_point
            == crate::validation::data::InvariantExecutionPoint::ProposedState
        {
            return Err(CustomInvariantRegistrationError::AggregateExecutionPoint);
        }
        Ok(())
    }
}
