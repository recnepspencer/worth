//! Original invariant results and their descriptive commit summary.
use super::commit_validation::{CommitValidationSummary, ValidationSummaryInput};
use crate::validation::engine::result::InvariantExecutionResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitValidation {
    pub summary: CommitValidationSummary,
    pub invariant_executions: Vec<InvariantExecutionResult>,
}

impl CommitValidation {
    pub fn invariant_executions(&self) -> &[InvariantExecutionResult] {
        &self.invariant_executions
    }

    pub fn summarize(invariant_executions: &[InvariantExecutionResult]) -> CommitValidationSummary {
        let mut summary = CommitValidationSummary::default();
        for execution in invariant_executions {
            summary.include_execution(ValidationSummaryInput {
                execution_point: execution.metadata().execution_point(),
                observation_kind: execution.metadata().observation_kind(),
                consumed_groups: execution.metadata().consumed_groups(),
                applicable_groups: execution.metadata().applicable_groups(),
                disposition: execution.metadata().disposition(),
                has_merged_plan: execution.metadata().has_merged_plan(),
                results: execution.summary(),
            });
        }
        summary
    }

    pub fn summary(&self) -> CommitValidationSummary {
        self.summary
    }
}
