//! Declared compensation correction mechanism.

use super::super::postcondition::DeclaredAftermathPostcondition;

/// Correction by a forward operation that neutralizes committed effects.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredCompensation {
    compensating_operation_slot: String,
    postcondition: DeclaredAftermathPostcondition,
}

impl DeclaredCompensation {
    pub fn new(
        compensating_operation_slot: impl Into<String>,
        postcondition: DeclaredAftermathPostcondition,
    ) -> Self {
        Self {
            compensating_operation_slot: compensating_operation_slot.into(),
            postcondition,
        }
    }

    /// Checks descriptive meaning without granting schema admission.
    pub fn validate(&self) -> Result<(), super::DeclaredCorrectionMechanismDenial> {
        use super::DeclaredCorrectionMechanismDenial as Cause;
        if self.compensating_operation_slot.trim().is_empty() {
            return Err(Cause::EmptyCompensatingOperation);
        }
        match &self.postcondition {
            DeclaredAftermathPostcondition::ExactPriorTruth => {
                Err(Cause::CompensationRejectsExactPriorTruth)
            }
            DeclaredAftermathPostcondition::InvariantRestored { invariant }
                if invariant.trim().is_empty() =>
            {
                Err(Cause::EmptyCompensationPostcondition)
            }
            DeclaredAftermathPostcondition::BusinessPostcondition { identity }
                if identity.trim().is_empty() =>
            {
                Err(Cause::EmptyCompensationPostcondition)
            }
            _ => Ok(()),
        }
    }

    pub fn compensating_operation_slot(&self) -> &str {
        &self.compensating_operation_slot
    }

    pub const fn postcondition(&self) -> &DeclaredAftermathPostcondition {
        &self.postcondition
    }
}
