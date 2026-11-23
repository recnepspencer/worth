use super::visible_record::VisibleMergeRecord;
use std::sync::Arc;

use crate::history::data::coordinates::{BranchId, CommitId};
use crate::history::data::merge_branch_basis::RelationalMergeBranchBasis;
use crate::merge::data::ancestry::MergeAncestrySummary;
use crate::merge::data::causal::CausalAnnotationSummary;
use crate::merge::data::decisions::{
    MergePlanningDecisionLog, MergePlanningDecisionLogDigestBasis,
};
use crate::merge::data::identity::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope, IdentityDiscoverySummary,
    IdentityMatchCandidate,
};
use crate::merge::data::policy::{LoweredMergePlanSummary, MergePolicyResolutionSummary};
use crate::merge::data::requests::normalized::NormalizedRelationalMergeRequest;
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BranchTouchedRecordDelta {
    pub(crate) target: RecordRef,
    pub(crate) envelopes: Arc<[Arc<crate::history::data::CanonicalCommitEnvelope>]>,
}

impl BranchTouchedRecordDelta {
    pub(crate) fn commit_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = CommitId> + ExactSizeIterator + '_ {
        self.envelopes
            .iter()
            .map(|envelope| envelope.commit.commit_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BranchCommitDelta {
    pub(crate) branch_id: BranchId,
    pub(crate) commits: Arc<[CommitId]>,
    pub(crate) touched_records: Arc<[BranchTouchedRecordDelta]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryScopedMergePlan {
    pub(crate) request: NormalizedRelationalMergeRequest,
    pub(crate) basis: RelationalMergeBranchBasis,
    pub(crate) target_delta: BranchCommitDelta,
    pub(crate) source_delta: BranchCommitDelta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdentityScopedMergePlan {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConflictClassifiedMergePlan {
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
    pub(crate) classifications: Arc<[crate::merge::data::conflicts::MergeConflictClassification]>,
    pub(crate) conflict_summary: crate::merge::data::conflicts::ConflictClassificationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CausallyAnnotatedMergePlan {
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
    pub(crate) classifications: Arc<[crate::merge::data::conflicts::MergeConflictClassification]>,
    pub(crate) conflict_summary: crate::merge::data::conflicts::ConflictClassificationSummary,
    pub(crate) causal_annotations: Arc<[crate::merge::data::causal::MergeRecordCausalAnnotation]>,
    pub(crate) causal_summary: CausalAnnotationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicyResolvedMergePlan {
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
    pub(crate) classifications: Arc<[crate::merge::data::conflicts::MergeConflictClassification]>,
    pub(crate) conflict_summary: crate::merge::data::conflicts::ConflictClassificationSummary,
    pub(crate) causal_annotations: Arc<[crate::merge::data::causal::MergeRecordCausalAnnotation]>,
    pub(crate) causal_summary: CausalAnnotationSummary,
    pub(crate) policy_records: Arc<[crate::merge::data::policy::MergePolicyResolutionRecord]>,
    pub(crate) policy_summary: MergePolicyResolutionSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoweredMergePlan {
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
    pub(crate) classifications: Arc<[crate::merge::data::conflicts::MergeConflictClassification]>,
    pub(crate) conflict_summary: crate::merge::data::conflicts::ConflictClassificationSummary,
    pub(crate) causal_annotations: Arc<[crate::merge::data::causal::MergeRecordCausalAnnotation]>,
    pub(crate) causal_summary: CausalAnnotationSummary,
    pub(crate) policy_records: Arc<[crate::merge::data::policy::MergePolicyResolutionRecord]>,
    pub(crate) policy_summary: MergePolicyResolutionSummary,
    pub(crate) lowered_records: Arc<[crate::merge::data::policy::LoweredMergePlanRecord]>,
    pub(crate) lowered_summary: LoweredMergePlanSummary,
    pub(crate) decision_log: MergePlanningDecisionLog,
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
