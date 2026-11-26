pub mod canonical_digest;
pub mod executable_plan;
pub mod materialized_aspect_values;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use canonical_digest::{
    compiled_executable_plan_digest, equality_witness_digest, schema_snapshot_digest,
};
pub use executable_plan::{bound_parent_order, visible_record_snapshot};
pub use executable_plan::{
    AdoptSourceRecordPlan, BoundExecutableMergePlan, BoundExecutableMergeRecordPlan,
    ConvergeDeletedOnBothSidesRecordPlan, DeletedOnBothSidesSemantics, ExecutableAspectPlan,
    MergeExecutableRecordProvenance, MergeLineageContinuityVerdict, PreserveSharedRecordPlan,
    ReconcileRecordPlan, ReconciledIdentityBasis, SharedTruthWitness, VisibleMergeRecordSnapshot,
};
pub use materialized_aspect_values::{aspect_reference, materialized_value_aspect_key};
pub use materialized_aspect_values::{
    MaterializedAspectValue, MaterializedAspectValueEvidence, MergeValueMaterialization,
    MergeValueSourceSide,
};

use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::branch::version::RelationalBranchVersion;
use crate::history::data::coordinates::{BranchId, CommitId};
use crate::merge::data::policy::LoweredRecordDecisionKind;
use crate::merge::data::requests::normalized::NormalizedRelationalMergeRequest;
use crate::transactions::data::primitives::RecordRef;

use super::planning_denial::MergePlanningError;

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
    pub target_head_commit_id: CommitId,
    pub source_head_commit_id: CommitId,
    pub target_reference: RelationalBranchReferenceObservation,
    pub source_reference: RelationalBranchReferenceObservation,
    pub target_truth_version: RelationalBranchVersion,
    pub source_truth_version: RelationalBranchVersion,
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
    Planning(MergePlanningError),
    NotExecutionReady(MergeExecutionReadinessReport),
    Compilation(MergeExecutionCompilationError),
    MutationPlan(MergeExecutionMutationPlanError),
    /// The source branch names this record so: a merge would adopt it unnamed.
    NamedRecord(
        BranchId,
        RecordRef,
        crate::lineage::data::naming::PersistentNameIdentity,
    ),
    /// The source branch published group lineage of this scheme at this
    /// commit, after the merge base: a merge never carries it.
    NamedGroupLineage(BranchId, CommitId, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeExecutionError {
    Planning(crate::merge::data::planning_denial::MergePlanningError),
    RuntimeInstanceMismatch {
        planned: RuntimeInstanceId,
        current: RuntimeInstanceId,
    },
    StaleBranchHead {
        branch: crate::history::data::coordinates::BranchId,
        planned: CommitId,
        current: Option<CommitId>,
    },
    StaleBranchReference {
        branch: crate::history::data::coordinates::BranchId,
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
    Commit(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError),
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
        resolution_class: crate::merge::data::policy::MergeResolutionClass,
    },
    ExecutableClassDecisionMismatch {
        record: RecordRef,
        executable_class: crate::merge::data::policy::MergeExecutableClass,
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
    /// Target read failed before executable mutation intents were produced.
    Read(crate::execution::denial_cause::RelationalExecutionDenialCause),
    MissingTargetEntitySnapshot {
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

impl From<MergeExecutionMutationPlanError> for MergeExecutionError {
    fn from(value: MergeExecutionMutationPlanError) -> Self {
        Self::MutationPlan(value)
    }
}

impl From<crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>
    for MergeExecutionError
{
    fn from(
        value: crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    ) -> Self {
        Self::Commit(value)
    }
}
