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
        let mut summary = CommitValidationSummary {
            execution_count: invariant_executions.len(),
            ..CommitValidationSummary::default()
        };

        for execution in invariant_executions {
            if execution.metadata().has_merged_plan() {
                summary.plan_backed_execution_count += 1;
            }

            match execution.metadata().observation_kind() {
                InvariantObservationKind::Committed => {
                    summary.committed_observation_count += 1;
                }
                InvariantObservationKind::Speculative => {
                    summary.speculative_observation_count += 1;
                }
            }

            match execution.metadata().execution_point() {
                InvariantExecutionPoint::ProposedState => {
                    summary.commit_boundary_seen = true;
                    summary.mutation_sensitive_seen = true;
                    summary.snapshot_publication_seen = true;
                }
                InvariantExecutionPoint::CommitBoundary => {
                    summary.commit_boundary_seen = true;
                }
                InvariantExecutionPoint::MutationSensitive => {
                    summary.mutation_sensitive_seen = true;
                }
                InvariantExecutionPoint::SnapshotPublication => {
                    summary.snapshot_publication_seen = true;
                }
                InvariantExecutionPoint::CertificationBoundary => {
                    summary.certification_boundary_seen = true;
                }
                InvariantExecutionPoint::HarnessAudit => {
                    summary.harness_audit_seen = true;
                }
                InvariantExecutionPoint::GraphComposition => {
                    summary.graph_composition_seen = true;
                }
            }

            summary.consumed_groups = summary
                .consumed_groups
                .union(execution.metadata().consumed_groups());
            summary.applicable_groups = summary
                .applicable_groups
                .union(execution.metadata().applicable_groups());

            match execution.metadata().disposition() {
                InvariantExecutionDisposition::Executed => {
                    summary.executed_count += 1;
                }
                InvariantExecutionDisposition::SkippedByPlanContract
                | InvariantExecutionDisposition::SkippedByMayBreakMask => {
                    summary.skipped_count += 1;
                }
            }

            let execution_summary = execution.summary();
            summary.result_count += execution_summary.result_count();
            summary.advisory_count += execution_summary.advisory_count();
            summary.violation_count += execution_summary.violation_count();
            summary.custom_failure_count += execution_summary.custom_failure_count();
            summary.custom_panic_count += execution_summary.custom_panic_count();
            summary.blocking_violation |= execution_summary.has_blocking_violation();
            summary.publication_violation |= execution_summary.has_publication_violation();
        }

        summary
    }

    pub fn summary(&self) -> CommitValidationSummary {
        self.summary
    }
}
