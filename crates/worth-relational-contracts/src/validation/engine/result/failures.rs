use crate::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use crate::internal_port::validation::data::results::InvariantViolationFieldsOwnerPort as _;
use crate::publication::bundle::publication_status::PublicationStage;
use crate::publication::data::publication_error::PublicationError;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::validation::data::execution::InvariantFailureEffect;
use crate::validation::data::results::{InvariantViolation, InvariantViolationFields};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantFailure {
    execution_point: crate::validation::data::execution::InvariantExecutionPoint,
    effect: InvariantFailureEffect,
    violation: InvariantViolation,
}

impl InvariantFailure {
    pub fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.violation.detail.capacity() as u64)
            .saturating_add(self.violation.fields.owned_allocation_capacity_bytes())
    }

    pub fn new(
        execution_point: crate::validation::data::execution::InvariantExecutionPoint,
        effect: InvariantFailureEffect,
        violation: InvariantViolation,
    ) -> Self {
        Self {
            execution_point,
            effect,
            violation,
        }
    }

    pub fn code(&self) -> crate::diagnostics::data::DiagnosticCode {
        self.violation.code
    }

    pub fn execution_point(&self) -> crate::validation::data::execution::InvariantExecutionPoint {
        self.execution_point
    }

    pub fn effect(&self) -> InvariantFailureEffect {
        self.effect
    }

    pub fn violation(&self) -> &InvariantViolation {
        &self.violation
    }

    pub fn detail(&self) -> &str {
        &self.violation.detail
    }

    pub fn fields(&self) -> &InvariantViolationFields {
        &self.violation.fields
    }

    pub fn into_commit_conflict(self) -> CommitConflict {
        CommitConflict::new(ConflictClass::InvariantViolation {
            code: self.violation.code,
            detail: self.violation.detail,
            fields: self.violation.fields,
        })
    }

    pub fn into_publication_error(self, stage: PublicationStage) -> PublicationError {
        PublicationError::from_invariant_violation(stage, self.violation)
    }
}
