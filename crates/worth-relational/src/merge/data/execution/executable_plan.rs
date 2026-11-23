use crate::history::data::merge_branch_basis::RelationalMergeBranchBasis;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::CommitId;
use crate::merge::data::conflicts::MergeConflictClass;
use crate::merge::data::execution::MergeExecutionAuthorityBinding;
use crate::merge::data::policy::ResolvedAspectMergePolicy;
use crate::merge::data::visible_record::{VisibleMergeRecord, VisibleMergeRecordKind};
use crate::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::primitives::RecordRef;

use super::materialized_aspect_values::MaterializedAspectValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum VisibleMergeRecordSnapshot {
    Entity(EntityReadRecord),
    Relation(RelationReadRecord),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedTruthWitness {
    pub witness_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciledIdentityBasis {
    pub source_record: RecordRef,
    pub target_record: RecordRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeLineageContinuityVerdict {
    Unchanged,
    Preserved,
    Transformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletedOnBothSidesSemantics {
    AuthoritativeMutualDeletionConvergence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutableAspectPlan {
    AdoptSourceValue {
        aspect_key: worth_foundational::facade::AspectKey,
        source_value: MaterializedAspectValue,
    },
    PreserveSharedValue {
        aspect_key: worth_foundational::facade::AspectKey,
        shared_value: MaterializedAspectValue,
    },
    ReconcileValue {
        aspect_key: worth_foundational::facade::AspectKey,
        source_value: Option<MaterializedAspectValue>,
        target_value: Option<MaterializedAspectValue>,
        base_value: Option<MaterializedAspectValue>,
        resolved_value: Option<MaterializedAspectValue>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeExecutableRecordProvenance {
    pub classification: MergeConflictClass,
    pub resolution_class: crate::merge::data::policy::MergeResolutionClass,
    pub executable_class: crate::merge::data::policy::MergeExecutableClass,
    pub causal_disposition: crate::merge::data::causal::MergeRecordCausalDisposition,
    pub policy_proof_boundary: crate::merge::data::policy::MergePolicyProofBoundary,
    pub applied_policies: Arc<[ResolvedAspectMergePolicy]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdoptSourceRecordPlan {
    pub source_record: RecordRef,
    pub(crate) record_kind: VisibleMergeRecordKind,
    pub source_visible_snapshot: VisibleMergeRecordSnapshot,
    pub provenance: MergeExecutableRecordProvenance,
    pub aspect_plan: Arc<[ExecutableAspectPlan]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreserveSharedRecordPlan {
    pub record: RecordRef,
    pub target_record: Option<RecordRef>,
    pub equality_witness: SharedTruthWitness,
    pub provenance: MergeExecutableRecordProvenance,
    pub aspect_plan: Arc<[ExecutableAspectPlan]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReconcileRecordPlan {
    pub source_record: RecordRef,
    pub target_record: RecordRef,
    pub source_visible_snapshot: VisibleMergeRecordSnapshot,
    pub identity_basis: ReconciledIdentityBasis,
    pub causal_disposition: crate::merge::data::causal::MergeRecordCausalDisposition,
    pub provenance: MergeExecutableRecordProvenance,
    pub aspect_plan: Arc<[ExecutableAspectPlan]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergeDeletedOnBothSidesRecordPlan {
    pub source_record: RecordRef,
    pub target_record: Option<RecordRef>,
    pub equality_witness: SharedTruthWitness,
    pub semantics: DeletedOnBothSidesSemantics,
    pub lineage_continuity: MergeLineageContinuityVerdict,
    pub provenance: MergeExecutableRecordProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum BoundExecutableMergeRecordPlan {
    AdoptSource(AdoptSourceRecordPlan),
    PreserveShared(PreserveSharedRecordPlan),
    Reconcile(ReconcileRecordPlan),
    ConvergeDeletedOnBothSides(ConvergeDeletedOnBothSidesRecordPlan),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundExecutableMergePlan {
    pub authority_binding: MergeExecutionAuthorityBinding,
    pub parent_order: Arc<[CommitId]>,
    pub record_plans: Arc<[BoundExecutableMergeRecordPlan]>,
    pub diagnostics_plan: crate::merge::data::execution_artifacts::MergeExecutionDiagnosticsPlan,
}

pub(crate) fn bound_parent_order(basis: &RelationalMergeBranchBasis) -> Arc<[CommitId]> {
    Arc::from([basis.target_head.commit_id, basis.source_head.commit_id])
}

pub(crate) fn visible_record_snapshot(
    record: &VisibleMergeRecord,
) -> Option<VisibleMergeRecordSnapshot> {
    match record.record_kind {
        VisibleMergeRecordKind::Entity => record
            .source_entity
            .clone()
            .map(VisibleMergeRecordSnapshot::Entity),
        VisibleMergeRecordKind::Relation => record
            .source_relation
            .clone()
            .map(VisibleMergeRecordSnapshot::Relation),
    }
}
