use crate::publication::bundle::PublicationStage;
use crate::publication::data::PublicationError;
use crate::transactions::data::{CommitConflict, ConflictClass};
use crate::validation::data::{
    InvariantFailureEffect, InvariantViolation, InvariantViolationFields,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantFailure {
    execution_point: crate::validation::data::InvariantExecutionPoint,
    effect: InvariantFailureEffect,
    violation: InvariantViolation,
}

impl InvariantFailure {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.violation.detail.capacity() as u64)
            .saturating_add(self.violation.fields.owned_allocation_capacity_bytes())
    }

    pub(crate) fn new(
        execution_point: crate::validation::data::InvariantExecutionPoint,
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

    pub fn execution_point(&self) -> crate::validation::data::InvariantExecutionPoint {
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
