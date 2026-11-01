use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::history::data::{BranchId, CommitId};
use crate::merge::data::{
    AspectComparisonState, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
    IdentityMatchClass, IdentityResolutionReason, LoweredMergeAction, LoweredMergeBlockedReason,
    LoweredMergeRejectedReason, LoweredRecordDecisionKind, LoweredRecordDenialKind,
    LoweredRecordExecutionIntentKind, MergeBaseSelectionRule, MergeConflictClass,
    MergeExecutableClass, MergeExecutionReadiness, MergeIntent,
    MergePlanningDecisionLogDigestBasis, MergePolicyProofBoundary, MergeRecordCausalDisposition,
    MergeResolutionClass, MergeVisibilityEvidence, RelationConflictEvidence,
    RelationalMergeCorrespondencePosture, RelationalMergeRequestFamily,
    RelationalMergeSchemaReconciliationPosture, RelationalMergeTopologyIntent,
    ResolvedAspectMergePolicy, StrategyConflictClass,
};
use crate::transactions::data::RecordRef;

use super::{
    MergeExecutionAuthorityContract, MergeLoweredAspectDigestRow, MergePolicyAspectDigestRow,
    MergeSchemaSnapshotDigestBasis,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeRequestDigestBasis {
    pub target_branch: BranchId,
    pub source_branch: BranchId,
    pub merge_intent: MergeIntent,
    pub family: RelationalMergeRequestFamily,
    pub correspondence_posture: RelationalMergeCorrespondencePosture,
    pub schema_reconciliation_posture: RelationalMergeSchemaReconciliationPosture,
    pub topology_intent: RelationalMergeTopologyIntent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeBaseDigestBasis {
    pub rule: MergeBaseSelectionRule,
    pub commit_id: CommitId,
    pub supporting_left_ancestors: crate::history::data::MergeBasisAncestors,
    pub supporting_right_ancestors: crate::history::data::MergeBasisAncestors,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeIdentityDigestBasis {
    pub effective_declarations: Arc<[IdentityBasisDeclaration]>,
    pub candidate_scopes: Arc<[Option<IdentityBasisScope>]>,
    pub candidate_sources: Arc<[RecordRef]>,
    pub candidate_targets: Arc<[Option<RecordRef>]>,
    pub candidate_bases: Arc<[IdentityBasisKind]>,
    pub candidate_match_classes: Arc<[IdentityMatchClass]>,
    pub candidate_reasons: Arc<[IdentityResolutionReason]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeCausalDigestBasis {
    pub records: Arc<[RecordRef]>,
    pub dispositions: Arc<[MergeRecordCausalDisposition]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeConflictDigestBasis {
    pub records: Arc<[RecordRef]>,
    pub classes: Arc<[MergeConflictClass]>,
    pub validated_schema_correspondence: Arc<[bool]>,
    pub strategy_conflict_classes: Arc<[Option<StrategyConflictClass>]>,
    pub source_strategy_descriptors:
        Arc<[Arc<[crate::commit_strategies::data::StrategyMergeDescriptor]>]>,
    pub target_strategy_descriptors:
        Arc<[Arc<[crate::commit_strategies::data::StrategyMergeDescriptor]>]>,
    pub relation_evidence: Arc<[Option<RelationConflictEvidence>]>,
    pub source_visibility_evidence: Arc<[MergeVisibilityEvidence]>,
    pub target_visibility_evidence: Arc<[MergeVisibilityEvidence]>,
    pub base_visibility_evidence: Arc<[MergeVisibilityEvidence]>,
    pub aspect_evidence_keys: Arc<[Arc<[worth_foundational::facade::AspectKey]>]>,
    pub aspect_evidence_comparisons: Arc<[Arc<[AspectComparisonState]>]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePolicyDigestBasis {
    pub records: Arc<[RecordRef]>,
    pub proof_boundaries: Arc<[MergePolicyProofBoundary]>,
    pub applied_policies: Arc<[Arc<[ResolvedAspectMergePolicy]>]>,
    pub aspect_rows: Arc<[Arc<[MergePolicyAspectDigestRow]>]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeLoweredPlanDigestBasis {
    pub records: Arc<[RecordRef]>,
    pub readiness: Arc<[MergeExecutionReadiness]>,
    pub resolution_classes: Arc<[MergeResolutionClass]>,
    pub executable_classes: Arc<[Option<MergeExecutableClass>]>,
    pub record_decisions: Arc<[LoweredRecordDecisionKind]>,
    pub lowered_actions: Arc<[Option<LoweredMergeAction>]>,
    pub blocked_reasons: Arc<[Option<LoweredMergeBlockedReason>]>,
    pub rejected_reasons: Arc<[Option<LoweredMergeRejectedReason>]>,
    pub execution_bundle_kinds: Arc<[Option<LoweredRecordExecutionIntentKind>]>,
    pub denial_bundle_kinds: Arc<[Option<LoweredRecordDenialKind>]>,
    pub aspect_rows: Arc<[Arc<[MergeLoweredAspectDigestRow]>]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeArtifactDigestBasis {
    pub request: MergeRequestDigestBasis,
    pub schema: MergeSchemaSnapshotDigestBasis,
    pub execution_contract: MergeExecutionAuthorityContract,
    pub merge_base: MergeBaseDigestBasis,
    pub identity: MergeIdentityDigestBasis,
    pub causal: MergeCausalDigestBasis,
    pub conflict: MergeConflictDigestBasis,
    pub policy: MergePolicyDigestBasis,
    pub lowered_plan: MergeLoweredPlanDigestBasis,
    pub decision_log: MergePlanningDecisionLogDigestBasis,
}
