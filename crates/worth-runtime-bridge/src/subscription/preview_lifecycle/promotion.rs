use std::collections::BTreeSet;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{
    BridgeSubscriptionPreviewLifecycleResidueEnvelope,
    BridgeSubscriptionPreviewLifecycleResidueKind,
};
use crate::preview::FinishedBridgePreviewSession;
use crate::subscription::{
    BridgeAdmittedSubscriptionIdentity, BridgePreviewActiveSubscription,
    BridgePreviewActiveSubscriptionIdentity, BridgeSubscriptionCounters,
    BridgeSubscriptionPreviewBasisIdentity, BridgeSubscriptionPreviewLifecyclePromotionIdentity,
    BridgeSubscriptionPreviewResidueScopeIdentity, BridgeSubscriptionPreviewScopeIdentity,
    BridgeSubscriptionPreviewWorkTrace, BridgeSubscriptionPreviewWorkTraceIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionPreviewLifecyclePromotionRejectionKind {
    PreviewWorkTraceMismatch,
    PromotionSessionMismatch,
    ResidueEnvelopeMismatch,
    MissingResidueKind,
    DuplicateResidueKind,
    PreviewCrossedCompletion,
    TemporalEvidenceDrift,
}

impl BridgeSubscriptionPreviewLifecyclePromotionRejectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreviewWorkTraceMismatch => "preview_work_trace_mismatch",
            Self::PromotionSessionMismatch => "promotion_session_mismatch",
            Self::ResidueEnvelopeMismatch => "residue_envelope_mismatch",
            Self::MissingResidueKind => "missing_residue_kind",
            Self::DuplicateResidueKind => "duplicate_residue_kind",
            Self::PreviewCrossedCompletion => "preview_crossed_completion",
            Self::TemporalEvidenceDrift => "temporal_evidence_drift",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecyclePromotionRejection {
    rejection_kind: BridgeSubscriptionPreviewLifecyclePromotionRejectionKind,
    rejection_context: Arc<str>,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecyclePromotionRejection {
    fn new(
        rejection_kind: BridgeSubscriptionPreviewLifecyclePromotionRejectionKind,
        rejection_context: impl Into<Arc<str>>,
    ) -> Self {
        let rejection_context = rejection_context.into();
        let canonical_basis = format!(
            "bridge-subscription-preview-lifecycle-promotion-rejection|kind={}|context={}",
            rejection_kind.as_str(),
            rejection_context.as_ref(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let counters = BridgeSubscriptionCounters::from_subscription_preview_promotion_rejection(
            rejection_kind
                == BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::PreviewCrossedCompletion,
            rejection_kind
                == BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::TemporalEvidenceDrift,
        );
        Self {
            rejection_kind,
            rejection_context,
            counters,
            digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-promotion-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgeSubscriptionPreviewLifecyclePromotionRejectionKind {
        self.rejection_kind
    }

    pub fn rejection_context(&self) -> &str {
        self.rejection_context.as_ref()
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecyclePromotion {
    promotion_identity: BridgeSubscriptionPreviewLifecyclePromotionIdentity,
    preview_active_subscription_identity: BridgePreviewActiveSubscriptionIdentity,
    admitted_subscription_identity: BridgeAdmittedSubscriptionIdentity,
    preview_basis_identity: BridgeSubscriptionPreviewBasisIdentity,
    preview_scope_identity: BridgeSubscriptionPreviewScopeIdentity,
    preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
    preview_work_trace_identity: BridgeSubscriptionPreviewWorkTraceIdentity,
    residue_envelope_digest: Arc<str>,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecyclePromotion {
    pub(crate) fn admit(
        preview_active: &BridgePreviewActiveSubscription,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
        residue_envelope: &BridgeSubscriptionPreviewLifecycleResidueEnvelope,
        finished: &FinishedBridgePreviewSession,
    ) -> Result<Self, BridgeSubscriptionPreviewLifecyclePromotionRejection> {
        if preview_work_trace.preview_active_subscription_identity()
            != preview_active.preview_active_subscription_identity()
            || preview_work_trace.preview_scope_identity()
                != preview_active.preview_scope_identity()
        {
            return Err(BridgeSubscriptionPreviewLifecyclePromotionRejection::new(
                BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::PreviewWorkTraceMismatch,
                format!(
                    "preview-active={}|trace-preview-active={}|preview-scope={}|trace-scope={}",
                    preview_active
                        .preview_active_subscription_identity()
                        .as_str(),
                    preview_work_trace
                        .preview_active_subscription_identity()
                        .as_str(),
                    preview_active.preview_scope_identity().as_str(),
                    preview_work_trace.preview_scope_identity().as_str(),
                ),
            ));
        }
        if !finished.belongs_to(&preview_active.session) {
            return Err(BridgeSubscriptionPreviewLifecyclePromotionRejection::new(
                BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::PromotionSessionMismatch,
                finished.session_identity().as_str(),
            ));
        }
        if residue_envelope.preview_active_subscription_identity()
            != preview_active.preview_active_subscription_identity()
            || residue_envelope.preview_scope_identity() != preview_active.preview_scope_identity()
            || residue_envelope.preview_residue_scope_identity()
                != preview_active.preview_residue_scope_identity()
            || residue_envelope.preview_work_trace_identity()
                != preview_work_trace.preview_work_trace_identity()
            || residue_envelope.preview_work_trace_digest() != preview_work_trace.digest()
        {
            return Err(BridgeSubscriptionPreviewLifecyclePromotionRejection::new(
                BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::ResidueEnvelopeMismatch,
                format!(
                    "preview-active={}|envelope-preview-active={}|preview-scope={}|envelope-scope={}|preview-residue-scope={}|envelope-residue-scope={}|preview-work-trace={}|envelope-preview-work-trace={}|preview-work-digest={}|envelope-preview-work-digest={}",
                    preview_active.preview_active_subscription_identity().as_str(),
                    residue_envelope.preview_active_subscription_identity().as_str(),
                    preview_active.preview_scope_identity().as_str(),
                    residue_envelope.preview_scope_identity().as_str(),
                    preview_active.preview_residue_scope_identity().as_str(),
                    residue_envelope.preview_residue_scope_identity().as_str(),
                    preview_work_trace.preview_work_trace_identity().as_str(),
                    residue_envelope.preview_work_trace_identity().as_str(),
                    preview_work_trace.digest(),
                    residue_envelope.preview_work_trace_digest(),
                ),
            ));
        }

        let mut seen = BTreeSet::new();
        for record in residue_envelope.residue_records() {
            if !seen.insert(record.kind()) {
                return Err(BridgeSubscriptionPreviewLifecyclePromotionRejection::new(
                    BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::DuplicateResidueKind,
                    record.kind().as_str(),
                ));
            }
            let expected_record_digest =
                preview_work_trace.record_digest_for(record.kind().preview_work_kind());
            let expected_evidence = format!(
                "bridge-subscription-preview-lifecycle-residue-evidence|trace={}|scope={}|record={expected_record_digest}|kind={}",
                preview_work_trace.digest(),
                preview_work_trace.preview_residue_scope_identity().as_str(),
                record.kind().as_str(),
            );
            if record.kind() == BridgeSubscriptionPreviewLifecycleResidueKind::TemporalWake
                && record.evidence_digest() != expected_evidence
            {
                return Err(BridgeSubscriptionPreviewLifecyclePromotionRejection::new(
                    BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::TemporalEvidenceDrift,
                    format!(
                        "expected={expected_evidence}|actual={}",
                        record.evidence_digest()
                    ),
                ));
            }
            if record.kind() == BridgeSubscriptionPreviewLifecycleResidueKind::CompletionWriteback
                && record.residue_count() != 0
            {
                return Err(BridgeSubscriptionPreviewLifecyclePromotionRejection::new(
                    BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::PreviewCrossedCompletion,
                    format!("completion-residue={}", record.residue_count()),
                ));
            }
        }
        for required in BridgeSubscriptionPreviewLifecycleResidueKind::all() {
            if !seen.contains(&required) {
                return Err(BridgeSubscriptionPreviewLifecyclePromotionRejection::new(
                    BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::MissingResidueKind,
                    required.as_str(),
                ));
            }
        }

        let canonical_basis = format!(
            "bridge-subscription-preview-lifecycle-promotion|preview-active={}|preview-basis={}|preview-scope={}|residue-scope={}|preview-work-trace={}|residue-envelope={}",
            preview_active.preview_active_subscription_identity().as_str(),
            preview_active.preview_basis_identity().as_str(),
            preview_active.preview_scope_identity().as_str(),
            preview_active.preview_residue_scope_identity().as_str(),
            preview_work_trace.preview_work_trace_identity().as_str(),
            residue_envelope.digest(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            promotion_identity:
                BridgeSubscriptionPreviewLifecyclePromotionIdentity::admit_bridge_owned(format!(
                    "bridge-subscription-preview-lifecycle-promotion-id:sha256:{digest:x}"
                )),
            preview_active_subscription_identity: preview_active
                .preview_active_subscription_identity()
                .clone(),
            admitted_subscription_identity: preview_active.admitted_subscription_identity().clone(),
            preview_basis_identity: preview_active.preview_basis_identity().clone(),
            preview_scope_identity: preview_active.preview_scope_identity().clone(),
            preview_residue_scope_identity: preview_active.preview_residue_scope_identity().clone(),
            preview_work_trace_identity: preview_work_trace.preview_work_trace_identity().clone(),
            residue_envelope_digest: Arc::from(residue_envelope.digest()),
            counters: BridgeSubscriptionCounters::from_subscription_preview_promotion(),
            digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-promotion:sha256:{digest:x}"
            )),
        })
    }

    pub fn promotion_identity(&self) -> &BridgeSubscriptionPreviewLifecyclePromotionIdentity {
        &self.promotion_identity
    }

    pub fn preview_active_subscription_identity(&self) -> &BridgePreviewActiveSubscriptionIdentity {
        &self.preview_active_subscription_identity
    }

    pub fn admitted_subscription_identity(&self) -> &BridgeAdmittedSubscriptionIdentity {
        &self.admitted_subscription_identity
    }

    pub fn preview_basis_identity(&self) -> &BridgeSubscriptionPreviewBasisIdentity {
        &self.preview_basis_identity
    }

    pub fn preview_scope_identity(&self) -> &BridgeSubscriptionPreviewScopeIdentity {
        &self.preview_scope_identity
    }

    pub fn preview_residue_scope_identity(&self) -> &BridgeSubscriptionPreviewResidueScopeIdentity {
        &self.preview_residue_scope_identity
    }

    pub fn preview_work_trace_identity(&self) -> &BridgeSubscriptionPreviewWorkTraceIdentity {
        &self.preview_work_trace_identity
    }

    pub fn residue_envelope_digest(&self) -> &str {
        self.residue_envelope_digest.as_ref()
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
