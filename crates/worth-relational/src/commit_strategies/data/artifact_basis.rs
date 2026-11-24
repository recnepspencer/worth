//! Descriptive inputs sealed by the original strategy lowering.
use super::execution::execution_draft::StrategyExecutionDraft;
use super::lowering_meaning::{StrategyLoweringProvenance, StrategyLoweringSummary};
use super::request::CanonicalStrategyCommitRequest;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;

#[derive(Debug)]
pub struct StrategyArtifactBasis {
    request: CanonicalStrategyCommitRequest,
    execution: StrategyExecutionDraft,
    merged_plan: MergedCommitPlan,
    provenance: StrategyLoweringProvenance,
    summary: StrategyLoweringSummary,
}

impl StrategyArtifactBasis {
    pub(super) fn new(
        request: CanonicalStrategyCommitRequest,
        execution: StrategyExecutionDraft,
        merged_plan: MergedCommitPlan,
        provenance: StrategyLoweringProvenance,
        summary: StrategyLoweringSummary,
    ) -> Self {
        Self {
            request,
            execution,
            merged_plan,
            provenance,
            summary,
        }
    }

    pub fn request(&self) -> &CanonicalStrategyCommitRequest {
        &self.request
    }

    pub fn execution(&self) -> &StrategyExecutionDraft {
        &self.execution
    }

    pub fn merged_plan(&self) -> &MergedCommitPlan {
        &self.merged_plan
    }

    pub fn lowering_provenance(&self) -> StrategyLoweringProvenance {
        self.provenance
    }

    pub fn lowering_summary(&self) -> &StrategyLoweringSummary {
        &self.summary
    }

    pub(super) fn into_merged_plan(self) -> MergedCommitPlan {
        self.merged_plan
    }
}
