//! Retains the exact lowered merge and the plan that can consume it.
use super::plans::{BranchTouchedRecordDelta, LoweredMergePlan};
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::history::data::merge_branch_basis::RelationalMergeBranchBasis;
use crate::contracts::merge::data::artifacts::planning_artifact_core::MergePlanningArtifactCore;
use crate::contracts::merge::data::artifacts::schema_snapshot_digest_basis::MergeSchemaSnapshotDigestBasis;
use crate::contracts::merge::data::execution::executable_plan::BoundExecutableMergePlan;
use crate::contracts::merge::data::execution::{
    MergeExecutionDeniedRecord, MergeExecutionFreshnessPolicy, MergeExecutionReadinessReport,
};
use crate::contracts::merge::data::policy::{
    LoweredMergePlanRecord, LoweredRecordDecision, LoweredRecordDecisionKind,
};
use crate::contracts::merge::data::requests::normalized::NormalizedRelationalMergeRequest;
use crate::contracts::merge::data::visible_record::VisibleMergeRecord;
use crate::contracts::transactions::data::outcomes::plan_artifacts::{
    assemble_merge_commit_mutation_plan, MergeCommitMutationPlanParts,
};
use std::sync::Arc;

#[doc(hidden)]
pub trait PreparedMergeExecutionOwnerPort: Sized {
    fn from_compiled(
        compiled: CompiledMergeExecution,
        mutation_plan: PreparedMergeMutationPlan,
        selected: (
            worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
            worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        ),
    ) -> Self;

    fn mutation_plan(&self) -> &PreparedMergeMutationPlan;

    fn compiled(&self) -> &CompiledMergeExecution;

    fn execution_ready_plan(&self) -> &ExecutionReadyLoweredMergePlan;

    fn bound_executable_plan(&self) -> &BoundExecutableMergePlan;
}

impl PreparedMergeExecutionOwnerPort for PreparedMergeExecution {
    fn from_compiled(
        compiled: CompiledMergeExecution,
        mutation_plan: PreparedMergeMutationPlan,
        selected: (
            worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
            worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        ),
    ) -> Self {
        Self::from_compiled(compiled, mutation_plan, selected)
    }

    fn mutation_plan(&self) -> &PreparedMergeMutationPlan {
        self.mutation_plan()
    }

    fn compiled(&self) -> &CompiledMergeExecution {
        self.compiled()
    }

    fn execution_ready_plan(&self) -> &ExecutionReadyLoweredMergePlan {
        self.execution_ready_plan()
    }

    fn bound_executable_plan(&self) -> &BoundExecutableMergePlan {
        self.bound_executable_plan()
    }
}

#[doc(hidden)]
#[path = "execution/test_support.rs"]
pub mod test_support;

#[derive(Debug, Clone)]
pub struct PreparedMergeExecution {
    compiled: CompiledMergeExecution,
    mutation_plan: PreparedMergeMutationPlan,
    _selected: (
        worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMergeMutationPlan {
    pub(crate) target_branch: crate::contracts::history::data::coordinates::BranchId,
    pub(crate) source_branch: crate::contracts::history::data::coordinates::BranchId,
    pub(crate) merge_parent_branches: Arc<[crate::contracts::history::data::coordinates::BranchId]>,
    pub(crate) requested_merge_parent_count: usize,
    pub(crate) parent_commits: crate::contracts::history::data::commit_receipt::OrderedParentList,
    pub(crate) merge_base_commits: Arc<[CommitId]>,
    pub(crate) merged_intents:
        Vec<crate::contracts::transactions::data::intents::mutation_intent::MutationIntent>,
    pub(crate) structural_summary:
        crate::contracts::transactions::data::outcomes::plan_artifacts::MergeExecutionStructuralSummary,
    pub(crate) merge_execution_summary:
        crate::contracts::transactions::data::outcomes::plan_artifacts::MergeExecutionSummary,
}

impl PreparedMergeMutationPlan {
    pub fn bind_transaction(
        &self,
        transaction_id: crate::contracts::transactions::data::primitives::TransactionId,
    ) -> crate::contracts::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan
    {
        assemble_merge_commit_mutation_plan(MergeCommitMutationPlanParts {
            transaction_id,
            target_branch: self.target_branch.clone(),
            source_branch: self.source_branch.clone(),
            merge_parent_branches: Arc::clone(&self.merge_parent_branches),
            requested_merge_parent_count: self.requested_merge_parent_count,
            parent_commits: self.parent_commits.clone(),
            merge_base_commits: Arc::clone(&self.merge_base_commits),
            merged_plan: crate::contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan {
                transaction_id,
                merged_intents: self.merged_intents.clone(),
            },
            structural_summary: self.structural_summary.clone(),
            merge_execution_summary: self.merge_execution_summary.clone(),
            proof_token: crate::contracts::transactions::data::outcomes::plan_artifacts::merge_commit_mutation_plan_token(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledMergeExecution {
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
            worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
            worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
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
    ) -> &[crate::contracts::transactions::data::intents::mutation_intent::MutationIntent] {
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
    pub fn new(
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
pub struct ExecutionReadyLoweredMergePlan {
    pub request: NormalizedRelationalMergeRequest,
    pub basis: RelationalMergeBranchBasis,
    pub schema_snapshot: MergeSchemaSnapshotDigestBasis,
    pub source_records: Arc<[VisibleMergeRecord]>,
    pub target_touched_records: Arc<[BranchTouchedRecordDelta]>,
    pub lowered_records: Arc<[LoweredMergePlanRecord]>,
    pub(crate) freshness_policy: MergeExecutionFreshnessPolicy,
}

impl ExecutionReadyLoweredMergePlan {
    pub fn try_from_lowered(
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
