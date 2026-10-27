mod diagnostic_artifacts;
mod execution_summary;
mod failures;
mod metadata;
#[cfg(test)]
mod tests;

use crate::validation::data::{InvariantCheckResult, InvariantDecisionRecord};

pub use diagnostic_artifacts::{
    CustomInvariantTraceArtifact, InvariantFailureArtifact, InvariantProofBoundaryArtifact,
};
pub use execution_summary::InvariantExecutionSummary;
pub use failures::InvariantFailure;
pub use metadata::{
    InvariantExecutionDisposition, InvariantExecutionMetadata, InvariantPlanScopeClass,
    InvariantProofBoundarySummary, InvariantScopeWideningCause,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantExecutionResult {
    metadata: InvariantExecutionMetadata,
    summary: InvariantExecutionSummary,
    results: Vec<InvariantCheckResult>,
    decision_log: Vec<InvariantDecisionRecord>,
}

fn build_decision_log(results: &[InvariantCheckResult]) -> Vec<InvariantDecisionRecord> {
    results
        .iter()
        .map(InvariantCheckResult::decision_record)
        .collect()
}

impl InvariantExecutionResult {
    pub fn executed(
        metadata: InvariantExecutionMetadata,
        results: Vec<InvariantCheckResult>,
    ) -> Self {
        assert_eq!(
            metadata.disposition(),
            InvariantExecutionDisposition::Executed,
            "executed invariant results require an executed disposition",
        );
        let summary = InvariantExecutionSummary::from_results(&results);
        let decision_log = build_decision_log(&results);
        Self::from_reduced_parts(metadata, summary, results, decision_log)
    }

    /// The reducer already classified each check and admitted its decision log.
    pub(crate) fn from_reduced_parts(
        metadata: InvariantExecutionMetadata,
        summary: InvariantExecutionSummary,
        results: Vec<InvariantCheckResult>,
        decision_log: Vec<InvariantDecisionRecord>,
    ) -> Self {
        debug_assert_eq!(summary.result_count(), results.len());
        debug_assert_eq!(decision_log.len(), results.len());
        Self {
            metadata,
            summary,
            results,
            decision_log,
        }
    }

    pub fn skipped(metadata: InvariantExecutionMetadata) -> Self {
        assert_ne!(
            metadata.disposition(),
            InvariantExecutionDisposition::Executed,
            "skipped invariant results require a skipped disposition",
        );
        Self {
            metadata,
            summary: InvariantExecutionSummary::from_results(&[]),
            results: Vec::new(),
            decision_log: Vec::new(),
        }
    }

    pub fn metadata(&self) -> &InvariantExecutionMetadata {
        &self.metadata
    }

    pub fn summary(&self) -> &InvariantExecutionSummary {
        &self.summary
    }

    pub fn results(&self) -> &[InvariantCheckResult] {
        &self.results
    }

    pub fn decision_log(&self) -> &[InvariantDecisionRecord] {
        &self.decision_log
    }

    pub fn blocking_failures(&self) -> Vec<InvariantFailure> {
        self.summary.blocking_failures(&self.results)
    }

    pub fn publication_failures(&self) -> Vec<InvariantFailure> {
        self.summary.publication_failures(&self.results)
    }

    pub fn proof_boundary_artifact(&self) -> Option<InvariantProofBoundaryArtifact> {
        diagnostic_artifacts::proof_boundary_artifact(self)
    }

    pub fn failure_artifact(&self, failure: &InvariantFailure) -> InvariantFailureArtifact {
        diagnostic_artifacts::failure_artifact(self, failure)
    }

    pub fn custom_trace_artifact(
        result: &InvariantCheckResult,
    ) -> Option<CustomInvariantTraceArtifact> {
        diagnostic_artifacts::custom_trace_artifact(result)
    }

    pub fn into_results(self) -> Vec<InvariantCheckResult> {
        self.results
    }
}
