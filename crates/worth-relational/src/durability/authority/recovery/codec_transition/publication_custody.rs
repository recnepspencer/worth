//! Retain native codec phase custody through detached preparation and settlement.
use std::sync::Arc;

/// The memory obligation makes this an owning runtime lifecycle value. Sharing
/// this allocation carries one exact transition occurrence, never caller text.
#[derive(Debug)]
pub(crate) struct RecoveredCodecPublicationCustody {
    pub(super) source: crate::branch::AdmittedRelationalBranchBasis,
    pub(super) retirements: Vec<super::retirement_preparation::PreparedCodecRetirement>,
    pub(super) retirement_funding: Arc<worth_execution::ExecutionMemoryReservation>,
    pub(super) activation: Arc<crate::runtime::RecoveredCheckpointProgramActivationUpdate>,
    _memory: worth_execution::ExecutionMemoryReservation,
}
impl RecoveredCodecPublicationCustody {
    pub(super) fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::branch::AdmittedRelationalBranchBasis,
        target: &crate::schema::data::RelationalSchemaRegistry,
        retirement: &crate::history::data::OwnedCodecRetirementClosure,
        activation: &crate::runtime::RecoveredCheckpointProgramActivationUpdate,
        memory: worth_execution::ExecutionMemoryReservation,
    ) -> Result<Arc<Self>, crate::runtime::RelationalSchemaTransitionAdmissionDenial> {
        let (retirements, retirement_funding) = super::retirement_preparation::capture(
            execution, source, target, retirement, activation,
        )?;
        Ok(Arc::new(Self {
            source: source.clone(),
            retirements,
            retirement_funding,
            activation: Arc::new(activation.clone()),
            _memory: memory,
        }))
    }
}
impl PartialEq for RecoveredCodecPublicationCustody {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for RecoveredCodecPublicationCustody {}

/// Issued only after the native owner validates protected payload preservation
/// and prepares the complete target index image for this exact occurrence.
pub(crate) struct AdmittedRecoveredCodecPublication {
    custody: Arc<RecoveredCodecPublicationCustody>,
}
impl AdmittedRecoveredCodecPublication {
    pub(super) fn new(custody: Arc<RecoveredCodecPublicationCustody>) -> Self {
        Self { custody }
    }
    pub(crate) fn admits(&self, custody: &Arc<RecoveredCodecPublicationCustody>) -> bool {
        Arc::ptr_eq(&self.custody, custody)
    }
}
