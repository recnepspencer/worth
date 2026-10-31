use crate::validation::data::{InvariantExecutionPoint, InvariantGroupSet};
use crate::validation::engine::{
    InvariantExecutionDisposition, InvariantExecutionResult, InvariantObservationKind,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitValidation {
    pub summary: CommitValidationSummary,
    pub invariant_executions: Vec<InvariantExecutionResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CommitValidationSummary {
    pub execution_count: usize,
    pub executed_count: usize,
    pub skipped_count: usize,
    pub committed_observation_count: usize,
    pub speculative_observation_count: usize,
    pub plan_backed_execution_count: usize,
    pub commit_boundary_seen: bool,
    pub mutation_sensitive_seen: bool,
    pub snapshot_publication_seen: bool,
    pub certification_boundary_seen: bool,
    pub harness_audit_seen: bool,
    pub graph_composition_seen: bool,
    pub consumed_groups: InvariantGroupSet,
    pub applicable_groups: InvariantGroupSet,
    pub result_count: usize,
    pub advisory_count: usize,
    pub violation_count: usize,
    pub custom_failure_count: usize,
    pub custom_panic_count: usize,
    pub blocking_violation: bool,
    pub publication_violation: bool,
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

pub(crate) struct ValidationSummaryInput<'a> {
    pub(crate) execution_point: InvariantExecutionPoint,
    pub(crate) observation_kind: InvariantObservationKind,
    pub(crate) consumed_groups: InvariantGroupSet,
    pub(crate) applicable_groups: InvariantGroupSet,
    pub(crate) disposition: InvariantExecutionDisposition,
    pub(crate) has_merged_plan: bool,
    pub(crate) results: &'a crate::validation::engine::InvariantExecutionSummary,
}

impl CommitValidationSummary {
    pub(crate) fn include_execution(&mut self, input: ValidationSummaryInput<'_>) {
        self.execution_count += 1;
        if input.has_merged_plan {
            self.plan_backed_execution_count += 1;
        }

        match input.observation_kind {
            InvariantObservationKind::Committed => {
                self.committed_observation_count += 1;
            }
            InvariantObservationKind::Speculative => {
                self.speculative_observation_count += 1;
            }
        }

        match input.execution_point {
            InvariantExecutionPoint::CommitBoundary => {
                self.commit_boundary_seen = true;
            }
            InvariantExecutionPoint::MutationSensitive => {
                self.mutation_sensitive_seen = true;
            }
            InvariantExecutionPoint::SnapshotPublication => {
                self.snapshot_publication_seen = true;
            }
            InvariantExecutionPoint::CertificationBoundary => {
                self.certification_boundary_seen = true;
            }
            InvariantExecutionPoint::HarnessAudit => {
                self.harness_audit_seen = true;
            }
            InvariantExecutionPoint::GraphComposition => {
                self.graph_composition_seen = true;
            }
        }

        self.consumed_groups = self.consumed_groups.union(input.consumed_groups);
        self.applicable_groups = self.applicable_groups.union(input.applicable_groups);

        match input.disposition {
            InvariantExecutionDisposition::Executed => {
                self.executed_count += 1;
            }
            InvariantExecutionDisposition::SkippedByPlanContract
            | InvariantExecutionDisposition::SkippedByMayBreakMask => {
                self.skipped_count += 1;
            }
        }

        let execution_summary = input.results;
        self.result_count += execution_summary.result_count();
        self.advisory_count += execution_summary.advisory_count();
        self.violation_count += execution_summary.violation_count();
        self.custom_failure_count += execution_summary.custom_failure_count();
        self.custom_panic_count += execution_summary.custom_panic_count();
        self.blocking_violation |= execution_summary.has_blocking_violation();
        self.publication_violation |= execution_summary.has_publication_violation();
    }
}
