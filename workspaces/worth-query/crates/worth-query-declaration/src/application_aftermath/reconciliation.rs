//! Declared reconciliation procedure for external-owner correction authority.

/// Named reconciliation procedure required when correction authority is
/// `RuntimeWithExternalOwner`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DeclaredReconciliationProcedure {
    procedure_slot: String,
}

impl DeclaredReconciliationProcedure {
    pub fn new(procedure_slot: impl Into<String>) -> Self {
        Self {
            procedure_slot: procedure_slot.into(),
        }
    }

    /// Checks descriptive meaning without granting schema admission.
    pub fn validate(&self) -> Result<(), super::DeclaredCorrectionMechanismDenial> {
        if self.procedure_slot.trim().is_empty() {
            Err(super::DeclaredCorrectionMechanismDenial::EmptyReconciliationProcedure)
        } else {
            Ok(())
        }
    }

    pub fn procedure_slot(&self) -> &str {
        &self.procedure_slot
    }
}
