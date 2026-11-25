use crate::contracts::merge::data::visible_record::VisibleMergeRecord;
use std::sync::Arc;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::history::data::merge_branch_basis::RelationalMergeBranchBasis;
use crate::contracts::merge::data::ancestry::MergeAncestrySummary;
use crate::contracts::merge::data::causal::CausalAnnotationSummary;
use crate::contracts::merge::data::decisions::{
    MergePlanningDecisionLog, MergePlanningDecisionLogDigestBasis,
};
use crate::contracts::merge::data::identity::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope, IdentityDiscoverySummary,
    IdentityMatchCandidate,
};
use crate::contracts::merge::data::policy::{
    LoweredMergePlanSummary, MergePolicyResolutionSummary,
};
use crate::contracts::merge::data::requests::normalized::NormalizedRelationalMergeRequest;
use crate::contracts::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchTouchedRecordDelta {
    pub target: RecordRef,
    pub envelopes: Arc<[Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>]>,
}

impl BranchTouchedRecordDelta {
    pub fn commit_ids(&self) -> impl DoubleEndedIterator<Item = CommitId> + ExactSizeIterator + '_ {
        self.envelopes
            .iter()
            .map(|envelope| envelope.commit.commit_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchCommitDelta {
    pub(crate) branch_id: BranchId,
    pub(crate) commits: Arc<[CommitId]>,
    pub touched_records: Arc<[BranchTouchedRecordDelta]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryScopedMergePlan {
    pub request: NormalizedRelationalMergeRequest,
    pub basis: RelationalMergeBranchBasis,
    pub target_delta: BranchCommitDelta,
    pub source_delta: BranchCommitDelta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityScopedMergePlan {
    pub(crate) request: NormalizedRelationalMergeRequest,
    pub basis: RelationalMergeBranchBasis,
    pub(crate) ancestry: MergeAncestrySummary,
    pub(crate) target_delta: BranchCommitDelta,
    pub(crate) source_delta: BranchCommitDelta,
    pub(crate) effective_identity_declarations: Arc<[IdentityBasisDeclaration]>,
    pub(crate) source_records: Arc<[VisibleMergeRecord]>,
    pub(crate) candidates: Arc<[IdentityMatchCandidate]>,
    pub(crate) validated_schema_correspondences: Arc<[ValidatedSchemaDeclaredCorrespondence]>,
    pub(crate) identity_summary: IdentityDiscoverySummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictClassifiedMergePlan {
    pub(crate) request: NormalizedRelationalMergeRequest,
    pub(crate) basis: RelationalMergeBranchBasis,
    pub(crate) ancestry: MergeAncestrySummary,
    pub(crate) target_delta: BranchCommitDelta,
    pub(crate) source_delta: BranchCommitDelta,
    pub(crate) effective_identity_declarations: Arc<[IdentityBasisDeclaration]>,
    pub(crate) source_records: Arc<[VisibleMergeRecord]>,
    pub(crate) candidates: Arc<[IdentityMatchCandidate]>,
    pub(crate) validated_schema_correspondences: Arc<[ValidatedSchemaDeclaredCorrespondence]>,
    pub(crate) identity_summary: IdentityDiscoverySummary,
    pub(crate) classifications:
        Arc<[crate::contracts::merge::data::conflicts::MergeConflictClassification]>,
    pub(crate) conflict_summary:
        crate::contracts::merge::data::conflicts::ConflictClassificationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausallyAnnotatedMergePlan {
    pub(crate) request: NormalizedRelationalMergeRequest,
    pub basis: RelationalMergeBranchBasis,
    pub(crate) ancestry: MergeAncestrySummary,
    pub(crate) target_delta: BranchCommitDelta,
    pub(crate) source_delta: BranchCommitDelta,
    pub(crate) effective_identity_declarations: Arc<[IdentityBasisDeclaration]>,
    pub(crate) source_records: Arc<[VisibleMergeRecord]>,
    pub(crate) candidates: Arc<[IdentityMatchCandidate]>,
    pub(crate) validated_schema_correspondences: Arc<[ValidatedSchemaDeclaredCorrespondence]>,
    pub(crate) identity_summary: IdentityDiscoverySummary,
    pub(crate) classifications:
        Arc<[crate::contracts::merge::data::conflicts::MergeConflictClassification]>,
    pub(crate) conflict_summary:
        crate::contracts::merge::data::conflicts::ConflictClassificationSummary,
    pub(crate) causal_annotations:
        Arc<[crate::contracts::merge::data::causal::MergeRecordCausalAnnotation]>,
    pub(crate) causal_summary: CausalAnnotationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyResolvedMergePlan {
    pub(crate) request: NormalizedRelationalMergeRequest,
    pub(crate) basis: RelationalMergeBranchBasis,
    pub(crate) ancestry: MergeAncestrySummary,
    pub(crate) target_delta: BranchCommitDelta,
    pub(crate) source_delta: BranchCommitDelta,
    pub(crate) effective_identity_declarations: Arc<[IdentityBasisDeclaration]>,
    pub(crate) source_records: Arc<[VisibleMergeRecord]>,
    pub(crate) candidates: Arc<[IdentityMatchCandidate]>,
    pub(crate) validated_schema_correspondences: Arc<[ValidatedSchemaDeclaredCorrespondence]>,
    pub(crate) identity_summary: IdentityDiscoverySummary,
    pub(crate) classifications:
        Arc<[crate::contracts::merge::data::conflicts::MergeConflictClassification]>,
    pub(crate) conflict_summary:
        crate::contracts::merge::data::conflicts::ConflictClassificationSummary,
    pub(crate) causal_annotations:
        Arc<[crate::contracts::merge::data::causal::MergeRecordCausalAnnotation]>,
    pub(crate) causal_summary: CausalAnnotationSummary,
    pub(crate) policy_records:
        Arc<[crate::contracts::merge::data::policy::MergePolicyResolutionRecord]>,
    pub(crate) policy_summary: MergePolicyResolutionSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredMergePlan {
    pub(crate) request: NormalizedRelationalMergeRequest,
    pub basis: RelationalMergeBranchBasis,
    pub ancestry: MergeAncestrySummary,
    pub target_delta: BranchCommitDelta,
    pub(crate) source_delta: BranchCommitDelta,
    pub(crate) effective_identity_declarations: Arc<[IdentityBasisDeclaration]>,
    pub source_records: Arc<[VisibleMergeRecord]>,
    pub(crate) candidates: Arc<[IdentityMatchCandidate]>,
    pub(crate) validated_schema_correspondences: Arc<[ValidatedSchemaDeclaredCorrespondence]>,
    pub identity_summary: IdentityDiscoverySummary,
    pub(crate) classifications:
        Arc<[crate::contracts::merge::data::conflicts::MergeConflictClassification]>,
    pub conflict_summary: crate::contracts::merge::data::conflicts::ConflictClassificationSummary,
    pub(crate) causal_annotations:
        Arc<[crate::contracts::merge::data::causal::MergeRecordCausalAnnotation]>,
    pub causal_summary: CausalAnnotationSummary,
    pub(crate) policy_records:
        Arc<[crate::contracts::merge::data::policy::MergePolicyResolutionRecord]>,
    pub policy_summary: MergePolicyResolutionSummary,
    pub(crate) lowered_records:
        Arc<[crate::contracts::merge::data::policy::LoweredMergePlanRecord]>,
    pub lowered_summary: LoweredMergePlanSummary,
    pub decision_log: MergePlanningDecisionLog,
    pub(crate) decision_log_digest_basis: MergePlanningDecisionLogDigestBasis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedSchemaDeclaredCorrespondence {
    pub(crate) scope: IdentityBasisScope,
    pub(crate) basis: IdentityBasisKind,
    pub(crate) source_record: RecordRef,
    pub(crate) target_record: RecordRef,
    pub(crate) candidate_count_for_source: usize,
    pub(crate) candidate_count_for_target: usize,
}
