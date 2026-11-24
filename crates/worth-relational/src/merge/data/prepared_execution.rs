//! Retains the exact lowered merge and the plan that can consume it.
use super::artifacts::{MergePlanningArtifactCore, MergeSchemaSnapshotDigestBasis};
use super::execution::{
    BoundExecutableMergePlan, MergeExecutionDeniedRecord, MergeExecutionFreshnessPolicy,
    MergeExecutionReadinessReport,
};
use super::plans::{BranchTouchedRecordDelta, LoweredMergePlan};
use super::policy::{LoweredMergePlanRecord, LoweredRecordDecision, LoweredRecordDecisionKind};
use super::requests::normalized::NormalizedRelationalMergeRequest;
use super::visible_record::VisibleMergeRecord;
use crate::history::data::coordinates::CommitId;
use crate::history::data::merge_branch_basis::RelationalMergeBranchBasis;
use std::sync::Arc;

#[cfg(test)]
#[path = "execution/test_support.rs"]
mod test_support;

#[derive(Debug, Clone)]
pub struct PreparedMergeExecution {
    compiled: CompiledMergeExecution,
    mutation_plan: PreparedMergeMutationPlan,
    _selected: (
        crate::branch::basis::AdmittedRelationalBranchBasis,
        crate::branch::basis::AdmittedRelationalBranchBasis,
    ),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedMergeMutationPlan {
    pub(crate) target_branch: crate::history::data::coordinates::BranchId,
    pub(crate) source_branch: crate::history::data::coordinates::BranchId,
    pub(crate) merge_parent_branches: Arc<[crate::history::data::coordinates::BranchId]>,
    pub(crate) requested_merge_parent_count: usize,
    pub(crate) parent_commits: crate::history::data::commit_receipt::OrderedParentList,
    pub(crate) merge_base_commits: Arc<[CommitId]>,
    pub(crate) merged_intents:
        Vec<crate::transactions::data::intents::mutation_intent::MutationIntent>,
    pub(crate) structural_summary:
        crate::transactions::data::outcomes::plan_artifacts::MergeExecutionStructuralSummary,
    pub(crate) merge_execution_summary:
        crate::transactions::data::outcomes::plan_artifacts::MergeExecutionSummary,
}

impl PreparedMergeMutationPlan {
    pub(crate) fn bind_transaction(
        &self,
        transaction_id: crate::transactions::data::primitives::TransactionId,
    ) -> crate::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan {
        crate::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan {
            transaction_id,
            target_branch: self.target_branch.clone(),
            source_branch: self.source_branch.clone(),
            merge_parent_branches: Arc::clone(&self.merge_parent_branches),
            requested_merge_parent_count: self.requested_merge_parent_count,
            parent_commits: self.parent_commits.clone(),
            merge_base_commits: Arc::clone(&self.merge_base_commits),
            merged_plan: crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan {
                transaction_id,
                merged_intents: self.merged_intents.clone(),
            },
            structural_summary: self.structural_summary.clone(),
            merge_execution_summary: self.merge_execution_summary.clone(),
            proof_token: crate::transactions::data::outcomes::plan_artifacts::merge_commit_mutation_plan_token(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledMergeExecution {
    artifact: MergePlanningArtifactCore,
    request: NormalizedRelationalMergeRequest,
    execution_ready_plan: ExecutionReadyLoweredMergePlan,
    bound_executable_plan: BoundExecutableMergePlan,
    execution_token: PreparedMergeExecutionToken,
}

impl PartialEq for PreparedMergeExecution {
    fn eq(&self, other: &Self) -> bool {
        self.compiled == other.compiled && self.mutation_plan == other.mutation_plan
    }
}
impl Eq for PreparedMergeExecution {}

impl PreparedMergeExecution {
    pub(crate) fn from_compiled(
        compiled: CompiledMergeExecution,
        mutation_plan: PreparedMergeMutationPlan,
        selected: (
            crate::branch::basis::AdmittedRelationalBranchBasis,
            crate::branch::basis::AdmittedRelationalBranchBasis,
        ),
    ) -> Self {
        Self {
            compiled,
            mutation_plan,
            _selected: selected,
        }
    }

    pub(crate) fn mutation_plan(&self) -> &PreparedMergeMutationPlan {
        &self.mutation_plan
    }

    pub(crate) fn compiled(&self) -> &CompiledMergeExecution {
        &self.compiled
    }

    pub fn request(&self) -> &NormalizedRelationalMergeRequest {
        self.compiled.request()
    }

    pub fn artifact(&self) -> &MergePlanningArtifactCore {
        self.compiled.artifact()
    }

    /// The writes this merge commits onto its target, in commit order. Read
    /// only: a caller may refuse the merge from them, never alter it.
    pub fn merged_intents(
        &self,
    ) -> &[crate::transactions::data::intents::mutation_intent::MutationIntent] {
        &self.mutation_plan.merged_intents
    }

    pub(crate) fn execution_ready_plan(&self) -> &ExecutionReadyLoweredMergePlan {
        self.compiled.execution_ready_plan()
    }

    pub(crate) fn bound_executable_plan(&self) -> &BoundExecutableMergePlan {
        self.compiled.bound_executable_plan()
    }
}

impl CompiledMergeExecution {
    pub(crate) fn new(
        request: NormalizedRelationalMergeRequest,
        artifact: MergePlanningArtifactCore,
        execution_ready_plan: ExecutionReadyLoweredMergePlan,
        bound_executable_plan: BoundExecutableMergePlan,
    ) -> Self {
        Self {
            request,
            artifact,
            execution_ready_plan,
            bound_executable_plan,
            execution_token: PreparedMergeExecutionToken,
        }
    }

    pub(crate) fn request(&self) -> &NormalizedRelationalMergeRequest {
        &self.request
    }

    pub(crate) fn artifact(&self) -> &MergePlanningArtifactCore {
        &self.artifact
    }

    pub(crate) fn execution_ready_plan(&self) -> &ExecutionReadyLoweredMergePlan {
        &self.execution_ready_plan
    }

    pub(crate) fn bound_executable_plan(&self) -> &BoundExecutableMergePlan {
        &self.bound_executable_plan
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreparedMergeExecutionToken;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionReadyLoweredMergePlan {
    pub(crate) request: NormalizedRelationalMergeRequest,
    pub(crate) basis: RelationalMergeBranchBasis,
    pub(crate) schema_snapshot: MergeSchemaSnapshotDigestBasis,
    pub(crate) source_records: Arc<[VisibleMergeRecord]>,
    pub(crate) target_touched_records: Arc<[BranchTouchedRecordDelta]>,
    pub(crate) lowered_records: Arc<[LoweredMergePlanRecord]>,
    pub(crate) freshness_policy: MergeExecutionFreshnessPolicy,
}

impl ExecutionReadyLoweredMergePlan {
    pub(crate) fn try_from_lowered(
        plan: LoweredMergePlan,
        schema_snapshot: MergeSchemaSnapshotDigestBasis,
    ) -> Result<Self, MergeExecutionReadinessReport> {
        let denied_records = plan
            .lowered_records
            .iter()
            .filter_map(readiness_denial_for_lowered_record)
            .collect::<Vec<_>>();

        if !denied_records.is_empty() {
            return Err(MergeExecutionReadinessReport {
                record_count: plan.lowered_summary.record_count,
                blocked_count: plan.lowered_summary.blocked_count,
                rejected_count: plan.lowered_summary.rejected_count,
                denied_records: Arc::from(denied_records),
            });
        }

        Ok(Self {
            request: plan.request,
            basis: plan.basis,
            schema_snapshot,
            source_records: plan.source_records,
            target_touched_records: plan.target_delta.touched_records,
            lowered_records: plan.lowered_records,
            freshness_policy: MergeExecutionFreshnessPolicy::ExactAuthorityParity,
        })
    }
}

fn readiness_denial_for_lowered_record(
    record: &LoweredMergePlanRecord,
) -> Option<MergeExecutionDeniedRecord> {
    match record.record_decision {
        LoweredRecordDecision::Execute(_) => None,
        LoweredRecordDecision::Block(_) => Some(MergeExecutionDeniedRecord {
            record: record.record.clone(),
            decision: LoweredRecordDecisionKind::Block,
        }),
        LoweredRecordDecision::Reject(_) => Some(MergeExecutionDeniedRecord {
            record: record.record.clone(),
            decision: LoweredRecordDecisionKind::Reject,
        }),
    }
}
