mod canonical_digest;
mod executable_plan;
mod materialized_aspect_values;
#[cfg(test)]
mod test_support;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub(crate) use canonical_digest::{
    compiled_executable_plan_digest, equality_witness_digest, merge_execution_diagnostics_digest,
    schema_snapshot_digest,
};
pub(crate) use executable_plan::{bound_parent_order, visible_record_snapshot};
pub use executable_plan::{
    AdoptSourceRecordPlan, BoundExecutableMergePlan, BoundExecutableMergeRecordPlan,
    ConvergeDeletedOnBothSidesRecordPlan, DeletedOnBothSidesSemantics, ExecutableAspectPlan,
    MergeExecutableRecordProvenance, MergeLineageContinuityVerdict, PreserveSharedRecordPlan,
    ReconcileRecordPlan, ReconciledIdentityBasis, SharedTruthWitness, VisibleMergeRecordSnapshot,
};
pub(crate) use materialized_aspect_values::{aspect_reference, materialized_value_aspect_key};
pub use materialized_aspect_values::{
    MaterializedAspectValue, MaterializedAspectValueEvidence, MergeValueMaterialization,
    MergeValueSourceSide,
};

use crate::branch::{RelationalBranchReferenceObservation, RelationalBranchVersion};
use crate::history::data::{CommitId, RelationalMergeBranchBasis};
use crate::merge::data::{
    BranchTouchedRecordDelta, LoweredMergePlanRecord, LoweredRecordDecision,
    LoweredRecordDecisionKind, MergePlanningArtifactCore, MergePlanningError,
    MergeSchemaSnapshotDigestBasis, NormalizedRelationalMergeRequest, VisibleMergeRecord,
};
use crate::transactions::data::RecordRef;

use super::plans::LoweredMergePlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeExecutionFreshnessPolicy {
    ExactAuthorityParity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RuntimeInstanceId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeExecutionAuthorityBinding {
    pub request: NormalizedRelationalMergeRequest,
    pub runtime_instance_id: RuntimeInstanceId,
    pub(crate) target_head_commit_id: CommitId,
    pub(crate) source_head_commit_id: CommitId,
    pub(crate) target_reference: RelationalBranchReferenceObservation,
    pub(crate) source_reference: RelationalBranchReferenceObservation,
    pub(crate) target_truth_version: RelationalBranchVersion,
    pub(crate) source_truth_version: RelationalBranchVersion,
    pub merge_base_commit_id: CommitId,
    pub schema_snapshot_digest: String,
    pub freshness_policy: MergeExecutionFreshnessPolicy,
    pub executable_plan_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeExecutionDeniedRecord {
    pub record: RecordRef,
    pub decision: LoweredRecordDecisionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeExecutionReadinessReport {
    pub record_count: usize,
    pub blocked_count: usize,
    pub rejected_count: usize,
    pub denied_records: Arc<[MergeExecutionDeniedRecord]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeExecutionPreparationError {
    OwnerBinding(crate::merge::data::RelationalMergeRequestBindingDenial),
    Planning(MergePlanningError),
    NotExecutionReady(MergeExecutionReadinessReport),
    Compilation(MergeExecutionCompilationError),
    MutationPlan(MergeExecutionMutationPlanError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeExecutionError {
    RuntimeInstanceMismatch {
        planned: RuntimeInstanceId,
        current: RuntimeInstanceId,
    },
    StaleBranchHead {
        branch: crate::history::data::BranchId,
        planned: CommitId,
        current: Option<CommitId>,
    },
    StaleBranchReference {
        branch: crate::history::data::BranchId,
        planned_generation: u64,
        current_generation: Option<u64>,
        planned_truth_version: u64,
        current_truth_version: Option<u64>,
    },
    MergeBaseDrift {
        planned: CommitId,
        current: Option<CommitId>,
    },
    SchemaSemanticDrift {
        planned_digest: String,
        current_digest: String,
    },
    Compilation(MergeExecutionCompilationError),
    MutationPlan(MergeExecutionMutationPlanError),
    Commit(crate::transactions::data::TransactionCommitError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeExecutionCompilationError {
    MissingSourceRecord {
        record: RecordRef,
    },
    MissingTargetRecord {
        record: RecordRef,
    },
    MissingSourceSnapshot {
        record: RecordRef,
        record_kind: &'static str,
    },
    MissingExecutableClass {
        record: RecordRef,
        resolution_class: crate::merge::data::MergeResolutionClass,
    },
    ExecutableClassDecisionMismatch {
        record: RecordRef,
        executable_class: crate::merge::data::MergeExecutableClass,
        decision: LoweredRecordDecisionKind,
    },
    UnsupportedRecordDecision {
        record: RecordRef,
        decision: LoweredRecordDecisionKind,
    },
    MissingAspectExecutionIntent {
        record: RecordRef,
        aspect_key: worth_foundational::facade::AspectKey,
    },
    MissingAuthorizedAspectValues {
        record: RecordRef,
        aspect_key: worth_foundational::facade::AspectKey,
    },
    MissingAspectValueWitness {
        record: RecordRef,
        aspect_key: worth_foundational::facade::AspectKey,
    },
    MissingAspectBinding {
        record: RecordRef,
        aspect_key: worth_foundational::facade::AspectKey,
    },
    PreparedAuthorityBindingMismatch {
        detail: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeExecutionMutationPlanError {
    MissingSourceHeadEnvelope,
    MissingTargetEntitySnapshot {
        record: RecordRef,
    },
    MissingTargetRelationSnapshot {
        record: RecordRef,
    },
    UnsupportedReconcileRecordKind {
        record: RecordRef,
        detail: &'static str,
    },
    UnsupportedAspectMutationMaterialization {
        record: RecordRef,
        aspect_key: worth_foundational::facade::AspectKey,
        detail: &'static str,
    },
    MissingResolvedAspectValue {
        record: RecordRef,
        aspect_key: worth_foundational::facade::AspectKey,
    },
    InvalidPinnedVisibleAspect {
        record: RecordRef,
        aspect_key: worth_foundational::facade::AspectKey,
        detail: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMergeExecution {
    compiled: CompiledMergeExecution,
    mutation_plan: PreparedMergeMutationPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedMergeMutationPlan {
    pub(crate) target_branch: crate::history::data::BranchId,
    pub(crate) source_branch: crate::history::data::BranchId,
    pub(crate) merge_parent_branches: Arc<[crate::history::data::BranchId]>,
    pub(crate) requested_merge_parent_count: usize,
    pub(crate) parent_commits: crate::history::data::OrderedParentList,
    pub(crate) merge_base_commits: Arc<[CommitId]>,
    pub(crate) merged_intents: Vec<crate::transactions::data::MutationIntent>,
    pub(crate) structural_summary: crate::transactions::data::MergeExecutionStructuralSummary,
    pub(crate) merge_execution_summary: crate::transactions::data::MergeExecutionSummary,
}

impl PreparedMergeMutationPlan {
    pub(crate) fn bind_transaction(
        &self,
        transaction_id: crate::transactions::data::TransactionId,
    ) -> crate::transactions::data::MergeCommitMutationPlan {
        crate::transactions::data::MergeCommitMutationPlan {
            transaction_id,
            target_branch: self.target_branch.clone(),
            source_branch: self.source_branch.clone(),
            merge_parent_branches: Arc::clone(&self.merge_parent_branches),
            requested_merge_parent_count: self.requested_merge_parent_count,
            parent_commits: self.parent_commits.clone(),
            merge_base_commits: Arc::clone(&self.merge_base_commits),
            merged_plan: crate::transactions::data::MergedCommitPlan {
                transaction_id,
                merged_intents: self.merged_intents.clone(),
            },
            structural_summary: self.structural_summary.clone(),
            merge_execution_summary: self.merge_execution_summary.clone(),
            proof_token: crate::transactions::data::merge_commit_mutation_plan_token(),
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

impl From<MergeExecutionMutationPlanError> for MergeExecutionError {
    fn from(value: MergeExecutionMutationPlanError) -> Self {
        Self::MutationPlan(value)
    }
}

impl From<crate::transactions::data::TransactionCommitError> for MergeExecutionError {
    fn from(value: crate::transactions::data::TransactionCommitError) -> Self {
        Self::Commit(value)
    }
}

impl PreparedMergeExecution {
    pub(crate) fn from_compiled(
        compiled: CompiledMergeExecution,
        mutation_plan: PreparedMergeMutationPlan,
    ) -> Self {
        Self {
            compiled,
            mutation_plan,
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
    pub fn merged_intents(&self) -> &[crate::transactions::data::MutationIntent] {
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
