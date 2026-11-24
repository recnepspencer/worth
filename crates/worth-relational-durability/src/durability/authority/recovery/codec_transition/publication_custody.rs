//! Retain native codec phase custody through detached preparation and settlement.
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

/// The memory obligation makes this an owning runtime lifecycle value. Sharing
/// this allocation carries one exact transition occurrence, never caller text.
#[derive(Debug)]
pub struct RecoveredCodecPublicationCustody {
    pub(super) source:
        worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    pub(super) retirements: Vec<super::retirement_preparation::PreparedCodecRetirement>,
    pub(super) retirement_funding: Arc<worth_execution::ExecutionMemoryReservation>,
    pub(super) activation: Arc<
        contracts::schema::data::program_activation::RecoveredCheckpointProgramActivationUpdate,
    >,
    _memory: worth_execution::ExecutionMemoryReservation,
}
impl RecoveredCodecPublicationCustody {
    pub fn admits_source(
        &self,
        source: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> bool {
        self.source.admission_identity() == source.admission_identity()
    }
    pub fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        target: &contracts::schema::data::RelationalSchemaRegistry,
        retirement: &worth_relational_branch::internal_port::history::data::codec_retirement_closure::OwnedCodecRetirementClosure,
        activation: &contracts::schema::data::program_activation::RecoveredCheckpointProgramActivationUpdate,
        memory: worth_execution::ExecutionMemoryReservation,
    ) -> Result<
        Arc<Self>,
        crate::durability::transition_admission_denial::RelationalSchemaTransitionAdmissionDenial,
    > {
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
pub struct AdmittedRecoveredCodecPublication {
    custody: Arc<RecoveredCodecPublicationCustody>,
}
impl AdmittedRecoveredCodecPublication {
    pub fn new(custody: Arc<RecoveredCodecPublicationCustody>) -> Self {
        Self { custody }
    }
    pub fn admits(&self, custody: &Arc<RecoveredCodecPublicationCustody>) -> bool {
        Arc::ptr_eq(&self.custody, custody)
    }
}
