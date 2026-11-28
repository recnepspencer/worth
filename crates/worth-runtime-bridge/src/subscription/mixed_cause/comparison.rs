use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::input::envelope::BridgeCommittedPatchEnvelope;
use crate::source::BridgeAsyncRequestTruthViewBasisKind;
use crate::source::{
    AdmittedBridgeAsyncCompletion, BridgeAsyncClassifiedDeniedCompletion,
    BridgeAsyncDeniedCompletion, BridgeAsyncRetryLineage, BridgeAsyncRevalidationLineage,
};
use crate::subscription::{
    BridgeTemporalCauseClassification, BridgeTemporalCauseRecord, BridgeTemporalRoutingLaneKind,
};

use super::async_result_transition::BridgeMixedCauseAsyncResultTransitionSeed;
use super::ordering::{BridgeMixedCauseDeniedKind, BridgeMixedCauseOrderFamilyKind};
use super::request::BridgeMixedCauseOrderingInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMixedCauseComparisonReasonKind {
    RootAdmission,
    PriorityClass,
    CanonicalDigestTieBreak,
    DuplicateDigestSuppression,
    AuthoritativePreviewRejection,
    AsyncStaleDenial,
    AsyncLineageNonDeliverable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMixedCauseComparisonEvidence {
    reason_kind: BridgeMixedCauseComparisonReasonKind,
    current_family_kind: BridgeMixedCauseOrderFamilyKind,
    current_source_identity: Arc<str>,
    compared_family_kind: Option<BridgeMixedCauseOrderFamilyKind>,
    compared_source_identity: Option<Arc<str>>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeMixedCauseComparisonEvidence {
    pub(super) fn root(candidate: &Candidate) -> Self {
        Self::new(
            BridgeMixedCauseComparisonReasonKind::RootAdmission,
            candidate,
            None,
        )
    }

    pub(super) fn ordered_after(candidate: &Candidate, prior: &Candidate) -> Self {
        let reason_kind = if candidate.precedence != prior.precedence {
            BridgeMixedCauseComparisonReasonKind::PriorityClass
        } else {
            BridgeMixedCauseComparisonReasonKind::CanonicalDigestTieBreak
        };
        Self::new(reason_kind, candidate, Some(prior))
    }

    pub(super) fn duplicate(candidate: &Candidate, exemplar: &Candidate) -> Self {
        Self::new(
            BridgeMixedCauseComparisonReasonKind::DuplicateDigestSuppression,
            candidate,
            Some(exemplar),
        )
    }

    pub(super) fn denied(candidate: &Candidate, denied_kind: BridgeMixedCauseDeniedKind) -> Self {
        let reason_kind = match denied_kind {
            BridgeMixedCauseDeniedKind::AuthoritativePreviewCauseRejected => {
                BridgeMixedCauseComparisonReasonKind::AuthoritativePreviewRejection
            }
            BridgeMixedCauseDeniedKind::AsyncStaleCauseRejected => {
                BridgeMixedCauseComparisonReasonKind::AsyncStaleDenial
            }
            BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable => {
                BridgeMixedCauseComparisonReasonKind::AsyncLineageNonDeliverable
            }
        };
        Self::new(reason_kind, candidate, None)
    }

    fn new(
        reason_kind: BridgeMixedCauseComparisonReasonKind,
        current: &Candidate,
        compared: Option<&Candidate>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-mixed-cause-comparison-evidence|reason={reason_kind:?}|current-family={:?}|current-source={}|compared-family={}|compared-source={}",
            current.family_kind,
            current.source_identity,
            compared
                .map(|candidate| format!("{:?}", candidate.family_kind))
                .unwrap_or_else(|| "-".to_owned()),
            compared
                .map(|candidate| candidate.source_identity.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            reason_kind,
            current_family_kind: current.family_kind,
            current_source_identity: current.source_identity.clone(),
            compared_family_kind: compared.map(|candidate| candidate.family_kind),
            compared_source_identity: compared.map(|candidate| candidate.source_identity.clone()),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-mixed-cause-comparison-evidence:sha256:{digest:x}"
            )),
        }
    }

    pub fn reason_kind(&self) -> BridgeMixedCauseComparisonReasonKind {
        self.reason_kind
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone)]
pub(super) struct Candidate {
    pub(super) family_kind: BridgeMixedCauseOrderFamilyKind,
    pub(super) source_identity: Arc<str>,
    pub(super) source_digest: Arc<str>,
    pub(super) dedup_key: Arc<str>,
    pub(super) precedence: u8,
    pub(super) preview_local: bool,
    pub(super) stale_or_nondeliverable: Option<BridgeMixedCauseDeniedKind>,
    pub(super) async_result_transition: Option<BridgeMixedCauseAsyncResultTransitionSeed>,
}

impl Candidate {
    pub(super) fn from_input(input: &BridgeMixedCauseOrderingInput) -> Self {
        match input {
            BridgeMixedCauseOrderingInput::TruthPatch(patch) => Self::from_truth_patch(patch),
            BridgeMixedCauseOrderingInput::Temporal(cause) => Self::from_temporal(cause),
            BridgeMixedCauseOrderingInput::AsyncCompletion(completion) => {
                Self::from_async_completion(completion)
            }
            BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(completion) => {
                Self::from_async_denied_completion(completion)
            }
            BridgeMixedCauseOrderingInput::AsyncClassifiedDeniedCompletion(denied) => {
                Self::from_async_denied(denied)
            }
            BridgeMixedCauseOrderingInput::AsyncRetryLineage(lineage) => {
                Self::from_retry_lineage(lineage)
            }
            BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(lineage) => {
                Self::from_revalidation_lineage(lineage)
            }
        }
    }

    pub(super) fn sort_key(&self) -> (u8, &str) {
        (self.precedence, self.source_digest.as_ref())
    }

    pub(super) fn denied_kind(
        &self,
        lane_kind: super::request::BridgeMixedCauseOrderingLaneKind,
    ) -> Option<BridgeMixedCauseDeniedKind> {
        if matches!(
            lane_kind,
            super::request::BridgeMixedCauseOrderingLaneKind::Authoritative
        ) && self.preview_local
        {
            return Some(BridgeMixedCauseDeniedKind::AuthoritativePreviewCauseRejected);
        }
        self.stale_or_nondeliverable
    }

    fn from_truth_patch(patch: &BridgeCommittedPatchEnvelope) -> Self {
        Self {
            family_kind: BridgeMixedCauseOrderFamilyKind::TruthPatch,
            source_identity: Arc::from(patch.patch_identity().canonical_label().to_owned()),
            source_digest: Arc::from(patch.digest().as_str().to_owned()),
            dedup_key: Arc::from(format!("truth-patch:{}", patch.digest().as_str())),
            precedence: 0,
            preview_local: false,
            stale_or_nondeliverable: None,
            async_result_transition: None,
        }
    }

    fn from_temporal(cause: &BridgeTemporalCauseRecord) -> Self {
        Self {
            family_kind: match cause.classification() {
                BridgeTemporalCauseClassification::TruthPlusTime => {
                    BridgeMixedCauseOrderFamilyKind::TemporalTruthPlusTime
                }
                BridgeTemporalCauseClassification::TimeOnly => {
                    BridgeMixedCauseOrderFamilyKind::TemporalTimeOnly
                }
            },
            source_identity: Arc::from(cause.cause_record_identity().as_str().to_owned()),
            source_digest: Arc::from(cause.digest().to_owned()),
            dedup_key: Arc::from(format!("temporal:{}", cause.digest())),
            precedence: match cause.classification() {
                BridgeTemporalCauseClassification::TruthPlusTime => 1,
                BridgeTemporalCauseClassification::TimeOnly => 3,
            },
            preview_local: matches!(
                cause.routing_lane_kind(),
                BridgeTemporalRoutingLaneKind::Preview
            ),
            stale_or_nondeliverable: None,
            async_result_transition: None,
        }
    }

    fn from_async_completion(completion: &AdmittedBridgeAsyncCompletion) -> Self {
        Self {
            family_kind: BridgeMixedCauseOrderFamilyKind::AsyncCompletion,
            source_identity: Arc::from(completion.completion_identity().to_owned()),
            source_digest: Arc::from(completion.digest().to_owned()),
            dedup_key: Arc::from(format!("async-completion:{}", completion.digest())),
            precedence: 2,
            preview_local: matches!(
                completion
                    .request_identity()
                    .basis_binding()
                    .truth_view_basis_kind(),
                BridgeAsyncRequestTruthViewBasisKind::Preview
            ),
            stale_or_nondeliverable: None,
            async_result_transition: Some(
                BridgeMixedCauseAsyncResultTransitionSeed::from_completion(completion),
            ),
        }
    }

    fn from_async_denied_completion(completion: &BridgeAsyncDeniedCompletion) -> Self {
        Self {
            family_kind: BridgeMixedCauseOrderFamilyKind::AsyncDeniedCompletion,
            source_identity: Arc::from(completion.denial_identity().to_owned()),
            source_digest: Arc::from(completion.digest().to_owned()),
            dedup_key: Arc::from(format!("async-denied-completion:{}", completion.digest())),
            precedence: 2,
            preview_local: matches!(
                completion
                    .request_identity()
                    .basis_binding()
                    .truth_view_basis_kind(),
                BridgeAsyncRequestTruthViewBasisKind::Preview
            ),
            stale_or_nondeliverable: None,
            async_result_transition: Some(
                BridgeMixedCauseAsyncResultTransitionSeed::from_denied_completion(completion),
            ),
        }
    }

    fn from_async_denied(denied: &BridgeAsyncClassifiedDeniedCompletion) -> Self {
        Self {
            family_kind: BridgeMixedCauseOrderFamilyKind::AsyncClassifiedDeniedCompletion,
            source_identity: Arc::from(denied.denied_completion().denial_identity().to_owned()),
            source_digest: Arc::from(denied.receipt().digest().to_owned()),
            dedup_key: Arc::from(format!("async-denied:{}", denied.receipt().digest())),
            precedence: 4,
            preview_local: matches!(
                denied
                    .denied_completion()
                    .request_identity()
                    .basis_binding()
                    .truth_view_basis_kind(),
                BridgeAsyncRequestTruthViewBasisKind::Preview
            ),
            stale_or_nondeliverable: Some(BridgeMixedCauseDeniedKind::AsyncStaleCauseRejected),
            async_result_transition: Some(
                BridgeMixedCauseAsyncResultTransitionSeed::from_classified_denied(denied),
            ),
        }
    }

    fn from_retry_lineage(lineage: &BridgeAsyncRetryLineage) -> Self {
        Self {
            family_kind: BridgeMixedCauseOrderFamilyKind::AsyncRetryLineage,
            source_identity: Arc::from(lineage.causality_identity().to_owned()),
            source_digest: Arc::from(lineage.digest().to_owned()),
            dedup_key: Arc::from(format!("async-retry:{}", lineage.digest())),
            precedence: 5,
            preview_local: matches!(
                lineage
                    .newer_request()
                    .basis_binding()
                    .truth_view_basis_kind(),
                BridgeAsyncRequestTruthViewBasisKind::Preview
            ),
            stale_or_nondeliverable: Some(BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable),
            async_result_transition: Some(BridgeMixedCauseAsyncResultTransitionSeed::from_retry(
                lineage,
            )),
        }
    }

    fn from_revalidation_lineage(lineage: &BridgeAsyncRevalidationLineage) -> Self {
        Self {
            family_kind: BridgeMixedCauseOrderFamilyKind::AsyncRevalidationLineage,
            source_identity: Arc::from(lineage.causality_identity().to_owned()),
            source_digest: Arc::from(lineage.digest().to_owned()),
            dedup_key: Arc::from(format!("async-revalidation:{}", lineage.digest())),
            precedence: 6,
            preview_local: matches!(
                lineage
                    .newer_request()
                    .basis_binding()
                    .truth_view_basis_kind(),
                BridgeAsyncRequestTruthViewBasisKind::Preview
            ),
            stale_or_nondeliverable: Some(BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable),
            async_result_transition: Some(
                BridgeMixedCauseAsyncResultTransitionSeed::from_revalidation(lineage),
            ),
        }
    }
}
